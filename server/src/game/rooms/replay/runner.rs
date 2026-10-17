use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use grenadiers_core::{
    game::{GENERATOR_VERSION, SeatId},
    protocol::{MAX_COUNTER, RoomCode, RoomId, RulesVersion},
    room::MatchPhase,
};

use super::super::{
    match_runtime::{MatchAction, MatchRuntime, RuntimeInspection},
    profile::RoomProfile,
    types::RoomIdentity,
};
use super::record::{
    CLASSIC_V2_RULES, CLASSIC_V3_RULES, FORMAT_VERSION, GENERATED_V1, Journal,
    M5_LIFECYCLE_PROFILE, M6_LIFECYCLE_PROFILE, Operation, ReplayAction, ReplayStatus,
};
use crate::round_seed::RoundSeedSource;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReplayError {
    Truncated,
    UnsupportedProfile,
    Incomplete,
    UnsupportedFormat,
    UnsupportedLifecycle,
    UnsupportedRules,
    UnsupportedGenerator,
    UnsupportedInitialTick,
    InvalidOrder,
    WrongTick,
    InvalidBounds,
    MissingSeed,
    MissingRoundStart,
    RoundStartMismatch,
    ExtraSeed,
    Runtime,
}

pub(super) fn replay(journal: &Journal) -> Result<RuntimeInspection, ReplayError> {
    validate_header(journal)?;
    validate_order(&journal.records)?;
    let seeds = journal
        .records
        .iter()
        .filter_map(|record| match record.operation {
            Operation::RoundStarted { generation, .. } => Some(Some(generation.seed())),
            _ => None,
        })
        .collect::<VecDeque<_>>();
    let remaining = Arc::new(Mutex::new(seeds));
    let source_remaining = Arc::clone(&remaining);
    let missing_seed = Arc::new(AtomicBool::new(false));
    let source_missing_seed = Arc::clone(&missing_seed);
    let source = RoundSeedSource::with_entropy(move |bytes| {
        let outcome = source_remaining.lock().map_err(|_| ())?.pop_front();
        if outcome.is_none() {
            source_missing_seed.store(true, Ordering::Relaxed);
        }
        let seed = outcome.flatten().ok_or(())?;
        bytes.copy_from_slice(&seed);
        Ok(())
    });
    let mut runtime =
        MatchRuntime::new_without_replay(identity_for_rules(journal.header.rules_version)?, source);
    for seat in SeatId::ALL {
        if journal.header.initial_seats[seat.index()] {
            runtime.occupy(seat).map_err(|_| ReplayError::Runtime)?;
        }
    }
    let mut awaiting_start = None;
    for record in &journal.records {
        if record.tick != runtime.lifecycle().tick() {
            return Err(ReplayError::WrongTick);
        }
        match record.operation {
            Operation::RoundStarted {
                epoch,
                assignment,
                generation,
            } => {
                if awaiting_start != Some(RoundBoundary::Started) {
                    return Err(ReplayError::ExtraSeed);
                }
                let inspection = runtime.replay_inspection();
                let simulation = inspection
                    .simulation
                    .ok_or(ReplayError::RoundStartMismatch)?;
                if simulation.epoch != epoch
                    || simulation.state.round_assignment() != assignment
                    || simulation.state.tick() != record.tick
                    || simulation.generation != Some(generation)
                {
                    return Err(ReplayError::RoundStartMismatch);
                }
                awaiting_start = None;
            }
            operation => {
                if awaiting_start.is_some() {
                    return Err(ReplayError::MissingRoundStart);
                }
                validate_bounds(operation)?;
                let before = runtime.lifecycle().phase();
                apply_profile(&mut runtime, operation, journal.header.lifecycle_profile)?;
                if matches!(operation, Operation::Tick) {
                    if missing_seed.swap(false, Ordering::Relaxed) {
                        return Err(ReplayError::MissingSeed);
                    }
                    awaiting_start = if before != MatchPhase::Running
                        && runtime.lifecycle().phase() == MatchPhase::Running
                    {
                        Some(RoundBoundary::Started)
                    } else {
                        None
                    };
                }
            }
        }
    }
    if awaiting_start.is_some() {
        return Err(ReplayError::MissingRoundStart);
    }
    if !remaining
        .lock()
        .map_err(|_| ReplayError::Runtime)?
        .is_empty()
    {
        return Err(ReplayError::ExtraSeed);
    }
    Ok(runtime.replay_inspection())
}

fn validate_header(journal: &Journal) -> Result<(), ReplayError> {
    match journal.status {
        ReplayStatus::Complete => {}
        ReplayStatus::Truncated { .. } => return Err(ReplayError::Truncated),
        ReplayStatus::Incomplete { .. } => return Err(ReplayError::Incomplete),
        #[cfg(test)]
        ReplayStatus::UnsupportedProfile => return Err(ReplayError::UnsupportedProfile),
    }
    let header = journal.header;
    if header.format_version != FORMAT_VERSION {
        return Err(ReplayError::UnsupportedFormat);
    }
    if !matches!(
        header.lifecycle_profile,
        M5_LIFECYCLE_PROFILE | M6_LIFECYCLE_PROFILE
    ) {
        return Err(ReplayError::UnsupportedLifecycle);
    }
    if !matches!(header.rules_version, CLASSIC_V2_RULES | CLASSIC_V3_RULES)
        || RulesVersion::try_from(header.rules_version).is_err()
    {
        return Err(ReplayError::UnsupportedRules);
    }
    if header.generator_version != GENERATED_V1 || header.generator_version != GENERATOR_VERSION {
        return Err(ReplayError::UnsupportedGenerator);
    }
    if header.initial_tick.0 != 0 {
        return Err(ReplayError::UnsupportedInitialTick);
    }
    Ok(())
}

fn validate_order(records: &[super::record::Record]) -> Result<(), ReplayError> {
    for (index, record) in records.iter().enumerate() {
        let expected = match records.get(index.wrapping_sub(1)) {
            Some(previous) if previous.tick == record.tick => previous
                .order
                .checked_add(1)
                .ok_or(ReplayError::InvalidOrder)?,
            _ => 0,
        };
        if record.order != expected || index > 0 && records[index - 1].tick.0 > record.tick.0 {
            return Err(ReplayError::InvalidOrder);
        }
    }
    Ok(())
}

fn validate_bounds(operation: Operation) -> Result<(), ReplayError> {
    match operation {
        Operation::Action { epoch: 0, .. } => Err(ReplayError::InvalidBounds),
        Operation::Action {
            action: ReplayAction::Bomb(action_id),
            ..
        } if action_id == 0 || action_id > MAX_COUNTER => Err(ReplayError::InvalidBounds),
        _ => Ok(()),
    }
}

pub(super) fn apply(runtime: &mut MatchRuntime, operation: Operation) -> Result<(), ReplayError> {
    if matches!(operation, Operation::Tick) {
        return runtime.tick().map(|_| ()).map_err(|_| ReplayError::Runtime);
    }
    apply_profile(runtime, operation, M6_LIFECYCLE_PROFILE)
}

fn apply_profile(
    runtime: &mut MatchRuntime,
    operation: Operation,
    profile: u16,
) -> Result<(), ReplayError> {
    match operation {
        Operation::Occupy(seat) => runtime.occupy(seat).map(|_| ()),
        Operation::Action {
            seat,
            epoch,
            action,
        } => runtime
            .action(
                seat,
                epoch,
                match action {
                    ReplayAction::Ready(value) => MatchAction::Ready(value),
                    ReplayAction::Movement(value) => MatchAction::Movement(value),
                    ReplayAction::Bomb(value) => MatchAction::Bomb(value),
                    ReplayAction::Rematch(value) => MatchAction::Rematch(value),
                },
            )
            .map(|_| ()),
        Operation::InputFence(seat) => {
            runtime.stop_seat_inputs(seat);
            Ok(())
        }
        Operation::Tick => runtime.replay_tick_profile(profile),
        Operation::Depart(departures) => runtime.depart_batch(departures).map(|_| ()),
        Operation::GraceDepart(departures) => runtime.replay_grace_depart(departures),
        Operation::Resume { seat, kind } => {
            let prepared = runtime
                .prepare_resume(seat)
                .map_err(|_| ReplayError::Runtime)?;
            (runtime.commit_admission(prepared) == kind)
                .then_some(())
                .ok_or(crate::game::rooms::match_runtime::MatchRuntimeError::Publication)
        }
        Operation::Expiry => runtime.replay_expiry(),
        Operation::ReportFlush => runtime.replay_report_flush(),
        Operation::Interrupt(reason) => runtime.interrupt(reason).map(|_| ()),
        Operation::InterruptAndDepart { reason, departures } => {
            runtime.interrupt_and_depart(reason, departures).map(|_| ())
        }
        Operation::RoundStarted { .. } => unreachable!("round starts are validation records"),
    }
    .map_err(|_| ReplayError::Runtime)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RoundBoundary {
    Started,
}

pub(super) fn identity() -> RoomIdentity {
    identity_for_profile(RoomProfile::GeneratedClassicV2)
}

fn identity_for_rules(rules_version: u16) -> Result<RoomIdentity, ReplayError> {
    let profile = match RulesVersion::try_from(rules_version) {
        Ok(RulesVersion::ClassicV2) => RoomProfile::GeneratedClassicV2,
        Ok(RulesVersion::ClassicV3) => RoomProfile::GeneratedClassicV3,
        _ => return Err(ReplayError::UnsupportedRules),
    };
    Ok(identity_for_profile(profile))
}

pub(super) fn identity_for_profile(profile: RoomProfile) -> RoomIdentity {
    RoomIdentity {
        id: RoomId::new("00000000000000000000000000000001").expect("static room id"),
        code: RoomCode::new("ABCDEFGH").expect("static room code"),
        profile,
    }
}

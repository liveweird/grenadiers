use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
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
    CLASSIC_V2_RULES, FORMAT_VERSION, GENERATED_V1, Journal, M5_LIFECYCLE_PROFILE, Operation,
    ReplayAction, ReplayStatus,
};
use crate::round_seed::RoundSeedSource;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReplayError {
    Truncated,
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
            Operation::RoundStarted { generation, .. } => Some(generation.seed()),
            _ => None,
        })
        .collect::<VecDeque<_>>();
    let remaining = Arc::new(Mutex::new(seeds));
    let source_remaining = Arc::clone(&remaining);
    let source = RoundSeedSource::with_entropy(move |bytes| {
        let seed = source_remaining
            .lock()
            .map_err(|_| ())?
            .pop_front()
            .ok_or(())?;
        bytes.copy_from_slice(&seed);
        Ok(())
    });
    let mut runtime = MatchRuntime::new(identity(), source);
    for seat in SeatId::ALL {
        if journal.header.initial_seats[seat.index()] {
            runtime.occupy(seat).map_err(|_| ReplayError::Runtime)?;
        }
    }
    let mut awaiting_start = false;
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
                if !awaiting_start {
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
                awaiting_start = false;
            }
            operation => {
                if awaiting_start {
                    return Err(ReplayError::MissingRoundStart);
                }
                validate_bounds(operation)?;
                let before = runtime.lifecycle().phase();
                apply(&mut runtime, operation)?;
                awaiting_start = before != MatchPhase::Running
                    && runtime.lifecycle().phase() == MatchPhase::Running;
                let phase = runtime.lifecycle().phase();
                let entropy_failed = matches!(
                    phase,
                    MatchPhase::Interrupted {
                        reason: grenadiers_core::room::InterruptReason::InternalError,
                        ..
                    }
                ) || matches!(phase, MatchPhase::MatchResult { outcome, .. }
                if matches!(outcome.result(), grenadiers_core::room::series::MatchResult::Interrupted(
                    grenadiers_core::room::InterruptReason::InternalError
                )));
                if matches!(before, MatchPhase::Countdown { .. }) && entropy_failed {
                    return Err(ReplayError::MissingSeed);
                }
            }
        }
    }
    if awaiting_start {
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
    if journal.status != ReplayStatus::Complete {
        return Err(ReplayError::Truncated);
    }
    let header = journal.header;
    if header.format_version != FORMAT_VERSION {
        return Err(ReplayError::UnsupportedFormat);
    }
    if header.lifecycle_profile != M5_LIFECYCLE_PROFILE {
        return Err(ReplayError::UnsupportedLifecycle);
    }
    if header.rules_version != CLASSIC_V2_RULES
        || RulesVersion::try_from(header.rules_version) != Ok(RulesVersion::ClassicV2)
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
        Operation::Tick => runtime.tick().map(|_| ()),
        Operation::Depart(departures) => runtime.depart_batch(departures).map(|_| ()),
        Operation::Interrupt(reason) => runtime.interrupt(reason).map(|_| ()),
        Operation::RoundStarted { .. } => unreachable!("round starts are validation records"),
    }
    .map_err(|_| ReplayError::Runtime)
}

pub(super) fn identity() -> RoomIdentity {
    RoomIdentity {
        id: RoomId::new("00000000000000000000000000000001").expect("static room id"),
        code: RoomCode::new("ABCDEFGH").expect("static room code"),
        profile: RoomProfile::GeneratedClassicV2,
    }
}

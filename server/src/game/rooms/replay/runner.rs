use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use grenadiers_core::{
    game::{EnabledItems, SeatId},
    protocol::{MAX_COUNTER, RoomCode, RoomId},
    room::MatchPhase,
};

use super::super::{
    match_runtime::{MatchAction, MatchRuntime, RuntimeInspection},
    profile::RoomProfile,
    types::RoomIdentity,
};
use super::record::{
    CLASSIC_V2_RULES, CLASSIC_V3_RULES, CONFIGURABLE_V4_RULES, EXPANDED_V5_RULES, Journal,
    M6_LIFECYCLE_PROFILE, Operation, ReplayAction,
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
    InvalidActionProfile,
    MissingSeed,
    MissingRoundStart,
    RoundStartMismatch,
    ExtraSeed,
    Runtime,
}

pub(super) fn replay(journal: &Journal) -> Result<RuntimeInspection, ReplayError> {
    super::validation::validate_header(journal)?;
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
    let mut runtime = MatchRuntime::new_without_replay(
        identity_for_header(journal.header)?,
        source,
        journal.header.initial_settings,
        journal.header.initial_settings_revision,
        journal.header.initial_settings_editor,
    );
    for seat in SeatId::DUEL {
        if journal.header.initial_seats[seat.index()]
            && !runtime.lifecycle().seats()[seat.index()].occupied()
        {
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
                validate_operation(
                    operation,
                    journal.header.rules_version,
                    runtime.lifecycle().settings().enabled_items(),
                )?;
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

fn validate_operation(
    operation: Operation,
    rules_version: u16,
    effective_items: EnabledItems,
) -> Result<(), ReplayError> {
    if let Operation::Action { action, .. } = operation {
        if matches!(action, ReplayAction::Equipment(_)) && rules_version != EXPANDED_V5_RULES {
            return Err(ReplayError::InvalidActionProfile);
        }
        let valid_for_profile = match rules_version {
            CLASSIC_V2_RULES | CLASSIC_V3_RULES => !matches!(
                action,
                ReplayAction::ReadyChecked { .. } | ReplayAction::UpdateSettings { .. }
            ),
            CONFIGURABLE_V4_RULES => match action {
                ReplayAction::Ready(_) => false,
                ReplayAction::UpdateSettings { settings, .. } => {
                    settings.enabled_items() == EnabledItems::EMPTY
                }
                _ => true,
            },
            EXPANDED_V5_RULES => {
                !matches!(action, ReplayAction::Ready(_))
                    && (!matches!(action, ReplayAction::Equipment(_))
                        || effective_items.contains_punch())
            }
            _ => false,
        };
        if !valid_for_profile {
            return Err(ReplayError::InvalidActionProfile);
        }
    }
    match operation {
        Operation::Action { epoch: 0, .. } => Err(ReplayError::InvalidBounds),
        Operation::Action {
            action: ReplayAction::Bomb(action_id) | ReplayAction::Equipment(action_id),
            ..
        } if action_id == 0 || action_id > MAX_COUNTER => Err(ReplayError::InvalidBounds),
        Operation::Action {
            action:
                ReplayAction::ReadyChecked {
                    settings_revision: 0,
                    ..
                }
                | ReplayAction::UpdateSettings {
                    settings_revision: 0,
                    ..
                },
            ..
        } => Err(ReplayError::InvalidBounds),
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
                    ReplayAction::ReadyChecked {
                        ready,
                        settings_revision,
                    } => MatchAction::ReadyChecked {
                        ready,
                        settings_revision,
                    },
                    ReplayAction::UpdateSettings {
                        settings_revision,
                        settings,
                    } => MatchAction::UpdateSettings {
                        settings_revision,
                        settings,
                    },
                    ReplayAction::Movement(value) => MatchAction::Movement(value),
                    ReplayAction::Bomb(value) => MatchAction::Bomb(value),
                    ReplayAction::Equipment(value) => MatchAction::Equipment(value),
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

fn identity_for_header(header: super::record::Header) -> Result<RoomIdentity, ReplayError> {
    let profile = match header.rules_version {
        CLASSIC_V2_RULES => RoomProfile::GeneratedClassicV2,
        CLASSIC_V3_RULES => RoomProfile::GeneratedClassicV3,
        CONFIGURABLE_V4_RULES => RoomProfile::GeneratedConfigurableV4,
        EXPANDED_V5_RULES => RoomProfile::ExpandedV5,
        _ => return Err(ReplayError::UnsupportedRules),
    };
    let mut identity = identity_for_profile(profile);
    identity.mode = header.mode;
    identity.private_cpu_seed = header.solo.map(|solo| solo.private_seed.0);
    Ok(identity)
}

pub(super) fn identity_for_profile(profile: RoomProfile) -> RoomIdentity {
    RoomIdentity {
        id: RoomId::new("00000000000000000000000000000001").expect("static room id"),
        code: RoomCode::new("ABCDEFGH").expect("static room code"),
        profile,
        mode: grenadiers_core::room::RoomMode::HumanDuel,
        registry_serial: 1,
        private_cpu_seed: None,
    }
}

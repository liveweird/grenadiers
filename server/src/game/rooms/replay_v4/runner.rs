use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use grenadiers_core::protocol::{RoomCode, RoomId};

use crate::{
    game::rooms::roster_runtime::{
        RosterAction, RosterIdentity, RosterRuntime, RosterRuntimeInspection,
    },
    round_seed::RoundSeedSource,
};

use super::profile::ReplayProfile;
use super::record::{ActionSource, Journal, Operation, ReplayAction, ReplayStatus, ReplayV4Header};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) enum ReplayError {
    InvalidHeader,
    Truncated,
    Incomplete,
    InvalidOrder,
    WrongTick,
    InvalidOperation,
    MissingRoundMarker,
    ExtraRoundMarker,
    RoundMarkerMismatch,
    SeedUnavailable,
    Runtime,
}

#[derive(Clone, Copy)]
pub(in crate::game::rooms) struct NormalizedRecord {
    pub(in crate::game::rooms) tick: grenadiers_core::game::Tick,
    pub(in crate::game::rooms) order: u32,
    pub(in crate::game::rooms) operation: NormalizedOperation,
}

#[derive(Clone, Copy)]
pub(in crate::game::rooms) enum NormalizedOperation {
    Legacy(Operation),
    EnvironmentSettings {
        seat: grenadiers_core::game::SeatId,
        epoch: u64,
        settings_revision: u32,
        settings: grenadiers_core::room::RoomSettings,
        environment: grenadiers_core::game::StormCloudConfig,
    },
}

pub(in crate::game::rooms) fn verify_exact(
    journal: &Journal,
    expected: &RosterRuntimeInspection,
) -> Result<(), String> {
    verify_exact_for_profile(ReplayProfile::V4, journal, expected)
}

pub(in crate::game::rooms) fn verify_exact_v5(
    journal: &Journal,
    expected: &RosterRuntimeInspection,
) -> Result<(), String> {
    verify_exact_for_profile(ReplayProfile::V5, journal, expected)
}

fn verify_exact_for_profile(
    profile: ReplayProfile,
    journal: &Journal,
    expected: &RosterRuntimeInspection,
) -> Result<(), String> {
    let actual = replay_for_profile(profile, journal)
        .map_err(|error| format!("replay failed: {error:?}"))?;
    (actual == *expected)
        .then_some(())
        .ok_or_else(|| format!("replay mismatch: actual={actual:?} expected={expected:?}"))
}

pub(in crate::game::rooms) fn replay(
    journal: &Journal,
) -> Result<RosterRuntimeInspection, ReplayError> {
    replay_for_profile(ReplayProfile::V4, journal)
}

pub(in crate::game::rooms) fn replay_v5(
    journal: &Journal,
) -> Result<RosterRuntimeInspection, ReplayError> {
    replay_for_profile(ReplayProfile::V5, journal)
}

fn replay_for_profile(
    profile: ReplayProfile,
    journal: &Journal,
) -> Result<RosterRuntimeInspection, ReplayError> {
    super::validation::validate_header_for_profile(profile, &journal.header)
        .map_err(|()| ReplayError::InvalidHeader)?;
    match journal.status {
        ReplayStatus::Complete => {}
        ReplayStatus::Truncated => return Err(ReplayError::Truncated),
        ReplayStatus::IncompleteRoundConstruction => return Err(ReplayError::Incomplete),
    }
    validate_order(&journal.records)?;
    for record in &journal.records {
        validate_operation(&journal.header, record.operation)?;
    }
    let normalized = journal
        .records
        .iter()
        .map(|record| NormalizedRecord {
            tick: record.tick,
            order: record.order,
            operation: NormalizedOperation::Legacy(record.operation),
        })
        .collect::<Vec<_>>();
    let seeds = journal
        .records
        .iter()
        .filter_map(|record| match record.operation {
            Operation::RoundStarted { generation, .. } => Some(generation.seed()),
            _ => None,
        })
        .collect::<VecDeque<_>>();
    let seeds = Arc::new(Mutex::new(seeds));
    let source_seeds = Arc::clone(&seeds);
    let source = RoundSeedSource::with_entropy(move |bytes| {
        let seed = source_seeds.lock().map_err(|_| ())?.pop_front().ok_or(())?;
        bytes.copy_from_slice(&seed);
        Ok(())
    });
    let runtime = runtime_from_header(&journal.header, source)?;
    execute_normalized(runtime, &normalized, seeds)
}

pub(in crate::game::rooms) fn execute_normalized(
    mut runtime: RosterRuntime,
    records: &[NormalizedRecord],
    seeds: Arc<Mutex<VecDeque<[u8; 32]>>>,
) -> Result<RosterRuntimeInspection, ReplayError> {
    validate_normalized_order(records)?;
    let mut marker_due = false;
    for record in records {
        if record.tick != runtime.replay_inspection().lifecycle.tick() {
            return Err(ReplayError::WrongTick);
        }
        if let NormalizedOperation::Legacy(Operation::RoundStarted {
            epoch,
            assignment,
            generation,
        }) = record.operation
        {
            if !marker_due {
                return Err(ReplayError::ExtraRoundMarker);
            }
            let sim = runtime
                .replay_inspection()
                .simulation
                .ok_or(ReplayError::RoundMarkerMismatch)?;
            if sim.epoch != epoch
                || sim.state.round_assignment() != assignment
                || sim.generation != generation
            {
                return Err(ReplayError::RoundMarkerMismatch);
            }
            marker_due = false;
            continue;
        }
        if marker_due {
            return Err(ReplayError::MissingRoundMarker);
        }
        let before = runtime.replay_inspection().simulation.map(|sim| sim.epoch);
        match record.operation {
            NormalizedOperation::Legacy(operation) => apply(&mut runtime, operation)?,
            NormalizedOperation::EnvironmentSettings {
                seat,
                epoch,
                settings_revision,
                settings,
                environment,
            } => runtime
                .update_environment_settings(seat, epoch, settings_revision, settings, environment)
                .map(|_| ())
                .map_err(|_| ReplayError::Runtime)?,
        }
        marker_due = runtime
            .replay_inspection()
            .simulation
            .is_some_and(|sim| Some(sim.epoch) != before);
    }
    if marker_due {
        return Err(ReplayError::MissingRoundMarker);
    }
    if !seeds.lock().map_err(|_| ReplayError::Runtime)?.is_empty() {
        return Err(ReplayError::ExtraRoundMarker);
    }
    Ok(runtime.replay_inspection())
}

fn validate_normalized_order(records: &[NormalizedRecord]) -> Result<(), ReplayError> {
    for (index, record) in records.iter().enumerate() {
        let expected = match records
            .get(index.wrapping_sub(1))
            .filter(|previous| previous.tick == record.tick)
        {
            Some(previous) => previous
                .order
                .checked_add(1)
                .ok_or(ReplayError::InvalidOrder)?,
            None => 0,
        };
        if record.order != expected || index > 0 && records[index - 1].tick.0 > record.tick.0 {
            return Err(ReplayError::InvalidOrder);
        }
    }
    Ok(())
}

pub(super) fn runtime_from_header(
    header: &ReplayV4Header,
    seeds: RoundSeedSource,
) -> Result<RosterRuntime, ReplayError> {
    let root = header.simulation_root.clone();
    let mut runtime = RosterRuntime::new(
        identity(header.configuration),
        header.initial_settings,
        seeds,
        Arc::new(move |epoch| root.for_round(epoch)),
    )
    .map_err(|_| ReplayError::Runtime)?;
    for &seat in header
        .configuration
        .human_seats()
        .collect::<Vec<_>>()
        .iter()
    {
        if header.initially_occupied_humans[seat.index()] {
            let prepared = runtime
                .prepare_join(seat)
                .map_err(|_| ReplayError::InvalidHeader)?;
            runtime.commit_admission(prepared);
        }
    }
    Ok(runtime)
}

pub(in crate::game::rooms) fn apply(
    runtime: &mut RosterRuntime,
    operation: Operation,
) -> Result<(), ReplayError> {
    match operation {
        Operation::Admit { seat, resume } => {
            let prepared = if resume {
                runtime.prepare_resume(seat)
            } else {
                runtime.prepare_join(seat)
            }
            .map_err(|_| ReplayError::Runtime)?;
            runtime.commit_admission(prepared);
            Ok(())
        }
        Operation::Action {
            seat,
            epoch,
            source,
            action,
        } => match source {
            ActionSource::Human => runtime.action(seat, epoch, domain_action(action)),
            ActionSource::Cpu => runtime.submit_cpu_intent(seat, epoch, domain_action(action)),
        }
        .map(|_| ())
        .map_err(|_| ReplayError::Runtime),
        Operation::InputFence(seat) => {
            runtime.stop_seat_inputs(seat);
            Ok(())
        }
        Operation::Settle(departures) => runtime
            .settle_boundary(departures)
            .map(|_| ())
            .map_err(|_| ReplayError::Runtime),
        Operation::Interrupt(reason) => runtime
            .interrupt(reason)
            .map(|_| ())
            .map_err(|_| ReplayError::Runtime),
        Operation::InterruptAndDepart { reason, departures } => runtime
            .interrupt_and_depart(reason, departures)
            .map(|_| ())
            .map_err(|_| ReplayError::Runtime),
        Operation::PublishFlush => runtime
            .current_notice()
            .map(|_| ())
            .map_err(|_| ReplayError::Runtime),
        Operation::RoundStarted { .. } => Err(ReplayError::InvalidOperation),
    }
}

fn validate_order(records: &[super::record::Record]) -> Result<(), ReplayError> {
    for (index, record) in records.iter().enumerate() {
        let expected = match records
            .get(index.wrapping_sub(1))
            .filter(|previous| previous.tick == record.tick)
        {
            Some(previous) => previous
                .order
                .checked_add(1)
                .ok_or(ReplayError::InvalidOrder)?,
            None => 0,
        };
        if record.order != expected || index > 0 && records[index - 1].tick.0 > record.tick.0 {
            return Err(ReplayError::InvalidOrder);
        }
    }
    Ok(())
}

fn validate_operation(header: &ReplayV4Header, operation: Operation) -> Result<(), ReplayError> {
    let Operation::Action {
        seat,
        epoch,
        source,
        action,
    } = operation
    else {
        return Ok(());
    };
    if epoch == 0 {
        return Err(ReplayError::InvalidOperation);
    }
    let expected = match header.configuration.controller(seat) {
        Some(grenadiers_core::room::SeatControl::Human) => ActionSource::Human,
        Some(grenadiers_core::room::SeatControl::Cpu) => ActionSource::Cpu,
        None => return Err(ReplayError::InvalidOperation),
    };
    if source != expected
        || source == ActionSource::Cpu
            && !matches!(
                action,
                ReplayAction::Movement(_) | ReplayAction::Bomb(_) | ReplayAction::Equipment(_)
            )
    {
        return Err(ReplayError::InvalidOperation);
    }
    if matches!(action, ReplayAction::Bomb(0) | ReplayAction::Equipment(0))
        || matches!(action, ReplayAction::Bomb(id) | ReplayAction::Equipment(id) if id > grenadiers_core::protocol::MAX_COUNTER)
        || matches!(
            action,
            ReplayAction::Ready {
                settings_revision: 0,
                ..
            } | ReplayAction::UpdateSettings {
                settings_revision: 0,
                ..
            }
        )
    {
        return Err(ReplayError::InvalidOperation);
    }
    Ok(())
}

pub(in crate::game::rooms) fn domain_action(action: ReplayAction) -> RosterAction {
    match action {
        ReplayAction::Ready {
            ready,
            settings_revision,
        } => RosterAction::Ready {
            ready,
            settings_revision,
        },
        ReplayAction::UpdateSettings {
            settings_revision,
            settings,
        } => RosterAction::UpdateSettings {
            settings_revision,
            settings,
        },
        ReplayAction::Movement(direction) => RosterAction::Movement(direction),
        ReplayAction::Bomb(id) => RosterAction::Bomb(id),
        ReplayAction::Equipment(id) => RosterAction::Equipment(id),
        ReplayAction::Rematch(ready) => RosterAction::Rematch(ready),
    }
}

fn identity(configuration: grenadiers_core::room::RoomConfiguration) -> RosterIdentity {
    RosterIdentity {
        id: RoomId::new("00000000000000000000000000000004").unwrap(),
        code: RoomCode::new("REPLAYV4").unwrap(),
        registry_serial: 4,
        configuration,
    }
}

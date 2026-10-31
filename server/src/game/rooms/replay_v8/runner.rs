use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use grenadiers_core::protocol::{RoomCode, RoomId};

use crate::{
    game::rooms::{
        replay_v4::{
            Operation as LegacyOperation, ReplayAction as LegacyAction,
            runner::{ReplayError, apply as apply_legacy},
        },
        roster_runtime::{RosterAction, RosterIdentity, RosterRuntime, RosterRuntimeInspection},
    },
    round_seed::RoundSeedSource,
};

use super::record::{ActionSource, Journal, Operation, ReplayAction, ReplayStatus};

pub(in crate::game::rooms) fn verify_exact(
    journal: &Journal,
    expected: &RosterRuntimeInspection,
) -> Result<(), String> {
    let actual = replay(journal).map_err(|error| format!("replay failed: {error:?}"))?;
    (actual == *expected)
        .then_some(())
        .ok_or_else(|| format!("replay mismatch: actual={actual:?} expected={expected:?}"))
}

pub(in crate::game::rooms) fn replay(
    journal: &Journal,
) -> Result<RosterRuntimeInspection, ReplayError> {
    replay_with_mob_navigation(
        journal,
        grenadiers_core::game::MobNavigationPolicy::LegacyV1,
    )
}

pub(in crate::game::rooms) fn replay_with_mob_navigation(
    journal: &Journal,
    navigation: grenadiers_core::game::MobNavigationPolicy,
) -> Result<RosterRuntimeInspection, ReplayError> {
    super::validation::validate_header(&journal.header).map_err(|()| ReplayError::InvalidHeader)?;
    match journal.status {
        ReplayStatus::Complete => {}
        ReplayStatus::Truncated => return Err(ReplayError::Truncated),
        ReplayStatus::IncompleteRoundConstruction => return Err(ReplayError::Incomplete),
    }
    for record in &journal.records {
        super::validation::validate_operation(&journal.header, record.operation)
            .map_err(|()| ReplayError::InvalidOperation)?;
    }
    let (source, seeds) = generation_seed_context(journal);
    let runtime = runtime_from_header_with_navigation(&journal.header, source, navigation)?;
    execute_journal(runtime, journal, seeds)
}

pub(in crate::game::rooms) type SeedQueue = Arc<Mutex<VecDeque<[u8; 32]>>>;

pub(in crate::game::rooms) fn generation_seed_context(
    journal: &Journal,
) -> (RoundSeedSource, SeedQueue) {
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
    (source, seeds)
}

pub(in crate::game::rooms) fn execute_journal(
    mut runtime: RosterRuntime,
    journal: &Journal,
    seeds: SeedQueue,
) -> Result<RosterRuntimeInspection, ReplayError> {
    validate_order(&journal.records)?;
    let mut marker_due = false;
    for record in &journal.records {
        if record.tick != runtime.replay_inspection().lifecycle.tick() {
            return Err(ReplayError::WrongTick);
        }
        if let Operation::RoundStarted {
            epoch,
            assignment,
            generation,
        } = record.operation
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
        apply_operation(&mut runtime, record.operation)?;
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

fn apply_operation(runtime: &mut RosterRuntime, operation: Operation) -> Result<(), ReplayError> {
    match operation {
        Operation::Action {
            seat,
            epoch,
            source: ActionSource::Human,
            action:
                ReplayAction::UpdateSettings {
                    settings_revision,
                    settings,
                    cloud,
                    tuning,
                },
        } => runtime
            .update_respawn_settings(seat, epoch, settings_revision, settings, cloud, tuning)
            .map(|_| ())
            .map_err(|_| ReplayError::Runtime),
        Operation::Action {
            seat,
            epoch,
            source: ActionSource::Human,
            action:
                ReplayAction::UpdateSpikeSettings {
                    settings_revision,
                    settings,
                    cloud,
                    tuning,
                    spike_pits,
                },
        } => runtime
            .update_spike_settings(
                seat,
                epoch,
                settings_revision,
                settings,
                cloud,
                tuning,
                spike_pits,
            )
            .map(|_| ())
            .map_err(|_| ReplayError::Runtime),
        Operation::Action {
            source: ActionSource::Cpu,
            action: ReplayAction::UpdateSettings { .. } | ReplayAction::UpdateSpikeSettings { .. },
            ..
        } => Err(ReplayError::InvalidOperation),
        operation => apply_legacy(runtime, legacy_operation(operation)?),
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

fn legacy_operation(operation: Operation) -> Result<LegacyOperation, ReplayError> {
    Ok(match operation {
        Operation::Admit { seat, resume } => LegacyOperation::Admit { seat, resume },
        Operation::Action {
            seat,
            epoch,
            source,
            action,
        } => LegacyOperation::Action {
            seat,
            epoch,
            source: match source {
                ActionSource::Human => super::super::replay_v4::record::ActionSource::Human,
                ActionSource::Cpu => super::super::replay_v4::record::ActionSource::Cpu,
            },
            action: legacy_action(action)?,
        },
        Operation::InputFence(seat) => LegacyOperation::InputFence(seat),
        Operation::Settle(value) => LegacyOperation::Settle(value),
        Operation::Interrupt(value) => LegacyOperation::Interrupt(value),
        Operation::InterruptAndDepart { reason, departures } => {
            LegacyOperation::InterruptAndDepart { reason, departures }
        }
        Operation::PublishFlush => LegacyOperation::PublishFlush,
        Operation::RoundStarted {
            epoch,
            assignment,
            generation,
        } => LegacyOperation::RoundStarted {
            epoch,
            assignment,
            generation,
        },
    })
}

fn legacy_action(action: ReplayAction) -> Result<LegacyAction, ReplayError> {
    Ok(match action {
        ReplayAction::Ready {
            ready,
            settings_revision,
        } => LegacyAction::Ready {
            ready,
            settings_revision,
        },
        ReplayAction::Movement(value) => LegacyAction::Movement(value),
        ReplayAction::Bomb(value) => LegacyAction::Bomb(value),
        ReplayAction::Equipment(value) => LegacyAction::Equipment(value),
        ReplayAction::Rematch(value) => LegacyAction::Rematch(value),
        ReplayAction::UpdateSettings { .. } => return Err(ReplayError::InvalidOperation),
        ReplayAction::UpdateSpikeSettings { .. } => return Err(ReplayError::InvalidOperation),
    })
}

pub(super) fn runtime_from_header(
    header: &super::ReplayHeader,
    seeds: RoundSeedSource,
) -> Result<RosterRuntime, ReplayError> {
    runtime_from_header_with_navigation(
        header,
        seeds,
        grenadiers_core::game::MobNavigationPolicy::LegacyV1,
    )
}

pub(super) fn runtime_from_header_with_navigation(
    header: &super::ReplayHeader,
    seeds: RoundSeedSource,
    navigation: grenadiers_core::game::MobNavigationPolicy,
) -> Result<RosterRuntime, ReplayError> {
    let root = header.simulation_root.clone();
    let base = *header.configuration.base();
    let tuning = grenadiers_core::room::RespawnLairTuning::try_from(header.configuration.lairs())
        .map_err(|_| ReplayError::InvalidHeader)?;
    let mut runtime = RosterRuntime::new_respawn_with_navigation(
        RosterIdentity {
            id: RoomId::new("00000000000000000000000000000008").unwrap(),
            code: RoomCode::new("REPLAYV8").unwrap(),
            registry_serial: 8,
            configuration: base,
        },
        header.initial_settings,
        header.configuration.storm_cloud(),
        tuning,
        seeds,
        Arc::new(move |epoch| root.for_round(epoch)),
        navigation,
    )
    .map_err(|_| ReplayError::Runtime)?;
    for seat in base.human_seats() {
        if header.initially_occupied_humans[seat.index()] {
            let prepared = runtime
                .prepare_join(seat)
                .map_err(|_| ReplayError::InvalidHeader)?;
            runtime.commit_admission(prepared);
        }
    }
    Ok(runtime)
}

pub(super) fn domain_action(action: ReplayAction) -> Result<RosterAction, ReplayError> {
    legacy_action(action).map(super::super::replay_v4::runner::domain_action)
}

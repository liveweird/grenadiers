use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use grenadiers_core::protocol::{RoomCode, RoomId};

use crate::{
    game::rooms::{
        replay_v4::{
            Operation as LegacyOperation, ReplayAction as LegacyAction,
            normalized_settings::NormalizedSettings,
            runner::{NormalizedOperation, NormalizedRecord, ReplayError, execute_normalized},
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
    let normalized = journal
        .records
        .iter()
        .map(normalize)
        .collect::<Result<Vec<_>, _>>()?;
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

fn normalize(record: &super::record::Record) -> Result<NormalizedRecord, ReplayError> {
    let operation = match record.operation {
        Operation::Action {
            seat,
            epoch,
            source: ActionSource::Human,
            action:
                ReplayAction::UpdateSettings {
                    settings_revision,
                    settings,
                    environment,
                },
        } => NormalizedOperation::Settings(NormalizedSettings::StormV7 {
            seat,
            epoch,
            revision: settings_revision,
            settings,
            cloud: environment,
        }),
        Operation::Action {
            seat: _,
            epoch: _,
            source: ActionSource::Cpu,
            action: ReplayAction::UpdateSettings { .. },
        } => {
            return Err(ReplayError::InvalidOperation);
        }
        operation => NormalizedOperation::Legacy(legacy_operation(operation)?),
    };
    Ok(NormalizedRecord {
        tick: record.tick,
        order: record.order,
        operation,
    })
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
    })
}

pub(super) fn runtime_from_header(
    header: &super::ReplayHeader,
    seeds: RoundSeedSource,
) -> Result<RosterRuntime, ReplayError> {
    let root = header.simulation_root.clone();
    let base = *header.configuration.base();
    let mut runtime = RosterRuntime::new_environment(
        RosterIdentity {
            id: RoomId::new("00000000000000000000000000000006").unwrap(),
            code: RoomCode::new("REPLAYV6").unwrap(),
            registry_serial: 6,
            configuration: base,
        },
        header.initial_settings,
        header.configuration.storm_cloud(),
        seeds,
        Arc::new(move |epoch| root.for_round(epoch)),
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

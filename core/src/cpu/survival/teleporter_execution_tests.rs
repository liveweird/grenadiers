use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, forecast};
use crate::cpu::lair_contact::v139_teleporter_fixture::captured_tick_6121;
use crate::cpu::policy::UtilityMemory;
use crate::cpu::routing::{RouteFailure, TeleporterStepRequest, certify_teleporter_step};
use crate::cpu::{
    DecisionId, MemoryKey, MovementIntent, Observation, ObservedActor, ObservedEnvironment,
    WorkBudget,
};
use crate::game::{Direction, StormCloudConfig, Tick};

use super::{
    PreparedTeleporterStep, TELEPORTER_ADMISSION_ALLOWANCE, TeleporterStepPlan,
    TeleporterStepStatus,
};

const LIMIT: u32 = 65_536;

#[test]
fn all_seven_start_ticks_admit_and_latest_boundary_is_exact() {
    for offset in 0..=6 {
        let (mut prepared, _, _) = prepare(offset, Tick(6_111), TELEPORTER_ADMISSION_ALLOWANCE);
        assert_eq!(prepared.hold_ticks(), 1);
        assert_eq!(prepared.manual(), None);
        assert_eq!(
            prepared.command().unwrap().admission_tick(),
            Tick(6_121 + offset)
        );
    }
    let basis = basis();
    let step = step(&basis, Tick(6_111)).unwrap();
    let current = at(&basis, 7);
    let result = PreparedTeleporterStep::admit(
        plan(step),
        basis.clone(),
        &current,
        Tick(6_111),
        decision(&basis),
        current.parts().tick,
        &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
    );
    assert!(result.is_err());
}

#[test]
fn retained_old_stop_is_preserved_but_future_or_replaced_stop_rejects() {
    let basis = basis();
    assert!(step(&basis, Tick(6_111)).is_ok());
    assert_eq!(
        step(&basis, Tick(6_122)).unwrap_err(),
        RouteFailure::Unsupported
    );

    let step = step(&basis, Tick(6_111)).unwrap();
    let current = at(&basis, 0);
    assert!(
        PreparedTeleporterStep::admit(
            plan(step),
            basis.clone(),
            &current,
            Tick(6_120),
            decision(&basis),
            current.parts().tick,
            &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
        )
        .is_err()
    );
}

#[test]
fn movement_commit_requires_own_mailbox_command_then_exact_stop_record() {
    let (mut first, basis, memory) = prepare(0, Tick(6_111), TELEPORTER_ADMISSION_ALLOWANCE);
    let (mut second, _, _) = prepare(0, Tick(6_111), TELEPORTER_ADMISSION_ALLOWANCE);
    let own = first.command().unwrap();
    let foreign = second.command().unwrap();
    assert!(first.commit(foreign).is_err());

    let (mut prepared, _, _) = prepare(0, Tick(6_111), TELEPORTER_ADMISSION_ALLOWANCE);
    let movement = prepared.command().unwrap();
    let mut committed = prepared.commit(movement).unwrap();
    assert_eq!(committed.next_memory(), None);
    let key = memory.key();
    assert!(matches!(
        committed.poll_command(Tick(6_121), key, 7),
        TeleporterStepStatus::Moving
    ));
    let TeleporterStepStatus::Submit(stop) = committed.poll_command(Tick(6_122), key, 7) else {
        panic!("the sole post-move command must be Stop")
    };
    assert_eq!(stop.movement(), MovementIntent::Stop);
    assert!(committed.record_stop(stop));

    let mut stopped = after_move(&basis, 2, Direction::West);
    stopped = stopped
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = &mut parts.actors[2] else {
                unreachable!()
            };
            // A blocked/no-move outcome remains inside the certificate.
            actor.position = match &basis.parts().actors[2] {
                ObservedActor::Visible(actor) => actor.position,
                ObservedActor::Hidden { .. } => unreachable!(),
            };
        })
        .unwrap();
    assert!(matches!(
        committed.observe_stopped(
            &stopped,
            Tick(6_123),
            &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
        ),
        TeleporterStepStatus::Completed(_)
    ));
    assert_eq!(committed.next_memory(), Some(memory));

    let _ = own;
}

#[test]
fn stopped_fence_accepts_partial_consumption_and_actual_swap_outcomes() {
    for partner_index in [None, Some(0), Some(1), Some(3)] {
        let swapped = partner_index.is_some();
        let (mut prepared, basis, memory) = if swapped {
            let basis = collect_basis();
            let memory = UtilityMemory::new(key(&basis));
            let prepared = PreparedTeleporterStep::admit(
                TeleporterStepPlan::new_for_test(
                    step_direction(&basis, Tick(6_111), Direction::North).unwrap(),
                    memory,
                ),
                basis.clone(),
                &basis,
                Tick(6_111),
                decision(&basis),
                basis.parts().tick,
                &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
            )
            .unwrap();
            (prepared, basis, memory)
        } else {
            prepare(0, Tick(6_111), TELEPORTER_ADMISSION_ALLOWANCE)
        };
        let movement = prepared.command().unwrap();
        let mut committed = prepared.commit(movement).unwrap();
        let TeleporterStepStatus::Submit(stop) =
            committed.poll_command(Tick(6_122), memory.key(), 7)
        else {
            unreachable!()
        };
        assert!(committed.record_stop(stop));
        let stopped = after_move(
            &basis,
            4,
            if swapped {
                Direction::North
            } else {
                Direction::West
            },
        )
        .map_parts(|parts| {
            parts
                .pickups
                .retain(|pickup| pickup.cell != crate::game::CellId::new(3, 9));
            let movement_tick = Tick(6_122);
            let root_start = match &basis.parts().actors[2] {
                ObservedActor::Visible(actor) => actor.position,
                ObservedActor::Hidden { .. } => unreachable!(),
            };
            let partner_start = match &basis.parts().actors[partner_index.unwrap_or(0)] {
                ObservedActor::Visible(actor) => actor.position,
                ObservedActor::Hidden { .. } => unreachable!(),
            };
            let ObservedActor::Visible(root) = &mut parts.actors[2] else {
                unreachable!()
            };
            root.position.x -= 24;
            if let Some(partner_index) = partner_index {
                root.position = crate::game::Position {
                    x: partner_start.x + 16,
                    y: partner_start.y,
                };
                root.last_teleport_tick = Some(movement_tick);
                let ObservedActor::Visible(partner) = &mut parts.actors[partner_index] else {
                    unreachable!()
                };
                partner.position = crate::game::Position {
                    // The swap resets it at the collecting endpoint, then its
                    // retained held input moves again before publication.
                    x: root_start.x + 100,
                    y: root_start.y - 51,
                };
                partner.last_teleport_tick = Some(movement_tick);
            }
        })
        .unwrap();
        let status = committed.observe_stopped(
            &stopped,
            Tick(6_123),
            &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
        );
        assert!(matches!(
            status,
            TeleporterStepStatus::Completed(outcome)
                if outcome.swapped == swapped && outcome.consumed
        ));
        assert_eq!(committed.next_memory(), Some(memory));
        // A later observation cannot mint another movement command.
        assert!(matches!(
            committed.poll_command(Tick(6_139), memory.key(), 7),
            TeleporterStepStatus::Completed(_)
        ));
    }
}

#[test]
fn missing_stop_record_epoch_change_and_observation_expiry_invalidate() {
    let (mut prepared, _, memory) = prepare(0, Tick(6_111), TELEPORTER_ADMISSION_ALLOWANCE);
    let command = prepared.command().unwrap();
    let mut committed = prepared.commit(command).unwrap();
    assert!(matches!(
        committed.poll_command(Tick(6_122), memory.key(), 8),
        TeleporterStepStatus::Invalidated
    ));

    let (mut prepared, _, memory) = prepare(0, Tick(6_111), TELEPORTER_ADMISSION_ALLOWANCE);
    let command = prepared.command().unwrap();
    let mut committed = prepared.commit(command).unwrap();
    let _stop = committed.poll_command(Tick(6_122), memory.key(), 7);
    assert!(matches!(
        committed.poll_command(Tick(6_123), memory.key(), 7),
        TeleporterStepStatus::Invalidated
    ));

    let (mut prepared, _, memory) = prepare(0, Tick(6_111), TELEPORTER_ADMISSION_ALLOWANCE);
    let command = prepared.command().unwrap();
    let mut committed = prepared.commit(command).unwrap();
    let TeleporterStepStatus::Submit(stop) = committed.poll_command(Tick(6_122), memory.key(), 7)
    else {
        unreachable!()
    };
    assert!(committed.record_stop(stop));
    assert!(matches!(
        committed.poll_command(Tick(6_139), memory.key(), 7),
        TeleporterStepStatus::Invalidated
    ));
}

#[test]
fn admission_has_exact_and_one_short_shared_budget() {
    let basis = basis();
    let current = at(&basis, 0);
    let mut measured = WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE);
    PreparedTeleporterStep::admit(
        plan(step(&basis, Tick(6_111)).unwrap()),
        basis.clone(),
        &current,
        Tick(6_111),
        decision(&basis),
        current.parts().tick,
        &mut measured,
    )
    .unwrap();
    let required = measured.spent();
    assert!((1..=TELEPORTER_ADMISSION_ALLOWANCE).contains(&required));
    for (limit, accepted) in [(required, true), (required - 1, false)] {
        let result = PreparedTeleporterStep::admit(
            plan(step(&basis, Tick(6_111)).unwrap()),
            basis.clone(),
            &current,
            Tick(6_111),
            decision(&basis),
            current.parts().tick,
            &mut WorkBudget::new(limit),
        );
        assert_eq!(result.is_ok(), accepted);
    }
}

fn prepare(
    offset: u64,
    settled: Tick,
    allowance: u32,
) -> (PreparedTeleporterStep, Observation, UtilityMemory) {
    let basis = basis();
    let current = at(&basis, offset);
    let memory = UtilityMemory::new(key(&basis));
    let prepared = PreparedTeleporterStep::admit(
        TeleporterStepPlan::new_for_test(step(&basis, settled).unwrap(), memory),
        basis.clone(),
        &current,
        settled,
        decision(&basis),
        current.parts().tick,
        &mut WorkBudget::new(allowance),
    )
    .unwrap();
    (prepared, basis, memory)
}

pub(super) fn basis() -> Observation {
    let observed = captured_tick_6121();
    Observation::new_rules_v10(
        observed.clone().into_parts(),
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0); 4]),
        None,
        None,
    )
    .unwrap()
}

fn collect_basis() -> Observation {
    basis()
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = &mut parts.actors[2] else {
                unreachable!()
            };
            actor.position = crate::game::Position {
                x: 3_584,
                y: 10_250,
            };
            actor.heading = Direction::North;
        })
        .unwrap()
}

pub(super) fn at(basis: &Observation, elapsed: u64) -> Observation {
    basis
        .clone()
        .map_parts(|parts| {
            parts.tick = Tick(parts.tick.0 + elapsed);
            parts.rules.running_updates += elapsed as u16;
            parts.rules.remaining_ticks -= elapsed as u16;
        })
        .unwrap()
}

pub(super) fn after_move(basis: &Observation, elapsed: u64, direction: Direction) -> Observation {
    at(basis, elapsed)
        .map_parts(|parts| {
            let ObservedActor::Visible(root) = &mut parts.actors[2] else {
                unreachable!()
            };
            root.heading = direction;
        })
        .unwrap()
}

pub(super) fn step(
    basis: &Observation,
    settled: Tick,
) -> Result<crate::cpu::routing::TeleporterStep, RouteFailure> {
    step_direction(basis, settled, Direction::West)
}

pub(super) fn step_direction(
    basis: &Observation,
    settled: Tick,
    direction: Direction,
) -> Result<crate::cpu::routing::TeleporterStep, RouteFailure> {
    let mut memory = ObservedHazardMemory::new(key(basis));
    memory.observe(basis).unwrap();
    let forecast = forecast(
        basis,
        &memory,
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(LIMIT),
    )
    .unwrap();
    certify_teleporter_step(
        basis,
        &forecast,
        TeleporterStepRequest {
            target: crate::game::CellId::new(3, 9),
            movement: MovementIntent::Direction(direction),
            settled_stop_tick: settled,
        },
        &mut WorkBudget::new(LIMIT),
    )
}

pub(super) fn plan(step: crate::cpu::routing::TeleporterStep) -> TeleporterStepPlan {
    let memory_key = key(step.basis_observation());
    TeleporterStepPlan::new_for_test(step, UtilityMemory::new(memory_key))
}

pub(super) fn key(observation: &Observation) -> MemoryKey {
    MemoryKey {
        world: observation.parts().world,
        cpu_seat: observation.parts().recipient,
    }
}

pub(super) fn decision(observation: &Observation) -> DecisionId {
    DecisionId {
        world: observation.parts().world,
        command_epoch: 7,
        observation_tick: observation.parts().tick,
        checked_request_serial: 9,
    }
}

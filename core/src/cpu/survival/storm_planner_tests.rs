use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{
    ControlPlan, UtilityMemory,
    tests::{key, observation},
};
use crate::cpu::{
    CpuRng, Intent, ManualIntent, MovementIntent, Observation, ObservedActor, ObservedEnvironment,
    ObservedFlame, WorkBudget,
};
use crate::game::{CellId, Direction, Position, StormCloudConfig, StormCloudView};

use super::{ExecutionCertificate, WORK_ALLOWANCE, plan};

fn single_observation() -> Observation {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    parts.rules.version = 7;
    Observation::new_rules_v7(
        parts,
        ObservedEnvironment::new(
            StormCloudConfig::Single,
            Some(StormCloudView {
                position: Position::center(CellId::new(13, 11)),
                direction: Direction::West,
                leg_remaining_ticks: 0,
            }),
            vec![None, Some(0)],
        ),
    )
    .unwrap()
}

fn planned(observation: &Observation, allowance: u32) -> (super::SurvivalPlan, WorkBudget) {
    let mut work = WorkBudget::new(allowance);
    let planned = plan(
        observation,
        &UtilityMemory::new(key()),
        &ObservedHazardMemory::new(key()),
        observation.parts().tick,
        &mut CpuRng::from_seed(key(), [8; 32]),
        &mut work,
    )
    .unwrap();
    (planned, work)
}

#[test]
fn clear_single_retains_manual_free_ordinary_pickup_navigation() {
    let mut parts = single_observation().into_parts();
    parts.pickups.push(crate::cpu::ObservedPickup {
        cell: CellId::new(8, 7),
        kind: crate::game::PickupKind::BombCapacity,
    });
    let observed = Observation::new_rules_v7(
        parts,
        ObservedEnvironment::new(
            StormCloudConfig::Single,
            Some(StormCloudView {
                position: Position::center(CellId::new(7, 6)),
                direction: Direction::East,
                leg_remaining_ticks: 128,
            }),
            vec![None, Some(0)],
        ),
    )
    .unwrap();

    let (planned, _) = planned(&observed, WORK_ALLOWANCE);

    assert_eq!(
        planned.control.intent.movement,
        MovementIntent::Direction(Direction::East)
    );
    assert!(planned.control.intent.manual.is_none());
    assert!(planned.lease.is_none());
    assert!(planned.execution.is_none());
}

#[test]
fn endangered_single_keeps_the_route_certificate_priority() {
    let mut parts = single_observation().into_parts();
    parts.flames.push(ObservedFlame {
        cell: CellId::new(1, 1),
        remaining_ticks: 20,
    });
    let environment = single_observation().environment().unwrap().clone();
    let observed = Observation::new_rules_v7(parts, environment).unwrap();

    let (planned, _) = planned(&observed, WORK_ALLOWANCE);

    assert!(matches!(
        planned.execution,
        Some(ExecutionCertificate::Route(_))
    ));
    assert!(planned.lease.is_none());
}

#[test]
fn exhausted_optional_plant_falls_through_without_manual_input() {
    let mut parts = single_observation().into_parts();
    parts.arena.blocks.push(CellId::new(8, 7));
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.heading = Direction::East;
    let environment = single_observation().environment().unwrap().clone();
    let observed = Observation::new_rules_v7(parts, environment).unwrap();

    let initial = Position::center(CellId::new(7, 7));
    let (planned, work, allowance) = [512, 1_024, 2_048, 4_096, 8_192, 16_384, 32_768]
        .into_iter()
        .find_map(|allowance| {
            let (planned, work) = planned(&observed, allowance);
            (planned.control.intent.manual.is_none()
                && planned.lease.as_ref().is_some_and(|lease| {
                    let end = lease.pose_after(lease.hold_ticks());
                    end.min() != initial || end.max() != initial
                }))
            .then_some((planned, work, allowance))
        })
        .expect("a bounded failed plant must leave work for a progressing certified pulse");

    assert!(planned.control.intent.manual.is_none());
    assert_ne!(
        planned.lease.as_ref().unwrap().movement(),
        MovementIntent::Direction(Direction::East),
        "the blocked proposal is not counted as the progressing pulse"
    );
    assert!(planned.execution.is_none());
    assert!(
        planned.control.next_memory.planted_origin().is_none(),
        "failed Bomb proposals cannot retain a nonexistent planted origin"
    );
    assert!(
        planned
            .control
            .next_memory
            .route()
            .iter()
            .all(Option::is_none),
        "failed Bomb proposals cannot retain the uncommitted escape route"
    );
    assert!(work.spent() <= allowance);
}

#[test]
fn single_certifies_an_actual_plant_escape_when_the_bounded_proof_succeeds() {
    let mut parts = single_observation().into_parts();
    parts.arena.blocks.push(CellId::new(8, 7));
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.heading = Direction::East;
    let environment = single_observation().environment().unwrap().clone();
    let observed = Observation::new_rules_v7(parts, environment).unwrap();

    let (planned, _) = planned(&observed, WORK_ALLOWANCE);

    assert_eq!(planned.control.intent.manual, Some(ManualIntent::Bomb));
    assert!(matches!(
        planned.execution,
        Some(ExecutionCertificate::Plant(_))
    ));
}

#[test]
fn failed_bounded_plant_charges_child_work_and_reserves_half() {
    let mut parts = single_observation().into_parts();
    parts.arena.blocks.push(CellId::new(8, 7));
    let environment = single_observation().environment().unwrap().clone();
    let observed = Observation::new_rules_v7(parts, environment).unwrap();
    let memory = UtilityMemory::new(key());
    let ordinary = ControlPlan {
        intent: Intent::STOP.with_manual(ManualIntent::Bomb),
        hold_ticks: 1,
        next_memory: memory,
    };
    let mut work = WorkBudget::new(6);

    let result = super::plant_tactic::certified_plant(
        &observed,
        &ObservedHazardMemory::new(key()),
        observed.parts().tick,
        &memory,
        &ordinary,
        true,
        &mut work,
    );

    assert!(result.is_none());
    assert_eq!(work.spent(), 3, "failed child work is globally charged");
    assert_eq!(work.remaining(), 3, "half remains for certified survival");
}

#[test]
fn ambiguous_storm_motion_never_authorizes_manual_input() {
    let parts = single_observation().into_parts();
    let actor_position = Position::center(CellId::new(7, 7));
    let observed = Observation::new_rules_v7(
        parts,
        ObservedEnvironment::new(
            StormCloudConfig::Single,
            Some(StormCloudView {
                position: actor_position,
                direction: Direction::North,
                leg_remaining_ticks: 0,
            }),
            vec![None, Some(2)],
        ),
    )
    .unwrap();

    let (planned, _) = planned(&observed, WORK_ALLOWANCE);

    assert!(planned.control.intent.manual.is_none());
}

#[test]
fn disabled_profile_keeps_the_legacy_clear_board_fast_path() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    parts.rules.version = 7;
    parts.pickups.push(crate::cpu::ObservedPickup {
        cell: CellId::new(8, 7),
        kind: crate::game::PickupKind::BombCapacity,
    });
    let observed = Observation::new_rules_v7(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![None, Some(0)]),
    )
    .unwrap();

    let (planned, _) = planned(&observed, WORK_ALLOWANCE);

    assert_eq!(
        planned.control.intent.movement,
        MovementIntent::Direction(Direction::East)
    );
    assert!(planned.lease.is_none());
    assert!(planned.execution.is_none());
}

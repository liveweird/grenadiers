use crate::cpu::routing::{PlantRequest, RouteLimits, plan_lair_ordinary_plant_to_refuge};
use crate::cpu::{
    BombKnowledge, CpuRng, MemoryKey, MobContactLease, MovementIntent, ObservedActor,
    ObservedFlame, WorkBudget,
};
use crate::game::{CellId, Direction, EntityId, Position, StormCloudView, Tick};

use super::foreign_bomb_farming_clear_fixture::{FARM_TARGET, SETTLED_STOP, captured_clear_basis};
use super::{
    ExecutionCertificate, WORK_ALLOWANCE, plan_corner_refuges_v56_with_origin,
    plan_hard_corner_refuges_v57_with_origin, plan_hard_pit_avoidance_v53_with_origin,
    plant_value::useful_plant_for,
};
use crate::cpu::policy::StrategicTarget;

#[test]
fn clear_tick_432_preserves_the_actual_public_scene_and_empty_history() {
    let captured = captured_clear_basis();
    let parts = captured.observation.parts();
    assert_eq!(parts.tick, SETTLED_STOP);
    assert_eq!(parts.rules.running_updates, 252);
    assert_eq!(parts.rules.remaining_ticks, 10_548);
    assert_eq!(parts.arena.blocks.len(), 46);
    assert!(!parts.arena.blocks.contains(&CellId::new(1, 8)));
    assert!(!parts.arena.blocks.contains(&CellId::new(13, 8)));
    let BombKnowledge::Visible(bombs) = &parts.bombs else {
        unreachable!()
    };
    assert!(bombs.is_empty());
    assert!(parts.flames.is_empty());
    let actors = parts
        .actors
        .iter()
        .map(|actor| match actor {
            ObservedActor::Visible(actor) => (
                actor.id,
                actor.position,
                actor.heading,
                actor.live_bomb_count,
            ),
            ObservedActor::Hidden { .. } => unreachable!(),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actors[1],
        (
            EntityId(2),
            Position {
                x: 11_327,
                y: 1_536
            },
            Direction::East,
            0
        )
    );
    assert_eq!(
        actors[2],
        (
            EntityId(3),
            Position {
                x: 3_479,
                y: 11_776
            },
            Direction::East,
            0
        )
    );
    assert_eq!(
        actors[3],
        (
            EntityId(4),
            Position {
                x: 11_881,
                y: 11_776
            },
            Direction::West,
            0
        )
    );
    assert_eq!(captured.hazard_memory.retained_samples(EntityId(0)), 0);
    assert_eq!(captured.hazard_memory.retained_samples(EntityId(5)), 0);
    assert_eq!(
        captured.observation.environment().unwrap().storm_cloud(),
        Some(StormCloudView {
            position: Position { x: 7_648, y: 6_656 },
            direction: Direction::East,
            leg_remaining_ticks: 4,
        })
    );
}

#[test]
fn hypothesized_row_two_refuge_has_no_raw_plant_route() {
    let captured = captured_clear_basis();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let result = complete_candidate(&captured, CellId::new(13, 2), &mut work);
    assert_eq!(result, Err("plant:Route(NoRoute)".to_owned()));
    assert_eq!(work.spent(), 1_345);
}

#[test]
fn southern_refuge_fits_the_existing_cap_and_needs_its_exact_charged_work() {
    let captured = captured_clear_basis();
    let mut cap = WorkBudget::new(8_192);
    let cap_result = complete_candidate(&captured, CellId::new(13, 3), &mut cap);
    let mut exact = WorkBudget::new(3_889);
    let exact_result = complete_candidate(&captured, CellId::new(13, 3), &mut exact);
    let mut one_short = WorkBudget::new(3_888);
    let short_result = complete_candidate(&captured, CellId::new(13, 3), &mut one_short);
    assert_eq!(cap_result, exact_result);
    assert!(exact_result.is_ok());
    assert!(short_result.unwrap_err().contains("WorkExhausted"));
    assert_eq!(cap.spent(), 3_889);
    assert_eq!(exact.spent(), 3_889);
    // The final eight-unit charge fails atomically, leaving seven units unused.
    assert_eq!(one_short.spent(), 3_881);
    assert_eq!(one_short.remaining(), 7);
}

#[test]
fn explicitly_named_alternate_refuges_are_separate_diagnostics() {
    let captured = captured_clear_basis();
    let mut southern_work = WorkBudget::new(WORK_ALLOWANCE);
    let southern = complete_candidate(&captured, CellId::new(13, 3), &mut southern_work).unwrap();
    assert_eq!(southern.0.placed_at, Tick(439));
    assert_eq!(southern.0.cell, CellId::new(11, 1));
    assert_eq!(southern.0.detonates_at, Tick(619));
    assert_eq!(southern.1, Tick(649));
    assert_eq!(southern.2, Some(true));
    assert_eq!(southern.3, Tick(649));
    assert_eq!(southern_work.spent(), 3_889);

    let mut northern_work = WorkBudget::new(WORK_ALLOWANCE);
    let northern = complete_candidate(&captured, CellId::new(13, 1), &mut northern_work);
    assert_eq!(northern, Err("plant:Route(NoRoute)".to_owned()));
    assert_eq!(northern_work.spent(), 1_208);
}

#[test]
fn hard_v57_certifies_the_useful_corner_farm_plant() {
    let captured = captured_clear_basis();
    let key = MemoryKey {
        world: captured.observation.parts().world,
        cpu_seat: captured.observation.parts().recipient,
    };
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = plan_hard_corner_refuges_v57_with_origin(
        &captured.observation,
        &captured.memory,
        &captured.hazard_memory,
        SETTLED_STOP,
        None,
        &mut CpuRng::from_seed(key, [57; 32]),
        &mut work,
    )
    .unwrap();
    let Some(ExecutionCertificate::Plant(plant)) = planned.survival.execution else {
        panic!("expected v57 corner plant")
    };
    assert_eq!(
        planned.survival.control.intent,
        crate::cpu::Intent {
            movement: MovementIntent::Direction(Direction::East),
            manual: Some(crate::cpu::ManualIntent::Bomb),
        }
    );
    assert_eq!(plant.placement().cell, CellId::new(11, 1));
    assert_eq!(plant.placement().placed_at, Tick(439));
    assert_eq!(plant.placement().detonates_at, Tick(619));
    assert_eq!(plant.route().survive_until(), Tick(649));
    assert_eq!(planned.contact.unwrap().safe_until(), Tick(649));
    assert_eq!(work.spent(), 4_239);
}

#[test]
fn standard_v56_certifies_the_same_closed_corner_plant() {
    let captured = captured_clear_basis();
    let key = MemoryKey {
        world: captured.observation.parts().world,
        cpu_seat: captured.observation.parts().recipient,
    };
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = plan_corner_refuges_v56_with_origin(
        &captured.observation,
        &captured.memory,
        &captured.hazard_memory,
        SETTLED_STOP,
        None,
        &mut CpuRng::from_seed(key, [56; 32]),
        &mut work,
    )
    .unwrap();
    assert_eq!(
        planned.survival.control.intent.manual,
        Some(crate::cpu::ManualIntent::Bomb)
    );
    assert!(matches!(
        planned.survival.execution,
        Some(ExecutionCertificate::Plant(_))
    ));
    assert!(planned.contact.is_some());
    assert_eq!(work.spent(), 4_239);
}

#[test]
fn archived_hard_v53_keeps_its_nonplant_result_and_work() {
    let captured = captured_clear_basis();
    let key = MemoryKey {
        world: captured.observation.parts().world,
        cpu_seat: captured.observation.parts().recipient,
    };
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = plan_hard_pit_avoidance_v53_with_origin(
        &captured.observation,
        &captured.memory,
        &captured.hazard_memory,
        SETTLED_STOP,
        None,
        &mut CpuRng::from_seed(key, [55; 32]),
        &mut work,
    )
    .unwrap();
    assert_eq!(
        planned.survival.control.intent,
        crate::cpu::Intent::moving(Direction::East)
    );
    assert!(planned.survival.execution.is_none());
    assert_eq!(work.spent(), 58_041);
}

#[test]
fn hard_v57_measures_the_existing_child_allocation_boundary() {
    let captured = captured_clear_basis();
    let key = MemoryKey {
        world: captured.observation.parts().world,
        cpu_seat: captured.observation.parts().recipient,
    };
    let run = |allowance| {
        let mut work = WorkBudget::new(allowance);
        let planned = plan_hard_corner_refuges_v57_with_origin(
            &captured.observation,
            &captured.memory,
            &captured.hazard_memory,
            SETTLED_STOP,
            None,
            &mut CpuRng::from_seed(key, [57; 32]),
            &mut work,
        )
        .unwrap();
        (planned, work.spent(), work.remaining())
    };
    let exact = run(31_511);
    let one_short = run(31_510);
    assert_eq!(
        exact.0.survival.control.intent,
        crate::cpu::Intent {
            movement: MovementIntent::Direction(Direction::East),
            manual: Some(crate::cpu::ManualIntent::Bomb),
        }
    );
    assert!(matches!(
        exact.0.survival.execution,
        Some(ExecutionCertificate::Plant(_))
    ));
    assert_eq!((exact.1, exact.2), (4_239, 27_272));
    assert_eq!(
        one_short.0.survival.control.intent,
        crate::cpu::Intent::moving(Direction::East)
    );
    assert!(one_short.0.survival.execution.is_none());
    assert_eq!((one_short.1, one_short.2), (28_321, 3_189));
}

#[test]
fn blocked_away_refuges_fail_closed_outside_the_narrow_clear_static_case() {
    let make_flame = |mut captured: super::foreign_bomb_farming_clear_fixture::ClearBasis| {
        captured.observation = captured
            .observation
            .map_parts(|parts| {
                parts.flames.push(ObservedFlame {
                    cell: CellId::new(10, 1),
                    remaining_ticks: 1,
                });
            })
            .unwrap();
        captured
    };
    let block_both_refuges =
        |mut captured: super::foreign_bomb_farming_clear_fixture::ClearBasis| {
            captured.observation = captured
                .observation
                .map_parts(|parts| {
                    parts
                        .arena
                        .blocks
                        .extend([CellId::new(9, 3), CellId::new(13, 3)]);
                    parts.arena.blocks.sort_unstable();
                })
                .unwrap();
            captured
        };
    for captured in [
        make_flame(captured_clear_basis()),
        block_both_refuges(captured_clear_basis()),
    ] {
        let key = MemoryKey {
            world: captured.observation.parts().world,
            cpu_seat: captured.observation.parts().recipient,
        };
        let planned = plan_hard_corner_refuges_v57_with_origin(
            &captured.observation,
            &captured.memory,
            &captured.hazard_memory,
            SETTLED_STOP,
            None,
            &mut CpuRng::from_seed(key, [57; 32]),
            &mut WorkBudget::new(WORK_ALLOWANCE),
        )
        .unwrap();
        assert!(!matches!(
            planned.survival.execution,
            Some(ExecutionCertificate::Plant(_))
        ));
    }
}

fn complete_candidate(
    captured: &super::foreign_bomb_farming_clear_fixture::ClearBasis,
    refuge: CellId,
    work: &mut WorkBudget,
) -> Result<(crate::cpu::routing::PlannedBomb, Tick, Option<bool>, Tick), String> {
    let plant = plan_lair_ordinary_plant_to_refuge(
        &captured.observation,
        &captured.hazard_memory,
        PlantRequest {
            execution_tick: Tick(439),
            settled_stop_tick: SETTLED_STOP,
            first_movement: MovementIntent::Direction(Direction::East),
            limits: RouteLimits::default(),
        },
        refuge,
        work,
    )
    .map_err(|error| format!("plant:{error:?}"))?;
    let useful = useful_plant_for(
        &captured.observation,
        plant.placement(),
        StrategicTarget::Block { cell: FARM_TARGET },
        work,
    );
    let contact = MobContactLease::certify_route(&captured.observation, plant.route(), work)
        .map_err(|error| format!("contact:{error:?}"))?;
    Ok((
        *plant.placement(),
        plant.route().survive_until(),
        useful,
        contact.safe_until(),
    ))
}

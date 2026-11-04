use crate::cpu::forecast::{
    ForecastLimits, HazardKind, HypotheticalPlacement, TickWindow, UncertaintyReason, forecast,
};
use crate::cpu::policy::{StrategicObjectiveKind, StrategicPhase, StrategicTarget};
use crate::cpu::routing::{
    PlannedBomb, PlantFailure, PlantRequest, RouteFailure, RouteLimits, plan_lair_ordinary_plant,
    plan_lair_ordinary_plant_to_refuge,
};
use crate::cpu::{
    BombKnowledge, CpuRng, Intent, MemoryKey, MovementIntent, ObservedActor, ObservedBombTrigger,
    WorkBudget,
};
use crate::game::{CellId, Direction, EnabledItems, EntityId, Position, Tick};

use super::foreign_bomb_farming_fixture::{CPU, SETTLED_STOP, captured_basis};
use super::{
    WORK_ALLOWANCE, plan_hard_blocked_births_v43_with_origin,
    plan_hard_corner_refuges_v57_with_origin,
};

#[test]
fn captured_basis_preserves_public_scene_memory_and_two_sample_bomb_history() {
    let captured = captured_basis();
    let observation = &captured.observation;
    let parts = observation.parts();
    assert_eq!(parts.tick, Tick(234));
    assert_eq!(parts.rules.running_updates, 54);
    assert_eq!(parts.rules.remaining_ticks, 10_746);
    assert_eq!(parts.rules.settings.enabled_items(), EnabledItems::ALL);
    assert_eq!(parts.arena.blocks.len(), 48);
    assert!(parts.flames.is_empty());
    assert!(parts.oil.is_empty());
    assert!(parts.pickups.is_empty());
    let actors = parts
        .actors
        .iter()
        .map(|actor| match actor {
            ObservedActor::Visible(actor) => (actor.seat, actor.position, actor.live_bomb_count),
            ObservedActor::Hidden { .. } => unreachable!(),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actors[1],
        (
            CPU,
            Position {
                x: 11_727,
                y: 1_536
            },
            0
        )
    );
    assert_eq!(actors[2].2, 1);
    assert_eq!(actors[3].2, 1);
    assert_eq!(actors[0].1, Position { x: 1_536, y: 1_536 });
    assert_eq!(
        actors[2].1,
        Position {
            x: 1_638,
            y: 11_776
        }
    );
    assert_eq!(
        actors[3].1,
        Position {
            x: 13_722,
            y: 11_776
        }
    );
    let BombKnowledge::Visible(bombs) = &parts.bombs else {
        unreachable!()
    };
    assert_eq!(bombs.len(), 2);
    assert_eq!(
        (bombs[0].id, bombs[0].owner, bombs[0].cell),
        (EntityId(0), EntityId(3), CellId::new(1, 10))
    );
    assert_eq!(
        (bombs[1].id, bombs[1].owner, bombs[1].cell),
        (EntityId(5), EntityId(4), CellId::new(13, 10))
    );
    for bomb in bombs {
        assert_eq!(bomb.placed_at, Tick(205));
        assert_eq!(bomb.range, 2);
        assert_eq!(
            bomb.trigger,
            ObservedBombTrigger::Timed {
                remaining_ticks: 151,
                paused: false,
            }
        );
        assert_eq!(captured.hazard_memory.retained_samples(bomb.id), 2);
    }
    assert_eq!(captured.memory.route()[0], Some(CellId::new(11, 1)));
    assert_eq!(captured.memory.refuge(), Some(CellId::new(11, 1)));
    let intent = captured.memory.strategic_intent().unwrap();
    assert_eq!(intent.objective, StrategicObjectiveKind::Farm);
    assert_eq!(
        intent.target,
        StrategicTarget::Block {
            cell: CellId::new(11, 2)
        }
    );
    assert_eq!(intent.approach_cell, Some(CellId::new(11, 1)));
    assert_eq!(intent.phase, StrategicPhase::Approach);
    assert_eq!(intent.observation_tick, Tick(210));
    assert_eq!(intent.score, 2);
    assert_eq!(intent.committed_until, Tick(303));
    assert_eq!(intent.progress_deadline, Tick(390));
    assert_eq!(
        intent.last_cpu_position,
        Position {
            x: 12_750,
            y: 1_536
        }
    );
    assert_eq!(intent.last_distance, 1);
    assert_eq!(SETTLED_STOP, Tick(234));
    assert!(observation.environment().is_some());
    assert_eq!(observation.respawn_lairs().unwrap().lairs().len(), 2);
    assert_eq!(observation.spike_pits().unwrap().pits().len(), 4);
}

#[test]
fn archived_hard_planner_reproduces_the_queued_north_non_manual_result() {
    let captured = captured_basis();
    let key = MemoryKey {
        world: captured.observation.parts().world,
        cpu_seat: CPU,
    };
    assert_eq!(captured.hazard_memory.key(), key);
    // The queued basis did not expose private RNG state. This controlled seed
    // is not claimed as restored history; the archived choice and work are
    // stable for this fully captured public basis.
    let mut rng = CpuRng::from_seed(key, [47; 32]);
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = plan_hard_blocked_births_v43_with_origin(
        &captured.observation,
        &captured.memory,
        &captured.hazard_memory,
        SETTLED_STOP,
        None,
        &mut rng,
        &mut work,
    )
    .unwrap();
    assert_eq!(
        planned.survival.control.intent,
        Intent {
            movement: MovementIntent::Direction(Direction::North),
            manual: None,
        }
    );
    assert!(planned.survival.execution.is_none());
    assert_eq!(work.spent(), 10_443);
}

#[test]
fn corner_refuges_remain_guarded_while_foreign_bombs_are_visible() {
    let captured = captured_basis();
    let key = MemoryKey {
        world: captured.observation.parts().world,
        cpu_seat: CPU,
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
    assert_eq!(
        planned.survival.control.intent,
        Intent::moving(Direction::North)
    );
    assert!(planned.survival.execution.is_none());
    assert_eq!(work.spent(), 10_627);
}

#[test]
fn exact_queued_north_plant_candidate_exhausts_allowance_before_completing_proof() {
    let captured = captured_basis();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let result = plan_lair_ordinary_plant(
        &captured.observation,
        &captured.hazard_memory,
        PlantRequest {
            execution_tick: Tick(241),
            settled_stop_tick: SETTLED_STOP,
            first_movement: MovementIntent::Direction(Direction::North),
            limits: RouteLimits::default(),
        },
        &mut work,
    );
    assert_eq!(
        result,
        Err(PlantFailure::Route(RouteFailure::WorkExhausted))
    );
    assert_eq!(work.spent(), WORK_ALLOWANCE);
}

#[test]
fn other_first_moves_are_separate_diagnostics_not_aggregate_budget_claims() {
    let captured = captured_basis();
    for direction in [Direction::East, Direction::South, Direction::West] {
        let mut work = WorkBudget::new(WORK_ALLOWANCE);
        let result = plan_lair_ordinary_plant(
            &captured.observation,
            &captured.hazard_memory,
            PlantRequest {
                execution_tick: Tick(241),
                settled_stop_tick: SETTLED_STOP,
                first_movement: MovementIntent::Direction(direction),
                limits: RouteLimits::default(),
            },
            &mut work,
        );
        assert_eq!(
            result,
            Err(PlantFailure::Route(RouteFailure::WorkExhausted)),
            "direction={direction:?}"
        );
        assert_eq!(work.spent(), WORK_ALLOWANCE, "direction={direction:?}");
    }
}

#[test]
fn stopped_first_move_is_an_independent_raw_plant_diagnostic() {
    let captured = captured_basis();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let result = plan_lair_ordinary_plant(
        &captured.observation,
        &captured.hazard_memory,
        PlantRequest {
            execution_tick: Tick(241),
            settled_stop_tick: SETTLED_STOP,
            first_movement: MovementIntent::Stop,
            limits: RouteLimits::default(),
        },
        &mut work,
    );
    assert_eq!(
        result,
        Err(PlantFailure::Route(RouteFailure::WorkExhausted))
    );
    assert_eq!(work.spent(), WORK_ALLOWANCE);
}

#[test]
fn two_samples_do_not_remove_certified_foreign_bomb_motion() {
    let captured = captured_basis();
    let mut absent_work = WorkBudget::new(WORK_ALLOWANCE);
    let absent = forecast(
        &captured.observation,
        &captured.hazard_memory,
        None,
        ForecastLimits::default(),
        &mut absent_work,
    )
    .unwrap();
    assert_eq!(absent_work.spent(), 1_179);
    assert_eq!(absent.stationary_assumptions(), &[EntityId(0), EntityId(5)]);
    let foreign_lethal = TickWindow {
        start: Tick(385),
        end: Tick(1_074),
    };
    for cell in [
        CellId::new(11, 1),
        CellId::new(12, 1),
        CellId::new(13, 1),
        CellId::new(13, 2),
        CellId::new(12, 3),
        CellId::new(13, 3),
    ] {
        assert!(has_uncertainty(
            absent.cell(cell).unwrap(),
            foreign_lethal,
            HazardKind::Lethal,
            UncertaintyReason::BombMotion(EntityId(5)),
        ));
    }
    assert_eq!(
        absent.cell(CellId::new(10, 1)).unwrap().uncertainty.len(),
        0
    );
    for cell in [CellId::new(13, 1), CellId::new(13, 2), CellId::new(13, 3)] {
        assert!(has_uncertainty(
            absent.cell(cell).unwrap(),
            TickWindow {
                start: Tick(234),
                end: Tick(386),
            },
            HazardKind::Blocking,
            UncertaintyReason::BombMotion(EntityId(5)),
        ));
    }
}

#[test]
fn named_east_refuge_is_useful_but_has_no_conservative_raw_route() {
    let captured = captured_basis();
    let placement = HypotheticalPlacement {
        placed_at: Tick(241),
        owner: EntityId(2),
        cell: CellId::new(11, 1),
        range: 2,
        blast: crate::game::BlastKind::Standard,
        detonates_at: Tick(421),
    };
    let planned = PlannedBomb {
        placed_at: placement.placed_at,
        owner: placement.owner,
        cell: placement.cell,
        range: placement.range,
        blast: placement.blast,
        detonates_at: placement.detonates_at,
    };
    let mut usefulness_work = WorkBudget::new(WORK_ALLOWANCE);
    let usefulness = super::plant_value::useful_plant_for(
        &captured.observation,
        &planned,
        captured.memory.strategic_intent().unwrap().target,
        &mut usefulness_work,
    );
    assert_eq!(usefulness, Some(true));
    assert_eq!(usefulness_work.spent(), 199);
    let mut present_work = WorkBudget::new(WORK_ALLOWANCE);
    let present = forecast(
        &captured.observation,
        &captured.hazard_memory,
        Some(placement),
        ForecastLimits::default(),
        &mut present_work,
    )
    .unwrap();
    assert_eq!(present_work.spent(), 1_706);
    let own_blast = TickWindow {
        start: Tick(421),
        end: Tick(451),
    };
    for column in 10..=13 {
        assert_eq!(
            present.cell(CellId::new(column, 1)).unwrap().lethal,
            [own_blast]
        );
    }
    for cell in [
        CellId::new(10, 1),
        CellId::new(11, 1),
        CellId::new(12, 1),
        CellId::new(13, 1),
        CellId::new(13, 2),
        CellId::new(12, 3),
        CellId::new(13, 3),
    ] {
        assert!(has_uncertainty(
            present.cell(cell).unwrap(),
            TickWindow {
                start: Tick(421),
                end: Tick(1_074),
            },
            HazardKind::Lethal,
            UncertaintyReason::BombMotion(EntityId(0)),
        ));
    }

    let mut route_work = WorkBudget::new(WORK_ALLOWANCE);
    let route = plan_lair_ordinary_plant_to_refuge(
        &captured.observation,
        &captured.hazard_memory,
        PlantRequest {
            execution_tick: Tick(241),
            settled_stop_tick: SETTLED_STOP,
            first_movement: MovementIntent::Direction(Direction::East),
            limits: RouteLimits::default(),
        },
        CellId::new(13, 3),
        &mut route_work,
    );
    assert_eq!(route, Err(PlantFailure::Route(RouteFailure::NoRoute)));
    assert_eq!(route_work.spent(), 3_232);
    // With no route certificate, usefulness is only an independent raw
    // placement fact and MobContactLease certification cannot be reached.
}

fn has_uncertainty(
    cell: &crate::cpu::forecast::CellForecast,
    window: TickWindow,
    kind: HazardKind,
    reason: UncertaintyReason,
) -> bool {
    cell.uncertainty.iter().any(|uncertain| {
        uncertain.window == window && uncertain.kind == kind && uncertain.reason == reason
    })
}

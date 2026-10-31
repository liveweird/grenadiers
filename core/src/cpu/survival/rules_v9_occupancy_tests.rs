use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::policy::{GoalKind, GoalMemory, PolicyError, UtilityMemory};
use crate::cpu::{
    CpuRng, MemoryKey, MovementIntent, Observation, ObservedActor, ObservedEnvironment,
    ObservedPickup, ObservedRespawnLairs, WorkBudget,
};
use crate::game::{
    CellId, Direction, EntityId, LairStatusView, PickupKind, Position, SeatId, StormCloudConfig,
    Terrain, Tick, lair_layout_v1,
};
use crate::room::{BoardSize, RespawnLairConfig, RespawnLairSettings};

use super::compatibility::{can_admit_ordinary, visible_rivals_allow_segment};

fn observed(cpu: CellId, rival: Option<CellId>) -> Observation {
    let mut parts = crate::cpu::policy::tests::observation(cpu).into_parts();
    parts.rules.version = 9;
    let mut timers = vec![None, Some(0)];
    if let Some(cell) = rival {
        let mut actor = match parts.actors[1].clone() {
            ObservedActor::Visible(actor) => actor,
            ObservedActor::Hidden { .. } => unreachable!(),
        };
        actor.id = EntityId(1);
        actor.seat = SeatId::One;
        actor.position = Position::center(cell);
        parts.actors[0] = ObservedActor::Visible(actor);
        timers[0] = Some(0);
    }
    Observation::new_rules_v9(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, timers),
        None,
    )
    .unwrap()
}

fn with_enabled_lairs(observation: Observation) -> Observation {
    with_lairs(observation, true)
}

fn with_lairs(observation: Observation, spawning: bool) -> Observation {
    let environment = observation.environment().unwrap().clone();
    let mut parts = observation.into_parts();
    let cells = lair_layout_v1(BoardSize::FifteenByThirteen);
    for cell in cells {
        let index = parts.arena.index(cell).unwrap();
        parts.arena.terrain[index] = Terrain::Wall;
    }
    let lairs = cells
        .into_iter()
        .map(|cell| LairStatusView {
            cell,
            next_spawn_at: Some(if spawning {
                parts.tick
            } else {
                Tick(parts.tick.0 + 300)
            }),
            living_mobs: 0,
        })
        .collect();
    Observation::new_rules_v9(
        parts,
        environment,
        Some(ObservedRespawnLairs::new(
            RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap()),
            lairs,
            Vec::new(),
        )),
    )
    .unwrap()
}

#[test]
fn v9_segment_check_uses_visible_living_footprints_and_not_hidden_positions() {
    let blocked = observed(CellId::new(7, 7), Some(CellId::new(9, 7)));
    assert!(!visible_rivals_allow_segment(
        &blocked,
        Position::center(CellId::new(7, 7)),
        Position::center(CellId::new(9, 7)),
    ));

    let hidden = observed(CellId::new(7, 7), None);
    assert!(visible_rivals_allow_segment(
        &hidden,
        Position::center(CellId::new(7, 7)),
        Position::center(CellId::new(9, 7)),
    ));
}

#[test]
fn unrelated_visible_rival_motion_does_not_starve_ordinary_admission() {
    let basis = with_enabled_lairs(observed(CellId::new(7, 7), Some(CellId::new(11, 7))));
    let current = with_enabled_lairs(observed(CellId::new(7, 7), Some(CellId::new(11, 8))));
    assert!(can_admit_ordinary(&basis, &current));
}

#[test]
fn v9_has_a_distinct_planner_entrypoint_and_keeps_legacy_policy_closed() {
    let observed = observed(CellId::new(7, 7), Some(CellId::new(11, 7)));
    let key = MemoryKey {
        world: observed.parts().world,
        cpu_seat: observed.parts().recipient,
    };
    let memory = UtilityMemory::new(key);
    assert_eq!(
        crate::cpu::policy::plan(
            &observed,
            &memory,
            &mut CpuRng::from_seed(key, [71; 32]),
            &mut WorkBudget::new(65_536),
        ),
        Err(PolicyError::InvalidObservation)
    );
    assert!(
        super::plan_respawn(
            &observed,
            &memory,
            &ObservedHazardMemory::new(key),
            observed.parts().tick,
            &mut CpuRng::from_seed(key, [72; 32]),
            &mut WorkBudget::new(65_536),
        )
        .is_ok()
    );
}

#[test]
fn v9_firing_position_plants_only_with_a_complete_escape_certificate() {
    let observed = observed(CellId::new(9, 7), Some(CellId::new(11, 7)));
    let key = MemoryKey {
        world: observed.parts().world,
        cpu_seat: observed.parts().recipient,
    };
    let planned = super::plan_respawn(
        &observed,
        &UtilityMemory::new(key),
        &ObservedHazardMemory::new(key),
        observed.parts().tick,
        &mut CpuRng::from_seed(key, [73; 32]),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert_eq!(
        planned.survival.control.intent.manual,
        Some(crate::cpu::ManualIntent::Bomb)
    );
    assert!(planned.survival.execution.is_some());
}

#[test]
fn v23_unrelated_capped_pickup_does_not_starve_a_safe_plant() {
    for lairs in [false, true] {
        let mut scene = observed(CellId::new(9, 7), Some(CellId::new(11, 7)))
            .map_parts(|parts| {
                let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
                    unreachable!()
                };
                actor.bomb_capacity = 9;
                parts.pickups.push(ObservedPickup {
                    cell: CellId::new(1, 1),
                    kind: PickupKind::BombCapacity,
                });
            })
            .unwrap();
        if lairs {
            scene = with_lairs(scene, false);
        }
        let key = MemoryKey {
            world: scene.parts().world,
            cpu_seat: scene.parts().recipient,
        };
        let planned = super::plan_respawn_v23_with_origin(
            &scene,
            &UtilityMemory::new(key),
            &ObservedHazardMemory::new(key),
            scene.parts().tick,
            None,
            &mut CpuRng::from_seed(key, [74; 32]),
            &mut WorkBudget::new(65_536),
        )
        .unwrap();
        assert_eq!(
            planned.survival.control.intent.manual,
            Some(crate::cpu::ManualIntent::Bomb),
            "lairs={lairs}"
        );
        assert!(planned.survival.execution.is_some(), "lairs={lairs}");
    }
}

#[test]
fn v23_commits_one_goal_and_makes_cell_level_progress_without_ab_cycles() {
    let mut scene = observed(CellId::new(7, 7), None)
        .map_parts(|parts| {
            parts.pickups.push(ObservedPickup {
                cell: CellId::new(11, 7),
                kind: PickupKind::GoldenFlame,
            });
        })
        .unwrap();
    let key = MemoryKey {
        world: scene.parts().world,
        cpu_seat: scene.parts().recipient,
    };
    let mut memory = UtilityMemory::new(key);
    let mut rng = CpuRng::from_seed(key, [75; 32]);
    let mut settled = vec![CellId::new(7, 7)];
    for _ in 0..6 {
        let plan = crate::cpu::policy::plan_respawn_v23(
            &scene,
            &memory,
            &mut rng,
            &mut WorkBudget::new(65_536),
        )
        .unwrap();
        memory = plan.next_memory;
        let current = *settled.last().unwrap();
        let next = match plan.intent.movement {
            MovementIntent::Direction(Direction::North) => {
                CellId::new(current.column, current.row - 1)
            }
            MovementIntent::Direction(Direction::East) => {
                CellId::new(current.column + 1, current.row)
            }
            MovementIntent::Direction(Direction::South) => {
                CellId::new(current.column, current.row + 1)
            }
            MovementIntent::Direction(Direction::West) => {
                CellId::new(current.column - 1, current.row)
            }
            MovementIntent::Stop => break,
        };
        settled.push(next);
        scene = scene
            .map_parts(|parts| {
                let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
                    unreachable!()
                };
                actor.position = Position::center(next);
            })
            .unwrap();
    }
    assert_eq!(
        memory.goal().map(|goal| goal.cell),
        Some(CellId::new(11, 7))
    );
    assert!(settled.last().unwrap().column > 7, "{settled:?}");
    assert!(
        settled.windows(6).all(|cells| !(cells[0] == cells[2]
            && cells[2] == cells[4]
            && cells[1] == cells[3]
            && cells[3] == cells[5])),
        "{settled:?}"
    );
}

#[test]
fn v23_ignores_an_unreachable_high_pickup_for_a_reachable_farming_goal() {
    let scene = observed(CellId::new(7, 7), None)
        .map_parts(|parts| {
            parts.pickups.push(ObservedPickup {
                cell: CellId::new(11, 7),
                kind: PickupKind::Heart,
            });
            parts.arena.blocks.extend([
                CellId::new(10, 7),
                CellId::new(12, 7),
                CellId::new(11, 6),
                CellId::new(11, 8),
            ]);
        })
        .unwrap();
    let key = MemoryKey {
        world: scene.parts().world,
        cpu_seat: scene.parts().recipient,
    };
    let plan = crate::cpu::policy::plan_respawn_v23(
        &scene,
        &UtilityMemory::new(key),
        &mut CpuRng::from_seed(key, [76; 32]),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert_ne!(plan.intent.movement, MovementIntent::Stop);
    assert_eq!(
        plan.next_memory.goal().map(|goal| goal.kind),
        Some(GoalKind::Farming)
    );
}

#[test]
fn v23_discards_stale_pickup_priority_when_the_cell_becomes_an_opening() {
    let stale = CellId::new(8, 7);
    let replacement = CellId::new(7, 9);
    let scene = observed(CellId::new(7, 7), None)
        .map_parts(|parts| {
            parts.arena.blocks.push(CellId::new(9, 7));
            parts.pickups.push(ObservedPickup {
                cell: replacement,
                kind: PickupKind::BombCapacity,
            });
        })
        .unwrap();
    let key = MemoryKey {
        world: scene.parts().world,
        cpu_seat: scene.parts().recipient,
    };
    let memory = UtilityMemory::new(key).with_progress(
        CellId::new(7, 7),
        GoalMemory {
            kind: GoalKind::Pickup,
            cell: stale,
            priority: 9,
        },
    );
    let plan = crate::cpu::policy::plan_respawn_v23(
        &scene,
        &memory,
        &mut CpuRng::from_seed(key, [77; 32]),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert_eq!(
        plan.next_memory.goal().map(|goal| goal.cell),
        Some(replacement)
    );
}

#[test]
fn newly_blocked_next_segment_invalidates_a_committed_v9_pulse() {
    let basis = with_enabled_lairs(observed(CellId::new(7, 7), Some(CellId::new(11, 7))));
    let key = MemoryKey {
        world: basis.parts().world,
        cpu_seat: basis.parts().recipient,
    };
    let mut work = WorkBudget::new(65_536);
    let hazards = forecast(
        &basis,
        &ObservedHazardMemory::new(key),
        None,
        ForecastLimits::default(),
        &mut work,
    )
    .unwrap();
    let first = TickWindow {
        start: crate::game::Tick(basis.parts().tick.0 + 1),
        end: crate::game::Tick(basis.parts().tick.0 + 2),
    };
    let lease = crate::cpu::routing::certify_respawn_pulse(
        &basis,
        &hazards,
        crate::cpu::routing::PulseRequest {
            movement: crate::cpu::MovementIntent::Direction(crate::game::Direction::East),
            hold_ticks: 12,
            start_window: first,
            stopped_safe_until: crate::game::Tick(basis.parts().tick.0 + 30),
            mode: crate::cpu::routing::RouteMode::CertifiedOnly,
            settled_stop_tick: basis.parts().tick,
        },
        &mut work,
    )
    .unwrap();
    let committed =
        super::CommittedPulse::admit(lease, basis.clone(), &basis, basis.parts().tick).unwrap();
    let environment = basis.environment().unwrap().clone();
    let lairs = basis.respawn_lairs().cloned();
    let mut blocked_parts = basis.clone().into_parts();
    let ObservedActor::Visible(rival) = &mut blocked_parts.actors[0] else {
        unreachable!()
    };
    rival.position = Position {
        x: Position::center(CellId::new(7, 7)).x + 650,
        y: Position::center(CellId::new(7, 7)).y,
    };
    let blocked = Observation::new_rules_v9(blocked_parts, environment, lairs).unwrap();
    assert_eq!(
        committed.validate(&blocked, basis.parts().tick),
        super::PulseStatus::Invalidated
    );
}

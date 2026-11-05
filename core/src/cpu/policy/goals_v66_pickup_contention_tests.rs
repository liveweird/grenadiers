use crate::cpu::policy::pit_avoidance::PitMask;
use crate::cpu::policy::{GoalModes, StrategicTarget, UtilityMemory};
use crate::cpu::{
    BombKnowledge, Observation, ObservedActor, ObservedBomb, ObservedBombTrigger, ObservedEffect,
    ObservedPickup, UnknownBombFacts, WorkBudget,
};
use crate::game::{
    BlastKind, CellId, EffectKind, EntityId, Equipment, PickupKind, Position, SpeedLevel, Terrain,
    Tick,
};

use super::{Candidate, StrategicObjectiveKind, promote_fresh};

const CPU: CellId = CellId::new(7, 7);
const BASELINE: CellId = CellId::new(5, 7);
const CONTESTED: CellId = CellId::new(9, 7);

fn scene() -> Observation {
    scene_with_rival(CellId::new(13, 7))
}

fn scene_with_rival(rival_cell: CellId) -> Observation {
    crate::cpu::policy::tests::observation(CPU)
        .map_parts(|parts| {
            let ObservedActor::Visible(cpu) = &parts.actors[1] else {
                unreachable!()
            };
            let mut rival = cpu.clone();
            rival.id = crate::game::EntityId(1);
            rival.seat = crate::game::SeatId::One;
            rival.position = Position::center(rival_cell);
            parts.actors[0] = ObservedActor::Visible(rival);
            parts.pickups = vec![
                ObservedPickup {
                    cell: BASELINE,
                    kind: PickupKind::BombCapacity,
                },
                ObservedPickup {
                    cell: CONTESTED,
                    kind: PickupKind::BombCapacity,
                },
            ];
        })
        .unwrap()
}

fn candidate(cell: CellId) -> Candidate {
    Candidate {
        objective: StrategicObjectiveKind::Collect,
        target: StrategicTarget::Pickup {
            cell,
            kind: PickupKind::BombCapacity,
        },
        approach: cell,
        score: 7,
        distance: 2,
        first_step: if cell == BASELINE {
            CellId::new(6, 7)
        } else {
            CellId::new(8, 7)
        },
    }
}

fn choose(observation: &Observation, allowance: u32) -> (Candidate, u32) {
    let cpu = observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == observation.parts().recipient => {
                Some(actor)
            }
            _ => None,
        })
        .unwrap();
    let baseline = candidate(BASELINE);
    let mut work = WorkBudget::new(allowance);
    let chosen = promote_fresh(
        observation,
        cpu,
        baseline,
        &[baseline, candidate(CONTESTED)],
        &PitMask::EMPTY,
        &mut work,
    );
    (chosen, work.spent())
}

#[test]
fn equal_value_and_self_distance_prefers_a_two_cell_winnable_contest() {
    let (chosen, spent) = choose(&scene(), 65_536);
    assert_eq!(chosen.target, candidate(CONTESTED).target);
    assert!(spent > 0);
    assert!(spent <= 2_048);
}

#[test]
fn hidden_living_rival_and_partial_budget_keep_the_baseline() {
    let hidden = scene()
        .map_parts(|parts| {
            parts.actors[0] = ObservedActor::Hidden {
                id: crate::game::EntityId(1),
                seat: crate::game::SeatId::One,
                alive: true,
            };
        })
        .unwrap();
    assert_eq!(choose(&hidden, 65_536).0.target, candidate(BASELINE).target);

    let (partial, spent) = choose(&scene(), 16);
    assert_eq!(partial.target, candidate(BASELINE).target);
    assert_eq!(spent, 0);
}

#[test]
fn unequal_intrinsic_value_or_self_distance_cannot_be_promoted() {
    let observation = scene();
    let cpu = match &observation.parts().actors[1] {
        ObservedActor::Visible(cpu) => cpu,
        _ => unreachable!(),
    };
    let baseline = candidate(BASELINE);
    for changed in [
        Candidate {
            score: 6,
            ..candidate(CONTESTED)
        },
        Candidate {
            distance: 3,
            ..candidate(CONTESTED)
        },
    ] {
        let chosen = promote_fresh(
            &observation,
            cpu,
            baseline,
            &[baseline, changed],
            &PitMask::EMPTY,
            &mut WorkBudget::new(65_536),
        );
        assert_eq!(chosen.target, baseline.target);
    }
}

#[test]
fn incompatible_rival_and_unknown_movement_pickup_abort_promotion() {
    let incompatible = scene()
        .map_parts(|parts| {
            let ObservedActor::Visible(rival) = &mut parts.actors[0] else {
                unreachable!()
            };
            rival.speed = SpeedLevel::One;
        })
        .unwrap();
    assert_eq!(
        choose(&incompatible, 65_536).0.target,
        candidate(BASELINE).target
    );

    let curse_corridor = scene()
        .map_parts(|parts| {
            parts.pickups.push(ObservedPickup {
                cell: CellId::new(11, 7),
                kind: PickupKind::SlowFoot,
            });
        })
        .unwrap();
    assert_eq!(
        choose(&curse_corridor, 65_536).0.target,
        candidate(BASELINE).target
    );
}

#[test]
fn hard_wall_detour_is_counted_in_the_public_rival_distance() {
    let detour = scene()
        .map_parts(|parts| {
            for row in 6..=8 {
                let cell = CellId::new(11, row);
                let index = parts.arena.index(cell).unwrap();
                parts.arena.terrain[index] = Terrain::Wall;
            }
        })
        .unwrap();
    assert_eq!(choose(&detour, 65_536).0.target, candidate(BASELINE).target);
}

#[test]
fn cpu_teleport_marker_uses_the_same_six_tick_recency_boundary() {
    let marked = |tick| {
        scene()
            .map_parts(|parts| {
                let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
                    unreachable!()
                };
                cpu.last_teleport_tick = Some(Tick(tick));
            })
            .unwrap()
    };
    assert_eq!(
        choose(&marked(4), 65_536).0.target,
        candidate(BASELINE).target
    );
    assert_eq!(
        choose(&marked(3), 65_536).0.target,
        candidate(CONTESTED).target
    );
}

#[test]
fn contest_margin_accepts_two_through_four_cells_only() {
    for (rival, expected) in [
        (CellId::new(12, 7), BASELINE),
        (CellId::new(13, 7), CONTESTED),
        (CellId::new(13, 9), CONTESTED),
        (CellId::new(13, 10), BASELINE),
    ] {
        assert_eq!(
            choose(&scene_with_rival(rival), 65_536).0.target,
            candidate(expected).target,
            "rival at {rival:?}"
        );
    }
}

#[test]
fn every_incompatible_living_rival_aborts_the_optional_comparison() {
    let mut scenes = Vec::new();
    scenes.push(
        scene()
            .map_parts(|parts| {
                let ObservedActor::Visible(rival) = &mut parts.actors[0] else {
                    unreachable!()
                };
                rival.equipment = Some(Equipment::WallPass);
            })
            .unwrap(),
    );
    scenes.push(
        scene()
            .map_parts(|parts| {
                let ObservedActor::Visible(rival) = &mut parts.actors[0] else {
                    unreachable!()
                };
                rival.voluntary_actions_frozen = true;
            })
            .unwrap(),
    );
    for kind in [EffectKind::SlowFoot, EffectKind::Stopwatch] {
        scenes.push(
            scene()
                .map_parts(|parts| {
                    let ObservedActor::Visible(rival) = &mut parts.actors[0] else {
                        unreachable!()
                    };
                    rival.effects.push(ObservedEffect {
                        kind,
                        remaining_ticks: 10,
                    });
                })
                .unwrap(),
        );
    }
    for observation in scenes {
        assert_eq!(
            choose(&observation, 65_536).0.target,
            candidate(BASELINE).target
        );
    }
}

#[test]
fn dead_rivals_are_irrelevant_but_a_living_hidden_rival_is_unknown() {
    for hidden in [false, true] {
        let dead = scene()
            .map_parts(|parts| {
                parts.rules.version = 6;
                let dead = if hidden {
                    ObservedActor::Hidden {
                        id: EntityId(3),
                        seat: crate::game::SeatId::Three,
                        alive: false,
                    }
                } else {
                    let ObservedActor::Visible(mut rival) = parts.actors[1].clone() else {
                        unreachable!()
                    };
                    rival.id = EntityId(3);
                    rival.seat = crate::game::SeatId::Three;
                    rival.alive = false;
                    ObservedActor::Visible(rival)
                };
                parts.actors.push(dead);
            })
            .unwrap();
        assert_eq!(choose(&dead, 65_536).0.target, candidate(CONTESTED).target);
    }
    let living_hidden = scene()
        .map_parts(|parts| {
            parts.actors[0] = ObservedActor::Hidden {
                id: EntityId(1),
                seat: crate::game::SeatId::One,
                alive: true,
            };
        })
        .unwrap();
    assert_eq!(
        choose(&living_hidden, 65_536).0.target,
        candidate(BASELINE).target
    );
}

#[test]
fn temporary_occupancy_bomb_and_pit_do_not_invent_a_rival_detour() {
    let occupied = scene()
        .map_parts(|parts| {
            let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
                unreachable!()
            };
            cpu.position = Position::center(CellId::new(11, 7));
            parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
                id: EntityId(90),
                placed_at: Tick(1),
                cell: CellId::new(10, 7),
                owner: EntityId(1),
                range: 2,
                trigger: ObservedBombTrigger::Timed {
                    remaining_ticks: 30,
                    paused: false,
                },
                blast: BlastKind::Standard,
                unknown: UnknownBombFacts::ALL,
            }]);
        })
        .unwrap();
    let cpu = match &occupied.parts().actors[1] {
        ObservedActor::Visible(cpu) => cpu,
        _ => unreachable!(),
    };
    let baseline = candidate(BASELINE);
    let pit = PitMask::containing(&occupied, CellId::new(12, 7));
    let chosen = promote_fresh(
        &occupied,
        cpu,
        baseline,
        &[baseline, candidate(CONTESTED)],
        &pit,
        &mut WorkBudget::new(65_536),
    );
    assert_eq!(chosen.target, candidate(CONTESTED).target);
}

#[test]
fn partial_optional_analysis_charges_work_and_discards_its_result() {
    let (chosen, spent) = choose(&scene(), 4_096);
    assert_eq!(chosen.target, candidate(BASELINE).target);
    assert!(spent > 0);
    assert!(spent <= 256);
}

#[test]
fn valid_incoming_pickup_commitment_is_not_interrupted() {
    let observation = scene();
    let actor = match &observation.parts().actors[1] {
        ObservedActor::Visible(actor) => actor,
        _ => unreachable!(),
    };
    let key = crate::cpu::MemoryKey {
        world: observation.parts().world,
        cpu_seat: observation.parts().recipient,
    };
    let baseline = super::super::select_pits_v59_modes(
        &observation,
        actor,
        CPU,
        &UtilityMemory::new(key),
        &mut WorkBudget::new(65_536),
        true,
        GoalModes::visible_yield(),
    )
    .unwrap();
    assert_eq!(baseline.intent.target, candidate(BASELINE).target);
    let retained = super::super::select_pits_v59_modes(
        &observation,
        actor,
        CPU,
        &UtilityMemory::new(key).with_strategic_intent(baseline.intent),
        &mut WorkBudget::new(65_536),
        true,
        GoalModes::visible_yield_contention(),
    )
    .unwrap();
    assert_eq!(retained.intent, baseline.intent);
}

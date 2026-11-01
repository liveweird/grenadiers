use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::{BombKnowledge, CpuRng, ManualIntent, MobContactLease, ObservedActor, WorkBudget};
use crate::game::{
    ActorInit, Arena, CellId, Direction, EntityId, InputAction, Position, RoundAssignment,
    RulesConfig, SeatId, SeatInput, SimulationSeed, State, StormCloudConfig, Tick, World,
};
use crate::room::{RespawnLairConfig, SpikePitConfig};

use super::farm_progress_tests::{captured_tick_7022, key, north_farm_memory};
use super::{
    ExecutionCertificate, WORK_ALLOWANCE, plan_farm_refuges_v36_with_origin,
    plan_hard_farm_refuges_v37_with_origin, plan_refuge_guidance_v33_with_origin,
    plan_refuge_guidance_v34_with_origin,
};

const CPU: SeatId = SeatId::Three;

fn controlled_state() -> State {
    let observed = captured_tick_7022();
    let parts = observed.parts();
    let arena = Arena::new(
        parts.arena.terrain.clone().try_into().unwrap(),
        parts.arena.blocks.iter().copied().collect(),
        [
            CellId::new(1, 1),
            CellId::new(13, 1),
            CellId::new(1, 11),
            CellId::new(13, 11),
        ],
    )
    .unwrap();
    let actors = [
        (1, SeatId::One, CellId::new(1, 1)),
        (2, SeatId::Two, CellId::new(13, 1)),
        (3, CPU, CellId::new(3, 8)),
        (4, SeatId::Four, CellId::new(13, 11)),
    ]
    .map(|(id, seat, cell)| ActorInit {
        id: EntityId(id),
        seat,
        position: Position::center(cell),
        heading: Direction::North,
        speed: crate::game::SpeedLevel::Base,
    });
    State::new_at_tick_with_assignment_and_seed(
        RulesConfig::environment_v10(
            parts.rules.settings,
            StormCloudConfig::Disabled,
            RespawnLairConfig::Disabled,
            SpikePitConfig::Disabled,
        ),
        World::new(arena),
        actors,
        parts.tick,
        RoundAssignment::first(),
        SimulationSeed::new([136; 32]),
    )
    .unwrap()
}

fn input(movement: Option<Direction>, bomb: bool) -> Vec<SeatInput> {
    let mut inputs = vec![SeatInput {
        seat: CPU,
        action: movement.map_or(InputAction::Clear, InputAction::Press),
    }];
    if bomb {
        inputs.push(SeatInput {
            seat: CPU,
            action: InputAction::PlaceBomb,
        });
    }
    inputs
}

fn centered_scene() -> crate::cpu::Observation {
    captured_tick_7022()
        .map_parts(|parts| {
            let ObservedActor::Visible(cpu) = &mut parts.actors[CPU.index()] else {
                unreachable!()
            };
            cpu.position = Position::center(CellId::new(3, 8));
            cpu.heading = Direction::North;
        })
        .unwrap()
}

#[test]
fn discovered_joint_refuge_executes_a_useful_plant_and_survives_its_flames() {
    let observed = centered_scene();
    let key = key(&observed);
    let memory = north_farm_memory(&observed);
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observed).unwrap();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = plan_farm_refuges_v36_with_origin(
        &observed,
        &memory,
        &history,
        observed.parts().tick,
        None,
        &mut CpuRng::from_seed(key, [136; 32]),
        &mut work,
    )
    .unwrap();
    assert_eq!(
        planned.survival.control.intent.manual,
        Some(ManualIntent::Bomb)
    );
    let Some(ExecutionCertificate::Plant(certificate)) = planned.survival.execution else {
        panic!("joint-refuge policy did not return a plant certificate");
    };
    let first = certificate.route().start_window().start;
    let lane = certificate.route().lane(first).unwrap();
    let mut state = controlled_state();
    while state.tick().0 + 1 < first.0 {
        state.step(&[]).unwrap();
    }
    while state.tick() < certificate.route().survive_until() {
        let update = Tick(state.tick().0 + 1);
        let movement = certificate
            .route()
            .steps()
            .iter()
            .enumerate()
            .find_map(|(index, step)| {
                let start = lane.step_start(index)?;
                let stop = lane.stop_update(index)?;
                (start <= update && update < stop).then_some(match step {
                    crate::cpu::routing::RouteStep::Move { direction, .. } => Some(*direction),
                    crate::cpu::routing::RouteStep::Wait { .. } => None,
                })
            })
            .flatten();
        state
            .step(&input(
                movement,
                update == certificate.placement().placed_at,
            ))
            .unwrap();
        assert!(state.actor(CPU).unwrap().alive());
    }
    assert!(!state.world().arena.blocks().contains(&CellId::new(3, 7)));
    assert_eq!(state.live_bomb_count(CPU), 0);
}

#[test]
fn joint_refuge_keeps_unsafe_current_pose_blocked_and_fails_closed() {
    let captured = captured_tick_7022();
    let centered = centered_scene();
    let cases = [
        captured,
        centered
            .clone()
            .map_parts(|parts| {
                parts.arena.blocks.push(CellId::new(1, 9));
            })
            .unwrap(),
        centered
            .map_parts(|parts| parts.bombs = BombKnowledge::Unavailable)
            .unwrap(),
    ];
    for observed in cases {
        let key = key(&observed);
        let memory = north_farm_memory(&observed);
        let mut history = ObservedHazardMemory::new(key);
        let _ = history.observe(&observed);
        let planned = plan_farm_refuges_v36_with_origin(
            &observed,
            &memory,
            &history,
            observed.parts().tick,
            None,
            &mut CpuRng::from_seed(key, [138; 32]),
            &mut WorkBudget::new(WORK_ALLOWANCE),
        )
        .unwrap();
        assert_ne!(
            planned.survival.control.intent.manual,
            Some(ManualIntent::Bomb)
        );
        assert!(
            planned
                .survival
                .execution
                .as_ref()
                .is_none_or(|execution| { !matches!(execution, ExecutionCertificate::Plant(_)) })
        );
    }
}

#[test]
fn eastern_hint_is_route_safe_but_rejected_by_the_unchanged_contact_proof() {
    let observed = centered_scene();
    let key = key(&observed);
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observed).unwrap();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let certificate = crate::cpu::routing::plan_lair_ordinary_plant(
        &observed,
        &history,
        crate::cpu::routing::PlantRequest {
            execution_tick: Tick(7_029),
            settled_stop_tick: Tick(7_022),
            first_movement: crate::cpu::MovementIntent::Direction(Direction::South),
            limits: crate::cpu::routing::RouteLimits::default(),
        },
        &mut work,
    )
    .unwrap();
    assert!(MobContactLease::certify_route(&observed, certificate.route(), &mut work).is_err());
}

#[test]
fn exhausted_hint_allowance_preserves_a_nonplant_fallback() {
    let observed = centered_scene();
    let key = key(&observed);
    let memory = north_farm_memory(&observed);
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observed).unwrap();
    let mut work = WorkBudget::new(4_096);
    let planned = plan_farm_refuges_v36_with_origin(
        &observed,
        &memory,
        &history,
        observed.parts().tick,
        None,
        &mut CpuRng::from_seed(key, [139; 32]),
        &mut work,
    )
    .unwrap();
    assert_ne!(
        planned.survival.control.intent.manual,
        Some(ManualIntent::Bomb)
    );
    assert!(work.spent() <= 4_096);
}

#[test]
fn no_lair_v36_and_v37_delegate_to_their_archived_base_policies() {
    let observed = captured_tick_7022()
        .map_parts(|parts| {
            let ObservedActor::Visible(cpu) = &mut parts.actors[CPU.index()] else {
                unreachable!()
            };
            cpu.position = Position::center(CellId::new(3, 8));
        })
        .unwrap();
    let observed = crate::cpu::Observation::new_rules_v10(
        observed.clone().into_parts(),
        observed.environment().unwrap().clone(),
        None,
        observed.spike_pits().cloned(),
    )
    .unwrap();
    let key = key(&observed);
    let memory = north_farm_memory(&observed);
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observed).unwrap();
    for hard in [false, true] {
        let mut old_rng = CpuRng::from_seed(key, [137 + u8::from(hard); 32]);
        let mut new_rng = old_rng.clone();
        let mut old_work = WorkBudget::new(WORK_ALLOWANCE);
        let mut new_work = WorkBudget::new(WORK_ALLOWANCE);
        let old = if hard {
            plan_refuge_guidance_v34_with_origin(
                &observed,
                &memory,
                &history,
                observed.parts().tick,
                None,
                &mut old_rng,
                &mut old_work,
            )
        } else {
            plan_refuge_guidance_v33_with_origin(
                &observed,
                &memory,
                &history,
                observed.parts().tick,
                None,
                &mut old_rng,
                &mut old_work,
            )
        }
        .unwrap();
        let new = if hard {
            plan_hard_farm_refuges_v37_with_origin(
                &observed,
                &memory,
                &history,
                observed.parts().tick,
                None,
                &mut new_rng,
                &mut new_work,
            )
        } else {
            plan_farm_refuges_v36_with_origin(
                &observed,
                &memory,
                &history,
                observed.parts().tick,
                None,
                &mut new_rng,
                &mut new_work,
            )
        }
        .unwrap();
        assert_eq!(new.survival, old.survival);
        assert_eq!(new.contact, old.contact);
        assert_eq!(new_work, old_work);
        assert_eq!(new_rng.draws(), old_rng.draws());
        assert_eq!(new_rng.bounded_u32(1_000), old_rng.bounded_u32(1_000));
    }
}

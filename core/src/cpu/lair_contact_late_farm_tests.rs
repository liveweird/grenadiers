use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{
    StrategicIntent, StrategicObjectiveKind, StrategicPhase, StrategicSwitchReason,
    StrategicTarget, UtilityMemory,
};
use crate::cpu::routing::{
    PlantCertificate, PlantRequest, RouteFailure, RouteLimits, plan_lair_ordinary_plant,
};
use crate::cpu::survival::{
    ExecutionCertificate, WORK_ALLOWANCE, plan_blocked_births_v42_with_origin,
    plan_contact_topology_v40_with_origin, plan_hard_blocked_births_v43_with_origin,
    plan_hard_contact_topology_v41_with_origin,
};
use crate::cpu::{CpuRng, MemoryKey, MobContactLease, MovementIntent, WorkBudget};
use crate::cpu::{Observation, ObservedRespawnLairs};
use crate::game::{CellId, Direction, EntityId, LairStatusView, MobView, Position, Tick};

use super::late_farm_fixture::{
    APPROACH, CPU, TARGET, captured_tick_10741, controlled_physical_state, input,
};
use super::{MobContactEnvelope, PlantContactProof, SourceKind};

fn certified_plant() -> (crate::cpu::Observation, PlantCertificate, u32) {
    let observation = captured_tick_10741();
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observation).unwrap();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let plant = plan_lair_ordinary_plant(
        &observation,
        &history,
        PlantRequest {
            execution_tick: Tick(10_748),
            settled_stop_tick: Tick(10_741),
            first_movement: MovementIntent::Direction(Direction::South),
            limits: RouteLimits::default(),
        },
        &mut work,
    )
    .expect("the exact captured pose has a complete ordinary plant route");
    (observation, plant, work.spent())
}

fn stalled_memory(observation: &crate::cpu::Observation) -> UtilityMemory {
    UtilityMemory::new(MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    })
    .with_strategic_intent(StrategicIntent {
        objective: StrategicObjectiveKind::Farm,
        target: StrategicTarget::Block { cell: TARGET },
        approach_cell: Some(APPROACH),
        phase: StrategicPhase::Approach,
        last_switch_reason: StrategicSwitchReason::Stalled,
        observation_tick: Tick(10_732),
        score: 2,
        committed_until: Tick(10_852),
        progress_deadline: Tick(10_912),
        last_distance: 0,
        last_cpu_position: Position { x: 3_584, y: 6_974 },
        deferred_until: None,
    })
}

#[test]
fn captured_blocked_births_are_the_exact_topology_refusal() {
    let (observation, plant, route_work) = certified_plant();
    assert_eq!(plant.placement().cell, APPROACH);
    assert_eq!(plant.placement().placed_at, Tick(10_748));
    assert_eq!(plant.placement().detonates_at, Tick(10_928));
    assert_eq!(plant.route().survive_until(), Tick(10_958));
    assert_eq!(route_work, 15_557);

    let mut envelope_work = WorkBudget::new(WORK_ALLOWANCE);
    let envelope = MobContactEnvelope::new(&observation, &mut envelope_work).unwrap();
    assert_eq!(envelope.sources().len(), 8);
    assert!(envelope.sources().iter().all(|source| {
        matches!(source.kind, SourceKind::PossibleSpawn { .. })
            && observation
                .parts()
                .arena
                .blocks
                .contains(&source.origin.cell().unwrap())
    }));
    assert!(
        !envelope
            .sources()
            .iter()
            .any(|source| source.origin.cell() == Some(TARGET))
    );
    assert_eq!(
        crate::cpu::lair_contact::blocked_birth_binding_for_test(&observation, 8),
        (true, 8)
    );
    assert_eq!(
        crate::cpu::lair_contact::blocked_birth_binding_for_test(&observation, 7),
        (false, 0)
    );

    let mut lease_work = WorkBudget::new(WORK_ALLOWANCE);
    assert_eq!(
        MobContactLease::certify_route(&observation, plant.route(), &mut lease_work),
        Err(RouteFailure::NoRoute)
    );
    assert_eq!(lease_work.spent(), 552);
    let mut topology_work = WorkBudget::new(WORK_ALLOWANCE);
    assert!(PlantContactProof::certify(&observation, &plant, &mut topology_work).is_none());
    assert_eq!(topology_work.spent(), 1_222);
    let mut blocked_work = WorkBudget::new(WORK_ALLOWANCE);
    let proof =
        PlantContactProof::certify_blocked_births(&observation, &plant, &mut blocked_work).unwrap();
    assert!(proof.requires_blocked_birth_support());
    assert_eq!(blocked_work.spent(), 1_222);
    assert!(
        PlantContactProof::certify_blocked_births(
            &observation,
            &plant,
            &mut WorkBudget::new(1_221),
        )
        .is_none()
    );
}

#[test]
fn new_planners_bind_blocked_birth_authority_inside_the_real_candidate_child() {
    let observation = captured_tick_10741();
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observation).unwrap();
    for (hard, seed) in [(false, 42), (true, 43)] {
        let mut work = WorkBudget::new(WORK_ALLOWANCE);
        let plan = if hard {
            plan_hard_blocked_births_v43_with_origin(
                &observation,
                &UtilityMemory::new(key),
                &history,
                observation.parts().tick,
                None,
                &mut CpuRng::from_seed(key, [seed; 32]),
                &mut work,
            )
        } else {
            plan_blocked_births_v42_with_origin(
                &observation,
                &UtilityMemory::new(key),
                &history,
                observation.parts().tick,
                None,
                &mut CpuRng::from_seed(key, [seed; 32]),
                &mut work,
            )
        }
        .unwrap();
        let execution = plan.survival.execution.unwrap();
        assert_eq!(
            execution.contact_plant_requires_blocked_birth_support(),
            Some(true)
        );
        let ExecutionCertificate::ContactPlant(contact) = execution else {
            panic!("new policy must retain typed contact authority")
        };
        assert_eq!(contact.plant().placement().cell, APPROACH);
        eprintln!("hard={hard} blocked-birth planner work={}", work.spent());
        assert_eq!(work.spent(), if hard { 29_206 } else { 30_686 });
    }
}

#[test]
fn empty_source_authority_renews_exactly_and_fails_closed_on_topology_change() {
    let (observation, plant, _) = certified_plant();
    let mut construction = WorkBudget::new(WORK_ALLOWANCE);
    let proof =
        PlantContactProof::certify_blocked_births(&observation, &plant, &mut construction).unwrap();
    let mut renewal = WorkBudget::new(crate::cpu::lair_contact::PLANT_CONTACT_RENEWAL_ALLOWANCE);
    assert!(proof.observation_fits(&observation, &mut renewal));
    let exact = renewal.spent();
    assert!(exact <= crate::cpu::lair_contact::PLANT_CONTACT_RENEWAL_ALLOWANCE);
    assert!(proof.observation_fits(&observation, &mut WorkBudget::new(exact)));
    assert!(!proof.observation_fits(&observation, &mut WorkBudget::new(exact.saturating_sub(1)),));

    for removed in [CellId::new(5, 6), CellId::new(4, 5)] {
        let changed = observation
            .clone()
            .map_parts(|parts| parts.arena.blocks.retain(|cell| *cell != removed))
            .unwrap();
        assert!(!proof.observation_fits(
            &changed,
            &mut WorkBudget::new(crate::cpu::lair_contact::PLANT_CONTACT_RENEWAL_ALLOWANCE),
        ));
    }

    let mut statuses = observation.respawn_lairs().unwrap().lairs().to_vec();
    statuses[0] = LairStatusView {
        cell: statuses[0].cell,
        next_spawn_at: None,
        living_mobs: 1,
    };
    let future_source = Observation::new_rules_v10(
        observation.parts().clone(),
        observation.environment().unwrap().clone(),
        Some(ObservedRespawnLairs::new(
            observation.respawn_lairs().unwrap().config(),
            statuses,
            vec![MobView {
                id: EntityId(99),
                lair: 0,
                position: Position::center(CellId::new(11, 11)),
                heading: Direction::West,
                frozen: false,
            }],
        )),
        observation.spike_pits().cloned(),
    )
    .unwrap();
    assert!(!proof.observation_fits(
        &future_source,
        &mut WorkBudget::new(crate::cpu::lair_contact::PLANT_CONTACT_RENEWAL_ALLOWANCE),
    ));
}

#[test]
fn archived_contact_planners_decline_the_exact_stalled_farm_intent() {
    let observation = captured_tick_10741();
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observation).unwrap();
    for (hard, seed) in [(false, 40), (true, 41)] {
        let mut work = WorkBudget::new(WORK_ALLOWANCE);
        let plan = if hard {
            plan_hard_contact_topology_v41_with_origin(
                &observation,
                &stalled_memory(&observation),
                &history,
                observation.parts().tick,
                None,
                &mut CpuRng::from_seed(key, [seed; 32]),
                &mut work,
            )
        } else {
            plan_contact_topology_v40_with_origin(
                &observation,
                &stalled_memory(&observation),
                &history,
                observation.parts().tick,
                None,
                &mut CpuRng::from_seed(key, [seed; 32]),
                &mut work,
            )
        }
        .unwrap();
        eprintln!(
            "hard={hard} work={} movement={:?} execution={} failed_target={:?} failed_approach={:?}",
            work.spent(),
            plan.survival.control.intent.movement,
            plan.survival.execution.is_some(),
            plan.survival.control.next_memory.failed_target(),
            plan.survival.control.next_memory.failed_farm_approach(),
        );
        assert!(plan.survival.execution.is_none());
        assert_eq!(work.spent(), if hard { 50_792 } else { 51_682 });
    }
    for (hard, seed) in [(false, 40), (true, 41)] {
        let mut work = WorkBudget::new(WORK_ALLOWANCE);
        let fresh = UtilityMemory::new(key);
        let plan = if hard {
            plan_hard_contact_topology_v41_with_origin(
                &observation,
                &fresh,
                &history,
                observation.parts().tick,
                None,
                &mut CpuRng::from_seed(key, [seed; 32]),
                &mut work,
            )
        } else {
            plan_contact_topology_v40_with_origin(
                &observation,
                &fresh,
                &history,
                observation.parts().tick,
                None,
                &mut CpuRng::from_seed(key, [seed; 32]),
                &mut work,
            )
        }
        .unwrap();
        eprintln!(
            "fresh hard={hard} work={} movement={:?} execution={} intent={:?}",
            work.spent(),
            plan.survival.control.intent.movement,
            plan.survival.execution.is_some(),
            plan.survival.control.next_memory.strategic_intent(),
        );
        assert!(plan.survival.execution.is_none());
        assert_eq!(work.spent(), if hard { 50_792 } else { 51_682 });
        let intent = plan
            .survival
            .control
            .next_memory
            .strategic_intent()
            .unwrap();
        assert_eq!(intent.target, StrategicTarget::Block { cell: TARGET });
        assert_eq!(intent.approach_cell, Some(APPROACH));
    }
}

#[test]
fn certified_ordinary_plant_physically_destroys_target_and_escapes() {
    let (_, plant, _) = certified_plant();
    let mut state = controlled_physical_state();
    let lane = plant
        .route()
        .lane(plant.route().start_window().start)
        .unwrap();
    let mut planted = false;
    while state.tick() < plant.route().survive_until() {
        let update = Tick(state.tick().0 + 1);
        let expected =
            (update < plant.route().survive_until()).then(|| lane.pose_at(update).unwrap());
        let current = state.actor(CPU).unwrap().position();
        let target = expected.map_or(current, |pose| pose.0);
        let movement = match (target.x.cmp(&current.x), target.y.cmp(&current.y)) {
            (std::cmp::Ordering::Less, _) => Some(Direction::West),
            (std::cmp::Ordering::Greater, _) => Some(Direction::East),
            (_, std::cmp::Ordering::Less) => Some(Direction::North),
            (_, std::cmp::Ordering::Greater) => Some(Direction::South),
            _ => None,
        };
        state
            .step(&input(movement, update == plant.placement().placed_at))
            .unwrap();
        let actor = state.actor(CPU).unwrap();
        assert!(actor.alive());
        if let Some(expected) = expected {
            assert_eq!((actor.position(), actor.heading()), expected);
        }
        planted |= state.live_bomb_count(CPU) == 1;
    }
    assert!(planted);
    assert!(state.actor(CPU).unwrap().alive());
    assert!(!state.world().arena().blocks().contains(&TARGET));
    assert_eq!(state.live_bomb_count(CPU), 0);
}

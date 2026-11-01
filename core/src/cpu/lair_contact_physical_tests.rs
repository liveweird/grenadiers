use std::collections::BTreeSet;
use std::ops::ControlFlow;

use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::routing::{PlantRequest, RouteFailure, RouteLimits, plan_lair_ordinary_plant};
use crate::cpu::survival::{
    CommittedRoute, ExecutionCertificate, ObservedBlastGeometry, RouteStatus, WORK_ALLOWANCE,
    plan_contact_topology_v40_with_origin, plan_farm_origins_v38_with_origin,
    plan_hard_contact_topology_v41_with_origin,
};
use crate::cpu::{CpuRng, MemoryKey, MobContactLease, MovementIntent, WorkBudget, WorldId};
use crate::game::blast_geometry::{BlastSource, visit_blast};
use crate::game::{BlastKind, CellId, Direction, Tick};
use crate::room::RespawnLairSettings;

use super::connectivity_tests::{IsolationEvidence, captured_blocks};
use super::physical_fixture::{CPU, MOB_POCKET, ORIGIN, TARGET, controlled_state, input, memory};

#[test]
fn natural_trapped_mob_has_a_full_plant_route_but_old_contact_rejects_it() {
    let (mut state, settings) = controlled_state();
    for _ in 0..RespawnLairSettings::DEFAULT.spawn_ticks() {
        state.step(&[]).unwrap();
    }
    assert!(!state.mobs().is_empty());
    assert!(
        state
            .mobs()
            .iter()
            .all(|mob| mob.position.cell() == Some(MOB_POCKET))
    );
    let observed =
        crate::cpu::survival::observe_with_lairs(&state, (settings, RespawnLairSettings::DEFAULT));
    assert_eq!(
        observed
            .parts()
            .arena
            .blocks
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        captured_blocks()
    );
    assert!(observed.parts().pickups.is_empty());
    assert_eq!(observed.parts().tick, Tick(600));
    let key = MemoryKey {
        world: WorldId {
            room_instance: 27,
            physical_round: 1,
        },
        cpu_seat: CPU,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observed).unwrap();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let certificate = plan_lair_ordinary_plant(
        &observed,
        &history,
        PlantRequest {
            execution_tick: Tick(607),
            settled_stop_tick: Tick(600),
            first_movement: MovementIntent::Direction(Direction::East),
            limits: RouteLimits::default(),
        },
        &mut work,
    )
    .expect("unchanged route proof should certify the controlled east escape");
    assert_eq!(certificate.placement().cell, ORIGIN);
    assert_eq!(certificate.placement().placed_at, Tick(607));
    assert_eq!(certificate.placement().blast, BlastKind::Standard);
    assert_eq!(certificate.route().survive_until(), Tick(817));
    assert_eq!(
        MobContactLease::certify_route(&observed, certificate.route(), &mut work),
        Err(RouteFailure::NoRoute)
    );
    let mut proof_work = WorkBudget::new(WORK_ALLOWANCE);
    let proof = crate::cpu::lair_contact::PlantContactProof::certify(
        &observed,
        &certificate,
        &mut proof_work,
    );
    assert!(proof.is_some());

    let mut planner_work = WorkBudget::new(WORK_ALLOWANCE);
    let full = plan_farm_origins_v38_with_origin(
        &observed,
        &memory(&observed),
        &history,
        observed.parts().tick,
        None,
        &mut CpuRng::from_seed(key, [38; 32]),
        &mut planner_work,
    )
    .unwrap();
    assert!(full.survival.execution.is_none());

    let mut topology_work = WorkBudget::new(WORK_ALLOWANCE);
    let topology = plan_contact_topology_v40_with_origin(
        &observed,
        &memory(&observed),
        &history,
        observed.parts().tick,
        None,
        &mut CpuRng::from_seed(key, [40; 32]),
        &mut topology_work,
    )
    .unwrap();
    let ExecutionCertificate::ContactPlant(topology_certificate) =
        topology.survival.execution.as_ref().unwrap()
    else {
        panic!("v40 must bind the full plant to topology contact authority");
    };
    assert_eq!(
        topology_certificate.plant().placement(),
        certificate.placement()
    );
    assert_eq!(
        topology_certificate.plant().route().survive_until(),
        certificate.route().survive_until()
    );
    let admitted_execution = topology.survival.execution.clone().unwrap();
    assert!(topology_work.spent() <= WORK_ALLOWANCE);

    let mut hard_work = WorkBudget::new(WORK_ALLOWANCE);
    let hard = plan_hard_contact_topology_v41_with_origin(
        &observed,
        &memory(&observed),
        &history,
        observed.parts().tick,
        None,
        &mut CpuRng::from_seed(key, [41; 32]),
        &mut hard_work,
    )
    .unwrap();
    assert!(matches!(
        hard.survival.execution,
        Some(ExecutionCertificate::ContactPlant(_))
    ));
    assert!(hard_work.spent() <= WORK_ALLOWANCE);
    let certificate = topology_certificate.plant().clone();

    let mut geometry_work = WorkBudget::new(WORK_ALLOWANCE);
    let geometry = ObservedBlastGeometry::new(&observed, &mut geometry_work).unwrap();
    let mut opened = BTreeSet::new();
    let _ = visit_blast(
        &geometry,
        BlastSource {
            cell: certificate.placement().cell,
            range: certificate.placement().range,
            kind: BlastKind::Standard,
        },
        |visit| {
            if geometry_work.try_take(1).is_err() {
                return ControlFlow::Break(());
            }
            if visit.soft_block {
                opened.insert(visit.cell);
            }
            ControlFlow::Continue(())
        },
    );
    assert_eq!(opened, BTreeSet::from([TARGET]));
    let sources = observed
        .respawn_lairs()
        .unwrap()
        .mobs()
        .iter()
        .filter_map(|mob| mob.position.cell())
        .collect::<Vec<_>>();
    let evidence = IsolationEvidence::build(
        &observed.parts().arena.blocks.iter().copied().collect(),
        &opened,
        &sources,
        &mut geometry_work,
    )
    .unwrap();
    let mut queried_snapshots = 0_u64;
    for first in
        certificate.route().start_window().start.0..certificate.route().start_window().end.0
    {
        let lane = certificate.route().lane(Tick(first)).unwrap();
        let mut previous = lane.pose_at(observed.parts().tick).unwrap().0;
        for tick in observed.parts().tick.0 + 1..certificate.route().survive_until().0 {
            let current = lane.pose_at(Tick(tick)).unwrap().0;
            assert!(evidence.excludes_segment(
                crate::game::MovementSegment::new(previous, current),
                &mut geometry_work,
            ));
            queried_snapshots += 1;
            previous = current;
        }
    }
    let lane_count =
        certificate.route().start_window().end.0 - certificate.route().start_window().start.0;
    assert_eq!(
        queried_snapshots,
        lane_count * (certificate.route().survive_until().0 - observed.parts().tick.0 - 1)
    );
    assert!(work.spent() <= WORK_ALLOWANCE);
    assert!(geometry_work.spent() <= WORK_ALLOWANCE);
    eprintln!(
        "contact topology witness: placement={} survive={} route_work={} proof_work={} planner_work={} topology_work={} hard_work={} geometry_work={}",
        certificate.placement().placed_at.0,
        certificate.route().survive_until().0,
        work.spent(),
        proof_work.spent(),
        planner_work.spent(),
        topology_work.spent(),
        hard_work.spent(),
        geometry_work.spent(),
    );

    let first = certificate.route().start_window().start;
    let lane = certificate.route().lane(first).unwrap();
    let mut saw_placement = false;
    while state.tick().0 + 1 < first.0 {
        state.step(&[]).unwrap();
    }
    let admission =
        crate::cpu::survival::observe_with_lairs(&state, (settings, RespawnLairSettings::DEFAULT));
    let mut committed = CommittedRoute::admit_execution(
        admitted_execution,
        observed.clone(),
        &admission,
        admission.parts().tick,
        MovementIntent::Stop,
    )
    .unwrap();
    assert!(committed.has_contact_authority());
    let mut accepted = MovementIntent::Stop;
    let mut settled_stop = Some(admission.parts().tick);
    while state.tick() < certificate.route().survive_until() {
        match committed.poll_command(state.tick(), settled_stop) {
            RouteStatus::Submit(command) => {
                assert!(committed.commit_command(command));
                accepted = command.movement();
                if accepted != MovementIntent::Stop {
                    settled_stop = None;
                }
            }
            RouteStatus::Invalidated => panic!("contact route invalidated before its next update"),
            _ => {}
        }
        let update = Tick(state.tick().0 + 1);
        let movement = match accepted {
            MovementIntent::Direction(direction) => Some(direction),
            MovementIntent::Stop => None,
        };
        state
            .step(&input(
                movement,
                update == certificate.placement().placed_at,
            ))
            .unwrap();
        if accepted == MovementIntent::Stop {
            settled_stop = Some(state.tick());
        }
        let actor = state.actor(CPU).unwrap();
        assert!(actor.alive());
        if state.tick() < certificate.route().survive_until() {
            let (expected_position, expected_heading) = lane.pose_at(state.tick()).unwrap();
            assert_eq!(actor.position(), expected_position);
            assert_eq!(actor.heading(), expected_heading);
        }
        if state.tick() >= certificate.placement().placed_at
            && state.tick() < certificate.placement().detonates_at
        {
            let actual = state
                .public_view()
                .bombs
                .into_iter()
                .find(|bomb| bomb.owner == certificate.placement().owner)
                .expect("planned bomb must be visible throughout its live interval");
            assert_eq!(actual.cell, certificate.placement().cell);
            assert_eq!(actual.placed_at, certificate.placement().placed_at);
            assert_eq!(actual.owner, certificate.placement().owner);
            assert_eq!(actual.range, certificate.placement().range);
            assert_eq!(actual.blast, certificate.placement().blast);
            saw_placement = true;
        }
        let publication = crate::cpu::survival::observe_with_lairs(
            &state,
            (settings, RespawnLairSettings::DEFAULT),
        );
        let status = committed.observe_with_stop(&publication, settled_stop);
        if state.tick().0 + 1 == certificate.route().survive_until().0 {
            assert_eq!(status, RouteStatus::Invalidated);
            break;
        }
        assert_ne!(
            status,
            RouteStatus::Invalidated,
            "contact renewal invalidated at tick {}",
            state.tick().0
        );
    }
    assert_eq!(state.tick().0 + 1, certificate.route().survive_until().0);
    state.step(&input(None, false)).unwrap();
    assert!(state.actor(CPU).unwrap().alive());
    assert!(saw_placement);
    assert!(!state.world().arena.blocks().contains(&TARGET));
    assert_eq!(state.live_bomb_count(CPU), 0);
    assert!(state.world().hazards().is_empty());
    assert!(
        state
            .mobs()
            .iter()
            .all(|mob| mob.position.cell() == Some(MOB_POCKET))
    );
    for barrier in [
        CellId::new(7, 5),
        CellId::new(7, 7),
        CellId::new(8, 5),
        CellId::new(8, 7),
    ] {
        assert!(state.world().arena.blocks().contains(&barrier));
    }
}

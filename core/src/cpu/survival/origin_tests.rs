use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::UtilityMemory;
use crate::cpu::routing::plant_tests::current_scene;
use crate::cpu::routing::tests::{key, observation};
use crate::cpu::routing::{
    PlantPresence, PlantRequest, RouteLimits, RouteStep, plan_ordinary_plant,
};
use crate::cpu::{BombKnowledge, CpuRng, MovementIntent, ObservedActor, ObservedOil, WorkBudget};
use crate::game::{Direction, EntityId, OilBarrelState, Tick};

use super::{
    CommittedPulse, CommittedRoute, ExecutionCertificate, PlantEscapeOrigin, RouteStatus,
    plan_with_origin,
};

fn stationary_at(observation: &crate::cpu::Observation, tick: Tick) -> crate::cpu::Observation {
    let mut parts = observation.clone().into_parts();
    let elapsed = tick.0 - parts.tick.0;
    parts.tick = tick;
    parts.rules.running_updates += u16::try_from(elapsed).unwrap();
    parts.rules.remaining_ticks -= u16::try_from(elapsed).unwrap();
    let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
        unreachable!()
    };
    let crate::cpu::ObservedBombTrigger::Timed {
        remaining_ticks, ..
    } = &mut bombs[0].trigger
    else {
        unreachable!()
    };
    *remaining_ticks -= u16::try_from(elapsed).unwrap();
    crate::cpu::Observation::new(parts).unwrap()
}

fn confirmed_origin() -> (
    crate::cpu::Observation,
    crate::cpu::routing::PlantCertificate,
    crate::cpu::Observation,
    PlantEscapeOrigin,
) {
    let basis = observation(Vec::new(), Vec::new());
    let certificate = plan_ordinary_plant(
        &basis,
        &ObservedHazardMemory::new(key()),
        PlantRequest {
            execution_tick: Tick(7),
            settled_stop_tick: Tick(0),
            first_movement: MovementIntent::Direction(Direction::East),
            limits: RouteLimits::default(),
        },
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    let first = match certificate.route().steps().first().unwrap() {
        RouteStep::Move { direction, .. } => MovementIntent::Direction(*direction),
        RouteStep::Wait { .. } => MovementIntent::Stop,
    };
    let awaiting = current_scene(&basis, &certificate, Tick(6), None);
    let mut committed = CommittedRoute::admit_execution(
        ExecutionCertificate::Plant(Box::new(certificate.clone())),
        basis.clone(),
        &awaiting,
        Tick(6),
        first,
    )
    .unwrap();
    assert_eq!(committed.plant_presence(), Some(PlantPresence::Awaiting));
    assert!(committed.escape_origin(&awaiting).is_none());
    let present = current_scene(&basis, &certificate, Tick(7), Some((EntityId(700), 2)));
    assert_ne!(committed.observe(&present), RouteStatus::Invalidated);
    assert_eq!(
        committed.plant_presence(),
        Some(PlantPresence::Present(EntityId(700)))
    );
    let origin = committed.escape_origin(&present).expect("confirmed origin");
    (basis, certificate, present, origin)
}

#[test]
fn origin_requires_confirmed_presence_and_exact_expected_pose() {
    let (basis, certificate, present, origin) = confirmed_origin();
    assert!(origin.retain_stopped(&present).is_some());

    let mut moved = present.clone().into_parts();
    let ObservedActor::Visible(actor) = &mut moved.actors[1] else {
        unreachable!()
    };
    actor.position.x += 1;
    assert!(
        origin
            .retain_stopped(&crate::cpu::Observation::new(moved).unwrap())
            .is_none()
    );

    let absent = current_scene(&basis, &certificate, Tick(7), None);
    assert!(PlantEscapeOrigin::new(basis, certificate, EntityId(700), &absent).is_none());
}

#[test]
fn origin_rejects_rewind_actor_or_bomb_signature_changes() {
    let (_, certificate, present, origin) = confirmed_origin();
    let mut cases = Vec::new();

    let mut actor_changed = present.clone().into_parts();
    let ObservedActor::Visible(actor) = &mut actor_changed.actors[1] else {
        unreachable!()
    };
    actor.bomb_range += 1;
    cases.push(crate::cpu::Observation::new(actor_changed).unwrap());

    let mut bomb_changed = present.clone().into_parts();
    let BombKnowledge::Visible(bombs) = &mut bomb_changed.bombs else {
        unreachable!()
    };
    bombs[0].range += 1;
    cases.push(crate::cpu::Observation::new(bomb_changed).unwrap());

    let mut unavailable = present.clone().into_parts();
    unavailable.bombs = BombKnowledge::Unavailable;
    cases.push(crate::cpu::Observation::new(unavailable).unwrap());

    let mut blocked = present.clone().into_parts();
    blocked.arena.blocks.push(certificate.placement().cell);
    cases.push(crate::cpu::Observation::new(blocked).unwrap());

    let mut oil = present.clone().into_parts();
    oil.oil.push(ObservedOil {
        cell: certificate.placement().cell,
        state: OilBarrelState::Unlit,
    });
    cases.push(crate::cpu::Observation::new(oil).unwrap());

    for changed in cases {
        assert!(origin.retain_stopped(&changed).is_none());
    }
    let mut later = present.clone().into_parts();
    later.tick = Tick(8);
    later.rules.running_updates += 1;
    later.rules.remaining_ticks -= 1;
    let BombKnowledge::Visible(bombs) = &mut later.bombs else {
        unreachable!()
    };
    let crate::cpu::ObservedBombTrigger::Timed {
        remaining_ticks, ..
    } = &mut bombs[0].trigger
    else {
        unreachable!()
    };
    *remaining_ticks -= 1;
    let advanced = origin
        .retain_stopped(&crate::cpu::Observation::new(later).unwrap())
        .expect("monotonic checkpoint");
    assert!(advanced.retain_stopped(&present).is_none());
}

#[test]
fn origin_planning_is_bounded_and_never_emits_manual_or_planting() {
    let (_, _, present, origin) = confirmed_origin();
    let key = key();
    let memory = UtilityMemory::new(key);
    let history = ObservedHazardMemory::new(key);
    let mut rng = CpuRng::from_seed(key, [9; 32]);
    let plan = plan_with_origin(
        &present,
        &memory,
        &history,
        present.parts().tick,
        Some(&origin),
        &mut rng,
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert!(plan.control.intent.manual.is_none());
    assert!(plan.execution.is_none());
    let lease = plan.lease.expect("origin exit pulse");
    assert_eq!(lease.start_window().start, Tick(8));
    assert_eq!(lease.start_window().end, Tick(15));
    assert!(lease.hold_ticks() <= 3);

    for admission_tick in 7..14 {
        let current = stationary_at(&present, Tick(admission_tick));
        let committed = CommittedPulse::admit(
            lease.clone(),
            present.clone(),
            &current,
            Tick(admission_tick),
        )
        .expect("each delayed lane admits");
        assert!(
            committed
                .retain_escape_origin(&origin, &current, Tick(admission_tick))
                .is_some()
        );
    }

    let latest = CommittedPulse::admit(
        lease.clone(),
        present.clone(),
        &stationary_at(&present, Tick(13)),
        Tick(13),
    )
    .unwrap();
    let terminal_tick = Tick(latest.first_execution().0 + u64::from(lease.hold_ticks()) + 2);
    let mut terminal = stationary_at(&present, terminal_tick).into_parts();
    let ObservedActor::Visible(actor) = &mut terminal.actors[1] else {
        unreachable!()
    };
    actor.position = lease.pose_after(lease.hold_ticks()).min();
    actor.heading = match lease.movement() {
        MovementIntent::Direction(direction) => direction,
        MovementIntent::Stop => actor.heading,
    };
    let terminal = crate::cpu::Observation::new(terminal).unwrap();
    assert!(
        latest
            .retain_escape_origin(&origin, &terminal, terminal_tick)
            .is_some(),
        "latest lane retains provenance at its first post-Stop snapshot"
    );

    let stopped = plan_with_origin(
        &present,
        &memory,
        &history,
        present.parts().tick,
        Some(&origin),
        &mut rng,
        &mut WorkBudget::new(1),
    )
    .unwrap();
    assert_eq!(stopped.control.intent.movement, MovementIntent::Stop);
    assert!(stopped.lease.is_none());
}

#[test]
fn lethal_origin_and_changed_trigger_never_receive_grace() {
    let (_, certificate, present, origin) = confirmed_origin();
    let mut paused = present.clone().into_parts();
    let BombKnowledge::Visible(bombs) = &mut paused.bombs else {
        unreachable!()
    };
    let crate::cpu::ObservedBombTrigger::Timed {
        paused: trigger_paused,
        ..
    } = &mut bombs[0].trigger
    else {
        unreachable!()
    };
    *trigger_paused = true;
    assert!(
        origin
            .retain_stopped(&crate::cpu::Observation::new(paused).unwrap())
            .is_none()
    );

    let mut lethal = present.clone().into_parts();
    lethal.flames.push(crate::cpu::ObservedFlame {
        cell: certificate.placement().cell,
        remaining_ticks: 20,
    });
    let lethal = crate::cpu::Observation::new(lethal).unwrap();
    let key = key();
    let result = plan_with_origin(
        &lethal,
        &UtilityMemory::new(key),
        &ObservedHazardMemory::new(key),
        lethal.parts().tick,
        Some(&origin),
        &mut CpuRng::from_seed(key, [4; 32]),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert_eq!(result.control.intent.movement, MovementIntent::Stop);
    assert!(result.lease.is_none());
}

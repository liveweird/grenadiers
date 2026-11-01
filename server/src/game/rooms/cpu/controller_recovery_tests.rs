use grenadiers_core::cpu::{
    BombKnowledge, Intent, ManualIntent, Observation, ObservedActor, ObservedBomb,
    ObservedBombTrigger, UnknownBombFacts,
};
use grenadiers_core::game::{BlastKind, EntityId, SeatId, Tick};

use super::CpuController;
use super::plant_test_support::advanced;
use super::plant_tests::{PlantFixture, inject_certified, placement_observation, prepared};

#[test]
fn confirmed_plant_invalidation_stops_then_admits_a_fresh_recovery_pulse() {
    let (mut controller, stopped) = stopped_recovery_controller();
    assert!(controller.submit(stopped, 4));
    let report = controller.planner.pump_for_test(1);
    assert_eq!(report.delivered, 1);
    let accepted = controller.poll(Tick(31), 4).expect("fresh recovery plan");
    assert!(accepted.intent.manual.is_none());
    assert!(accepted.committed_pulse.is_some());
    let intent = accepted.intent;
    controller.on_movement_admitted();
    controller.commit(accepted, intent);
    assert_eq!(controller.recovery_state(), (true, true));
}

#[test]
fn stale_result_does_not_restore_or_revoke_a_valid_stopped_origin() {
    let (mut controller, stopped) = stopped_recovery_controller();
    assert!(controller.submit(stopped, 4));
    assert_eq!(controller.planner.pump_for_test(1).delivered, 1);

    assert!(controller.poll(Tick(38), 4).is_none());
    assert!(!controller.has_pending());
    assert_eq!(controller.recovery_state(), (true, false));
    assert_eq!(controller.rng_draws(), 0);
}

#[test]
fn epoch_or_world_change_revokes_recovery_provenance() {
    let (mut epoch, _) = stopped_recovery_controller();
    epoch.validate_escape_epoch(5);
    assert_eq!(epoch.recovery_state(), (false, false));

    let (mut world, _) = stopped_recovery_controller();
    world.reset_world(4);
    assert_eq!(world.recovery_state(), (false, false));
}

#[test]
fn unconfirmed_stop_observation_revokes_recovery_provenance() {
    let (mut controller, basis, fixture) = invalidated_recovery_controller();
    controller.on_stop_admitted(Tick(30));
    let mut parts = stopped_observation(&basis, &fixture).into_parts();
    let ObservedActor::Visible(cpu) = &mut parts.actors[SeatId::Two.index()] else {
        unreachable!()
    };
    cpu.voluntary_actions_frozen = true;
    let frozen = Observation::new(parts).unwrap();

    assert!(controller.accept_observation(&frozen));
    assert_eq!(controller.recovery_state(), (false, false));
    assert!(!controller.has_pending());
}

#[test]
fn completed_result_cannot_restore_a_revoked_origin() {
    let (mut controller, stopped) = stopped_recovery_controller();
    assert!(controller.submit(stopped, 4));
    assert_eq!(controller.planner.pump_for_test(1).delivered, 1);
    controller.clear_escape_origin();

    assert!(controller.poll(Tick(31), 4).is_none());
    assert!(!controller.has_pending());
    assert_eq!(controller.recovery_state(), (false, false));
    assert_eq!(controller.rng_draws(), 0);
}

#[test]
fn recovery_movement_without_a_pulse_is_rejected_without_losing_valid_origin() {
    let (mut controller, stopped) = stopped_recovery_controller();
    assert!(controller.submit(stopped, 4));
    assert_eq!(controller.planner.pump_for_test(1).delivered, 1);
    let pending = controller.pending.as_mut().unwrap();
    let mut result = pending.receiver.try_recv().unwrap();
    result.ordinary_mut().lease = None;
    pending.completed = Some(result);

    assert!(controller.poll(Tick(31), 4).is_none());
    assert!(!controller.has_pending());
    assert_eq!(controller.recovery_state(), (true, false));
    assert_eq!(controller.rng_draws(), 0);
}

fn stopped_recovery_controller() -> (CpuController, Observation) {
    let (mut controller, basis, fixture) = invalidated_recovery_controller();
    controller.on_stop_admitted(Tick(30));
    let stopped = stopped_observation(&basis, &fixture);
    assert!(controller.accept_observation(&stopped));
    assert_eq!(controller.recovery_state(), (true, false));
    (controller, stopped)
}

fn invalidated_recovery_controller() -> (CpuController, Observation, PlantFixture) {
    let basis = crate::game::rooms::cpu::tests::observation();
    let mut controller = prepared(&basis);
    let fixture = inject_certified(&mut controller, &basis);
    let exact = advanced(&basis, Tick(26));
    assert!(controller.accept_observation(&exact));
    let accepted = controller.poll(Tick(26), 4).unwrap();
    controller.on_movement_admitted();
    controller.commit(
        accepted,
        Intent {
            movement: fixture.movement,
            manual: Some(ManualIntent::Bomb),
        },
    );
    let placed = placement_observation(&basis, &fixture, Some(EntityId(91)));
    assert!(controller.accept_observation(&placed));
    assert!(controller.observe_route(&placed).is_some());
    let unexpected = recovery_observation(&basis, &fixture, Tick(30));
    assert!(controller.accept_observation(&unexpected));
    assert_eq!(
        controller.observe_route(&unexpected),
        Some(grenadiers_core::cpu::survival::RouteStatus::Invalidated)
    );
    assert_eq!(controller.recovery_state(), (true, false));
    (controller, basis, fixture)
}

fn stopped_observation(basis: &Observation, fixture: &PlantFixture) -> Observation {
    let invalidation = recovery_observation(basis, fixture, Tick(30));
    let mut parts = recovery_observation(basis, fixture, Tick(31)).into_parts();
    let ObservedActor::Visible(previous) = &invalidation.parts().actors[SeatId::Two.index()] else {
        unreachable!()
    };
    let ObservedActor::Visible(current) = &mut parts.actors[SeatId::Two.index()] else {
        unreachable!()
    };
    current.position = previous.position;
    current.heading = previous.heading;
    Observation::new(parts).unwrap()
}

fn recovery_observation(basis: &Observation, fixture: &PlantFixture, tick: Tick) -> Observation {
    let mut parts = placement_observation(basis, fixture, Some(EntityId(91))).into_parts();
    let elapsed = u16::try_from(tick.0 - parts.tick.0).unwrap();
    parts.tick = tick;
    parts.rules.running_updates += elapsed;
    parts.rules.remaining_ticks -= elapsed;
    let ObservedActor::Visible(cpu) = &mut parts.actors[SeatId::Two.index()] else {
        unreachable!()
    };
    let (position, heading) = fixture
        .route
        .lane(Tick(27))
        .and_then(|lane| lane.pose_at(tick))
        .unwrap();
    cpu.position = position;
    cpu.heading = heading;
    for effect in &mut cpu.effects {
        effect.remaining_ticks = effect.remaining_ticks.saturating_sub(elapsed);
    }
    cpu.effects.retain(|effect| effect.remaining_ticks > 0);
    let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
        unreachable!()
    };
    let ObservedBombTrigger::Timed {
        remaining_ticks,
        paused: _,
    } = &mut bombs[0].trigger
    else {
        unreachable!()
    };
    *remaining_ticks = remaining_ticks.saturating_sub(elapsed);
    bombs.push(ObservedBomb {
        id: EntityId(92),
        placed_at: Tick(tick.0 - 1),
        cell: grenadiers_core::game::CellId::new(1, 1),
        owner: EntityId(1),
        range: 2,
        trigger: ObservedBombTrigger::Timed {
            remaining_ticks: 60,
            paused: false,
        },
        blast: BlastKind::Standard,
        unknown: UnknownBombFacts::ALL,
    });
    Observation::new(parts).unwrap()
}

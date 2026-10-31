use crate::cpu::policy::tests::{key, observation};
use crate::cpu::{Observation, ObservedActor, ObservedEffect, ObservedPickup, WorkBudget};
use crate::game::{CellId, EffectKind, EntityId, PickupKind, Position, SeatId, Tick};

use super::*;

fn choose(observation: &Observation, memory: UtilityMemory) -> GuidanceV24 {
    let actor = observation
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
    select(
        observation,
        actor,
        actor.position.cell().unwrap(),
        &memory,
        &mut WorkBudget::new(65_536),
    )
    .unwrap()
}

fn with_pickups(tick: u64) -> Observation {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    parts.tick = Tick(tick);
    parts.pickups.extend([
        ObservedPickup {
            cell: CellId::new(5, 7),
            kind: PickupKind::MovementSpeed,
        },
        ObservedPickup {
            cell: CellId::new(9, 7),
            kind: PickupKind::BombCapacity,
        },
    ]);
    Observation::new(parts).unwrap()
}

#[test]
fn exact_pickup_is_retained_across_equal_or_small_utility_changes() {
    let first = with_pickups(10);
    let retained = StrategicIntent {
        objective: StrategicObjectiveKind::Collect,
        target: StrategicTarget::Pickup {
            cell: CellId::new(5, 7),
            kind: PickupKind::MovementSpeed,
        },
        approach_cell: Some(CellId::new(5, 7)),
        phase: StrategicPhase::Approach,
        last_switch_reason: StrategicSwitchReason::Initial,
        observation_tick: Tick(9),
        score: 6,
        committed_until: Tick(9),
        progress_deadline: Tick(50),
        last_distance: 2,
        last_cpu_position: Position::center(CellId::new(7, 7)),
        deferred_until: None,
    };
    let chosen = choose(
        &first,
        UtilityMemory::new(key()).with_strategic_intent(retained),
    );
    assert_eq!(chosen.intent.target, retained.target);
    assert_eq!(
        chosen.intent.last_switch_reason,
        StrategicSwitchReason::Initial
    );
}

#[test]
fn progress_deadline_switches_at_boundary_and_defers_failed_target_boundedly() {
    let deadline = Tick(STRATEGIC_PROGRESS_TICKS);
    let at_deadline = with_pickups(deadline.0);
    let failed = StrategicIntent {
        objective: StrategicObjectiveKind::Collect,
        target: StrategicTarget::Pickup {
            cell: CellId::new(5, 7),
            kind: PickupKind::MovementSpeed,
        },
        approach_cell: Some(CellId::new(5, 7)),
        phase: StrategicPhase::Approach,
        last_switch_reason: StrategicSwitchReason::Initial,
        observation_tick: Tick(deadline.0 - 1),
        score: 6,
        committed_until: Tick(20),
        progress_deadline: deadline,
        last_distance: 2,
        last_cpu_position: Position::center(CellId::new(7, 7)),
        deferred_until: None,
    };
    let switched = choose(
        &at_deadline,
        UtilityMemory::new(key()).with_strategic_intent(failed),
    );
    assert_ne!(switched.intent.target, failed.target);
    assert_eq!(
        switched.intent.last_switch_reason,
        StrategicSwitchReason::Stalled
    );
    let defer_end = add_ticks(deadline, FAILED_TARGET_DEFER_TICKS);
    assert_eq!(switched.failed_target, Some((failed.target, defer_end)));

    let after_expiry = with_pickups(defer_end.0);
    let memory = UtilityMemory::new(key())
        .with_strategic_intent(switched.intent)
        .with_failed_target(switched.failed_target);
    let chosen = choose(&after_expiry, memory);
    assert!(matches!(
        chosen.intent.target,
        StrategicTarget::Pickup { .. }
    ));
}

#[test]
fn slow_subcell_advance_toward_previous_approach_refreshes_progress_deadline() {
    let deadline = Tick(STRATEGIC_PROGRESS_TICKS);
    let mut parts = with_pickups(deadline.0).into_parts();
    let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
        unreachable!()
    };
    let previous_position = Position::center(CellId::new(7, 7));
    cpu.position = Position {
        x: previous_position.x - 64,
        y: previous_position.y,
    };
    cpu.effects.push(ObservedEffect {
        kind: EffectKind::SlowFoot,
        remaining_ticks: 100,
    });
    assert_eq!(cpu.position.cell(), Some(CellId::new(7, 7)));
    let observation = Observation::new(parts).unwrap();
    let target = StrategicIntent {
        objective: StrategicObjectiveKind::Collect,
        target: StrategicTarget::Pickup {
            cell: CellId::new(5, 7),
            kind: PickupKind::MovementSpeed,
        },
        approach_cell: Some(CellId::new(5, 7)),
        phase: StrategicPhase::Approach,
        last_switch_reason: StrategicSwitchReason::Initial,
        observation_tick: Tick(deadline.0 - 1),
        score: 6,
        committed_until: Tick(0),
        progress_deadline: deadline,
        last_distance: 2,
        last_cpu_position: previous_position,
        deferred_until: None,
    };
    let advanced = choose(
        &observation,
        UtilityMemory::new(key()).with_strategic_intent(target),
    );
    assert_eq!(advanced.intent.target, target.target);
    assert_eq!(advanced.failed_target, None);
    assert_eq!(
        advanced.intent.progress_deadline,
        add_ticks(deadline, STRATEGIC_PROGRESS_TICKS)
    );
}

#[test]
fn expected_long_fuse_wait_suspends_stall_then_reengages_with_a_fresh_deadline() {
    let observation = with_pickups(200);
    let target = StrategicIntent {
        objective: StrategicObjectiveKind::Collect,
        target: StrategicTarget::Pickup {
            cell: CellId::new(5, 7),
            kind: PickupKind::MovementSpeed,
        },
        approach_cell: Some(CellId::new(5, 7)),
        phase: StrategicPhase::Escape,
        last_switch_reason: StrategicSwitchReason::SafetyInterrupt,
        observation_tick: Tick(100),
        score: 6,
        committed_until: Tick(120),
        progress_deadline: Tick(136),
        last_distance: 2,
        last_cpu_position: Position::center(CellId::new(7, 7)),
        deferred_until: None,
    };
    let waiting = choose(
        &observation,
        UtilityMemory::new(key()).with_strategic_intent(target),
    );
    assert_eq!(waiting.intent.target, target.target);
    assert_eq!(waiting.intent.phase, StrategicPhase::Escape);

    let resumed_memory = UtilityMemory::new(key())
        .with_strategic_intent(waiting.intent)
        .mark_strategic_phase(
            StrategicPhase::Waiting,
            StrategicSwitchReason::SafetyInterrupt,
            Tick(200),
        )
        .suspend_strategic_deadline(Tick(200));
    let fuse_wait = choose(&observation, resumed_memory);
    assert_eq!(fuse_wait.intent.target, target.target);
    assert_eq!(fuse_wait.intent.phase, StrategicPhase::Waiting);
    assert_eq!(fuse_wait.first_step, CellId::new(7, 7));

    let resumed_memory = UtilityMemory::new(key())
        .with_strategic_intent(fuse_wait.intent)
        .mark_strategic_phase(
            StrategicPhase::Reengage,
            StrategicSwitchReason::Resumed,
            Tick(200),
        );
    let resumed = choose(&observation, resumed_memory);
    assert_eq!(resumed.intent.target, target.target);
    assert_eq!(resumed.intent.phase, StrategicPhase::Approach);
    assert_eq!(
        resumed.intent.last_switch_reason,
        StrategicSwitchReason::Resumed
    );
    assert_eq!(
        resumed.intent.progress_deadline,
        add_ticks(Tick(200), STRATEGIC_PROGRESS_TICKS)
    );
}

#[test]
fn hidden_opponent_invalidates_identity_instead_of_retaining_a_stale_cell() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    parts.tick = Tick(20);
    let target = StrategicIntent {
        objective: StrategicObjectiveKind::Attack,
        target: StrategicTarget::Opponent { seat: SeatId::One },
        approach_cell: Some(CellId::new(6, 7)),
        phase: StrategicPhase::Approach,
        last_switch_reason: StrategicSwitchReason::Initial,
        observation_tick: Tick(19),
        score: 1,
        committed_until: Tick(40),
        progress_deadline: Tick(60),
        last_distance: 1,
        last_cpu_position: Position::center(CellId::new(7, 7)),
        deferred_until: None,
    };
    let chosen = choose(
        &Observation::new(parts).unwrap(),
        UtilityMemory::new(key()).with_strategic_intent(target),
    );
    assert_ne!(chosen.intent.target, target.target);
    assert_eq!(
        chosen.intent.last_switch_reason,
        StrategicSwitchReason::VisibilityLost
    );
}

#[test]
fn visible_opponent_keeps_seat_identity_while_firing_cell_moves() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    let ObservedActor::Visible(mut rival) = parts.actors[1].clone() else {
        unreachable!()
    };
    rival.id = EntityId(1);
    rival.seat = SeatId::One;
    rival.position = Position::center(CellId::new(10, 7));
    parts.actors[0] = ObservedActor::Visible(rival);
    let observed = Observation::new(parts).unwrap();
    let chosen = choose(&observed, UtilityMemory::new(key()));
    assert_eq!(
        chosen.intent.target,
        StrategicTarget::Opponent { seat: SeatId::One }
    );
    assert_ne!(chosen.intent.approach_cell, Some(CellId::new(10, 7)));
}

#[test]
fn cpu_advance_toward_previous_firing_cell_beats_a_moving_waypoint_false_stall() {
    let deadline = Tick(STRATEGIC_PROGRESS_TICKS);
    let scene = |cpu: CellId, rival_cell: CellId| {
        let mut parts = observation(cpu).into_parts();
        parts.tick = deadline;
        let ObservedActor::Visible(mut rival) = parts.actors[1].clone() else {
            unreachable!()
        };
        rival.id = EntityId(1);
        rival.seat = SeatId::One;
        rival.position = Position::center(rival_cell);
        parts.actors[0] = ObservedActor::Visible(rival);
        Observation::new(parts).unwrap()
    };
    let target = StrategicIntent {
        objective: StrategicObjectiveKind::Attack,
        target: StrategicTarget::Opponent { seat: SeatId::One },
        approach_cell: Some(CellId::new(9, 7)),
        phase: StrategicPhase::Approach,
        last_switch_reason: StrategicSwitchReason::Initial,
        observation_tick: Tick(deadline.0 - 1),
        score: 1,
        committed_until: Tick(30),
        progress_deadline: deadline,
        last_distance: 2,
        last_cpu_position: Position::center(CellId::new(7, 7)),
        deferred_until: None,
    };
    let advanced = choose(
        &scene(CellId::new(8, 7), CellId::new(11, 7)),
        UtilityMemory::new(key()).with_strategic_intent(target),
    );
    assert_eq!(advanced.intent.target, target.target);
    assert_eq!(
        advanced.intent.progress_deadline,
        add_ticks(deadline, STRATEGIC_PROGRESS_TICKS)
    );
    assert_eq!(advanced.failed_target, None);

    let stationary = choose(
        &scene(CellId::new(7, 7), CellId::new(11, 7)),
        UtilityMemory::new(key()).with_strategic_intent(target),
    );
    assert_eq!(
        stationary.failed_target,
        Some((
            target.target,
            add_ticks(deadline, FAILED_TARGET_DEFER_TICKS)
        ))
    );
    assert_eq!(
        stationary.intent.last_switch_reason,
        StrategicSwitchReason::Stalled
    );
}

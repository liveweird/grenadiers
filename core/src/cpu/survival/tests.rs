use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::policy::{
    UtilityMemory,
    tests::{key, observation},
};
use crate::cpu::routing::{PulseRequest, RouteMode, certify_pulse};
use crate::cpu::{
    CpuRng, Intent, MovementIntent, Observation, ObservedActor, ObservedFlame, WorkBudget,
};
use crate::game::{CellId, Direction, Position, Tick};

use super::*;

fn lease(
    basis: &Observation,
    movement: MovementIntent,
    hold: u8,
) -> crate::cpu::routing::PulseLease {
    let hazards = forecast(
        basis,
        &ObservedHazardMemory::new(key()),
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    certify_pulse(
        basis,
        &hazards,
        PulseRequest {
            movement,
            hold_ticks: hold,
            start_window: TickWindow {
                start: Tick(11),
                end: Tick(18),
            },
            stopped_safe_until: Tick(18 + u64::from(hold) + 9),
            mode: RouteMode::CertifiedOnly,
            settled_stop_tick: Tick(10),
        },
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap()
}

fn later(basis: &Observation, tick: u64, position: Position, heading: Direction) -> Observation {
    let mut parts = basis.clone().into_parts();
    let elapsed = u16::try_from(tick - parts.tick.0).unwrap();
    parts.tick = Tick(tick);
    parts.rules.running_updates += elapsed;
    parts.rules.remaining_ticks -= elapsed;
    for flame in &mut parts.flames {
        flame.remaining_ticks = flame.remaining_ticks.saturating_sub(elapsed);
    }
    parts.flames.retain(|flame| flame.remaining_ticks > 0);
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.position = position;
    actor.heading = heading;
    for effect in &mut actor.effects {
        effect.remaining_ticks = effect.remaining_ticks.saturating_sub(elapsed);
    }
    actor.effects.retain(|effect| effect.remaining_ticks > 0);
    Observation::new(parts).unwrap()
}

#[test]
fn delayed_admission_guards_the_snapshot_tick_and_stopped_endpoint() {
    let basis = observation(CellId::new(7, 7));
    let lease = lease(&basis, MovementIntent::Direction(Direction::East), 12);
    let committed = CommittedPulse::admit(lease.clone(), basis.clone(), &basis, Tick(16)).unwrap();
    assert_eq!(committed.first_execution(), Tick(17));
    let pose = lease.pose_after(3).min();
    let middle = later(&basis, 19, pose, Direction::East);
    // Owner tick21 has advanced past this snapshot; do not compare it with
    // the pose after five moves. The recipient snapshot contains three.
    assert_eq!(committed.validate(&middle, Tick(21)), PulseStatus::Continue);
    let stopped = later(&basis, 29, lease.pose_after(12).min(), Direction::East);
    assert_eq!(
        committed.validate(&stopped, Tick(29)),
        PulseStatus::Completed
    );
    assert_eq!(
        committed.validate(&stopped, Tick(lease.safe_until().0 - 1)),
        PulseStatus::Invalidated
    );
    assert!(CommittedPulse::admit(lease, basis.clone(), &basis, Tick(17)).is_err());
}

#[test]
fn fresh_hazards_displacement_and_wrong_identity_invalidate_the_lease() {
    let basis = observation(CellId::new(7, 7));
    let lease = lease(&basis, MovementIntent::Direction(Direction::East), 12);
    let committed = CommittedPulse::admit(lease.clone(), basis.clone(), &basis, Tick(10)).unwrap();
    let mut parts = later(&basis, 13, lease.pose_after(3).min(), Direction::East).into_parts();
    parts.flames.push(ObservedFlame {
        cell: CellId::new(10, 10),
        remaining_ticks: 20,
    });
    assert_eq!(
        committed.validate(&Observation::new(parts).unwrap(), Tick(13)),
        PulseStatus::Invalidated
    );
    let displaced = later(
        &basis,
        13,
        Position::center(CellId::new(6, 7)),
        Direction::East,
    );
    assert_eq!(
        committed.validate(&displaced, Tick(13)),
        PulseStatus::Invalidated
    );
    let mut other = basis.clone().into_parts();
    other.world.physical_round += 1;
    let other = Observation::new(other).unwrap();
    assert!(CommittedPulse::admit(lease, other.clone(), &other, Tick(10)).is_err());
}

#[test]
fn ordinary_admission_preserves_countdowns_but_rejects_changed_facts() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    parts.flames.push(ObservedFlame {
        cell: CellId::new(10, 10),
        remaining_ticks: 2,
    });
    let basis = Observation::new(parts).unwrap();
    let current = later(
        &basis,
        13,
        Position::center(CellId::new(7, 7)),
        Direction::West,
    );
    assert!(can_admit_ordinary(&basis, &current));
    let mut changed = current.into_parts();
    changed.arena.blocks.push(CellId::new(8, 8));
    assert!(!can_admit_ordinary(
        &basis,
        &Observation::new(changed).unwrap()
    ));
}

#[test]
fn waiting_lease_is_explicit_and_exhausted_planning_never_plants() {
    let basis = observation(CellId::new(7, 7));
    let lease = lease(&basis, MovementIntent::Stop, 1);
    let committed = CommittedPulse::admit(lease, basis.clone(), &basis, Tick(10)).unwrap();
    assert_eq!(committed.validate(&basis, Tick(10)), PulseStatus::Wait);
    let mut rng = CpuRng::from_seed(key(), [8; 32]);
    let result = plan(
        &basis,
        &UtilityMemory::new(key()),
        &ObservedHazardMemory::new(key()),
        Tick(10),
        &mut rng,
        &mut WorkBudget::new(0),
    )
    .unwrap();
    assert_eq!(result.control.intent, Intent::STOP);
    assert!(result.lease.is_none());
}

#[test]
fn planner_waits_for_a_temporary_blocked_goal_then_resumes_ordinary_input() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    parts.flames.push(ObservedFlame {
        cell: CellId::new(8, 7),
        remaining_ticks: 30,
    });
    parts.pickups.push(crate::cpu::ObservedPickup {
        cell: CellId::new(9, 7),
        kind: crate::game::PickupKind::BombCapacity,
    });
    let basis = Observation::new(parts).unwrap();
    let run = |observation: &Observation| {
        plan(
            observation,
            &UtilityMemory::new(key()),
            &ObservedHazardMemory::new(key()),
            observation.parts().tick,
            &mut CpuRng::from_seed(key(), [8; 32]),
            &mut WorkBudget::new(WORK_ALLOWANCE),
        )
        .unwrap()
    };
    let waiting = run(&basis);
    assert_eq!(waiting.control.intent, Intent::STOP);
    assert!(
        waiting.lease.is_some(),
        "waiting has an explicit safety lease"
    );
    let clear = later(
        &basis,
        40,
        Position::center(CellId::new(7, 7)),
        Direction::West,
    );
    let resumed = run(&clear);
    assert_eq!(resumed.control.intent, Intent::moving(Direction::East));
    assert!(
        resumed.lease.is_none(),
        "clear-board collection retains ordinary goals"
    );
}

#[test]
fn planner_marks_unknown_motion_as_conditional_and_never_plants() {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    parts.bombs = crate::cpu::BombKnowledge::Visible(vec![crate::cpu::ObservedBomb {
        id: crate::game::EntityId(7),
        cell: CellId::new(4, 7),
        owner: crate::game::EntityId(1),
        range: 3,
        trigger: crate::cpu::ObservedBombTrigger::Timed {
            remaining_ticks: 180,
            paused: false,
        },
        blast: crate::game::BlastKind::Standard,
        unknown: crate::cpu::UnknownBombFacts::ALL,
    }]);
    let observed = Observation::new(parts).unwrap();
    let planned = plan(
        &observed,
        &UtilityMemory::new(key()),
        &ObservedHazardMemory::new(key()),
        Tick(10),
        &mut CpuRng::from_seed(key(), [8; 32]),
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    assert!(planned.control.intent.manual.is_none());
    assert_eq!(planned.control.hold_ticks, 3);
    assert_eq!(
        planned.lease.unwrap().assurance(),
        &crate::cpu::routing::RouteAssurance::ConditionalStationary {
            bombs: vec![crate::game::EntityId(7)]
        }
    );
}

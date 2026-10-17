use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use grenadiers_core::{
    protocol::{RoomCode, RoomId},
    room::Phase,
};
use tokio::sync::{oneshot, watch};

use super::*;
use crate::game::{rooms::owner::RoomCommand, sessions::ConnectionId};
use grenadiers_core::game::Direction;

fn identity() -> RoomIdentity {
    identity_with_profile(super::super::profile::RoomProfile::FixedClassicV1)
}

fn identity_with_profile(profile: super::super::profile::RoomProfile) -> RoomIdentity {
    RoomIdentity {
        id: RoomId::new("00000000000000000000000000000001").unwrap(),
        code: RoomCode::new("ABCDEFGH").unwrap(),
        profile,
    }
}

fn runtime() -> Runtime {
    runtime_with(
        identity(),
        crate::round_seed::RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(0);
            Ok(())
        }),
    )
}

fn runtime_with(
    identity: RoomIdentity,
    round_seed_source: crate::round_seed::RoundSeedSource,
) -> Runtime {
    let initial =
        RoomNotice::View(publication::room_view(&identity, &RoomLifecycle::new(), 1).unwrap());
    let (notices, receiver) = watch::channel(initial);
    drop(receiver);
    Runtime::new(identity, notices, round_seed_source)
}

fn binding(connection: u64) -> BindingKey {
    BindingKey {
        connection_id: ConnectionId(connection),
        generation: 1,
    }
}

fn join(runtime: &mut Runtime, binding: BindingKey) -> Admission {
    let (reply, mut response) = oneshot::channel();
    runtime.command(RoomCommand::Join { binding, reply });
    response.try_recv().unwrap().unwrap()
}

fn action(
    runtime: &mut Runtime,
    binding: BindingKey,
    round_id: u64,
    action: RoomAction,
) -> Result<(), RoomError> {
    let (reply, mut response) = oneshot::channel();
    runtime.command(RoomCommand::Action {
        binding,
        room_id: runtime.identity.id.clone(),
        round_id,
        action,
        reply,
    });
    response.try_recv().unwrap()
}

fn ready_duel(runtime: &mut Runtime) -> (Admission, Admission) {
    let one = join(runtime, binding(1));
    let two = join(runtime, binding(2));
    action(runtime, binding(1), 1, RoomAction::Ready(true)).unwrap();
    action(runtime, binding(2), 1, RoomAction::Ready(true)).unwrap();
    (one, two)
}

#[test]
fn countdown_boundary_publishes_zero_update_state_then_physics_starts_next_tick() {
    let mut runtime = runtime();
    let (_one, _two) = ready_duel(&mut runtime);
    assert_eq!(
        runtime.lifecycle.phase(),
        Phase::Countdown {
            ends_at: grenadiers_core::game::Tick(180)
        }
    );
    for _ in 0..179 {
        assert!(runtime.tick());
    }
    assert!(runtime.state.is_none());
    assert!(runtime.tick());
    let state = runtime.state.as_ref().unwrap();
    assert_eq!(state.tick().0, 180);
    assert_eq!(state.running_updates(), 0);
    assert_eq!(runtime.lifecycle.phase(), Phase::Running);

    assert!(runtime.tick());
    assert_eq!(runtime.state.as_ref().unwrap().tick().0, 181);
    assert_eq!(runtime.state.as_ref().unwrap().running_updates(), 1);
}

#[test]
fn canceled_attempt_rejects_old_epoch_and_current_epoch_can_leave() {
    let mut runtime = runtime();
    let (one, _two) = ready_duel(&mut runtime);
    action(&mut runtime, binding(1), 1, RoomAction::Ready(false)).unwrap();
    assert_eq!(runtime.lifecycle.round_attempt(), 2);
    assert_eq!(one.round_id, 1);
    assert_eq!(
        action(&mut runtime, binding(1), 1, RoomAction::Ready(true)),
        Err(RoomError::StaleRoom)
    );

    let (reply, mut response) = oneshot::channel();
    runtime.command(RoomCommand::Leave {
        binding: binding(1),
        room_id: runtime.identity.id.clone(),
        round_id: 2,
        reply,
    });
    assert_eq!(response.try_recv().unwrap(), Ok(()));
}

#[test]
fn recipients_observe_the_same_snapshot_bytes_and_monotonic_revision() {
    let mut runtime = runtime();
    let (mut one, mut two) = ready_duel(&mut runtime);
    for _ in 0..180 {
        runtime.tick();
    }
    one.notices.borrow_and_update();
    two.notices.borrow_and_update();
    let first = one.notices.borrow().clone();
    let second = two.notices.borrow().clone();
    let (RoomNotice::Snapshot(first), RoomNotice::Snapshot(second)) = (first, second) else {
        panic!("expected snapshots");
    };
    assert_eq!(first, second);
    assert!(std::sync::Arc::ptr_eq(&first, &second));
    let initial: serde_json::Value = serde_json::from_str(&first).unwrap();

    for _ in 0..3 {
        runtime.tick();
    }
    let next = match one.notices.borrow().clone() {
        RoomNotice::Snapshot(snapshot) => snapshot,
        _ => panic!("expected snapshot"),
    };
    let next: serde_json::Value = serde_json::from_str(&next).unwrap();
    assert!(next["room"]["revision"].as_u64() > initial["room"]["revision"].as_u64());
    assert_eq!(next["room"]["tick"], 183);
}

#[test]
fn unchanged_movement_refresh_preserves_fractional_progress() {
    let mut runtime = runtime();
    let (_one, _two) = ready_duel(&mut runtime);
    for _ in 0..180 {
        runtime.tick();
    }
    let start = runtime.state.as_ref().unwrap().actors()[0].position().x;
    for _ in 0..5 {
        action(
            &mut runtime,
            binding(1),
            1,
            RoomAction::Movement(Some(Direction::East)),
        )
        .unwrap();
        runtime.tick();
    }

    assert_eq!(
        runtime.state.as_ref().unwrap().actors()[0].position().x,
        start + 256
    );
}

#[test]
fn failed_capacity_action_id_remains_consumed_after_capacity_releases() {
    let mut runtime = runtime();
    let (_one, _two) = ready_duel(&mut runtime);
    for _ in 0..180 {
        runtime.tick();
    }
    action(
        &mut runtime,
        binding(1),
        1,
        RoomAction::Movement(Some(Direction::East)),
    )
    .unwrap();
    action(&mut runtime, binding(1), 1, RoomAction::Bomb(1)).unwrap();
    runtime.tick();
    action(&mut runtime, binding(1), 1, RoomAction::Bomb(2)).unwrap();
    runtime.tick();
    assert_eq!(
        action(&mut runtime, binding(1), 1, RoomAction::Bomb(2)),
        Err(RoomError::DuplicateAction)
    );

    while runtime.lifecycle.tick().0 < 361 {
        action(
            &mut runtime,
            binding(1),
            1,
            RoomAction::Movement(Some(Direction::East)),
        )
        .unwrap();
        runtime.tick();
    }
    assert!(runtime.state.as_ref().unwrap().world().bombs().is_empty());
    assert_eq!(
        action(&mut runtime, binding(1), 1, RoomAction::Bomb(2)),
        Err(RoomError::DuplicateAction)
    );
    action(&mut runtime, binding(1), 1, RoomAction::Bomb(3)).unwrap();
    runtime.tick();
    assert_eq!(runtime.state.as_ref().unwrap().world().bombs().len(), 1);
}

#[test]
fn interruption_snapshot_flushes_unpublished_tick_events() {
    let mut runtime = runtime();
    let (mut one, _two) = ready_duel(&mut runtime);
    for _ in 0..180 {
        runtime.tick();
    }
    one.notices.borrow_and_update();
    action(&mut runtime, binding(1), 1, RoomAction::Bomb(1)).unwrap();
    runtime.tick();
    runtime.depart(1, DepartureReason::TransportLoss);

    let RoomNotice::Snapshot(snapshot) = one.notices.borrow_and_update().clone() else {
        panic!("expected terminal snapshot");
    };
    let snapshot: serde_json::Value = serde_json::from_str(&snapshot).unwrap();
    assert_eq!(snapshot["room"]["phase"]["type"], "interrupted");
    assert_eq!(snapshot["events"][0]["tick"], 181);
    assert_eq!(snapshot["events"][0]["kind"]["type"], "bombPlaced");
}

#[test]
fn publication_failure_marks_owner_for_fail_closed_termination() {
    let mut runtime = runtime();
    let (_one, _two) = ready_duel(&mut runtime);
    for _ in 0..180 {
        runtime.tick();
    }
    runtime.revision = u32::MAX;
    runtime.depart(1, DepartureReason::TransportLoss);

    assert!(runtime.failed);
    assert_eq!(runtime.revision, u32::MAX);
}

#[test]
fn completed_timeout_result_survives_later_departure() {
    let mut runtime = runtime();
    let (_one, _two) = ready_duel(&mut runtime);
    for _ in 0..180 + 10_800 {
        assert!(runtime.tick());
    }
    let result = runtime.lifecycle.phase();
    assert!(matches!(result, Phase::Result { .. }));
    let completed_tick = runtime.lifecycle.tick();
    for _ in 0..3 {
        runtime.tick();
    }
    assert_eq!(runtime.lifecycle.tick().0, completed_tick.0 + 3);
    assert_eq!(
        runtime.state.as_ref().unwrap().tick(),
        runtime.lifecycle.tick()
    );
    assert_eq!(runtime.lifecycle.phase(), result);
    runtime.depart(0, DepartureReason::TransportLoss);
    assert_eq!(runtime.lifecycle.phase(), result);
}

#[test]
fn countdown_updates_public_time_at_twenty_hertz_while_waiting_stays_quiet() {
    let mut runtime = runtime();
    let one = join(&mut runtime, binding(1));
    let waiting_revision = runtime.revision;
    for _ in 0..6 {
        assert!(runtime.tick());
    }
    assert_eq!(runtime.revision, waiting_revision);
    let _two = join(&mut runtime, binding(2));
    action(&mut runtime, binding(1), 1, RoomAction::Ready(true)).unwrap();
    action(&mut runtime, binding(2), 1, RoomAction::Ready(true)).unwrap();
    let started_revision = runtime.revision;
    for tick in 7..=12 {
        assert!(runtime.tick());
        assert_eq!(runtime.revision, started_revision + ((tick - 6) / 3) as u32);
        if tick % 3 == 0 {
            let RoomNotice::View(view) = one.notices.borrow().clone() else {
                panic!("countdown must publish a room view");
            };
            assert_eq!(view.tick.get(), tick);
            assert_eq!(
                view.phase,
                grenadiers_core::protocol::RoomPhase::Countdown {
                    ends_at: grenadiers_core::protocol::Tick::new(186).unwrap(),
                }
            );
        }
    }
}

#[test]
fn generated_round_consumes_one_seed_only_at_the_actual_start_boundary() {
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&calls);
    let source = crate::round_seed::RoundSeedSource::with_entropy(move |bytes| {
        observed.fetch_add(1, Ordering::SeqCst);
        bytes.fill(19);
        Ok(())
    });
    let identity = identity_with_profile(super::super::profile::RoomProfile::GeneratedClassicV2);
    let mut runtime = runtime_with(identity, source);

    let (_one, _two) = ready_duel(&mut runtime);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    for _ in 0..179 {
        assert!(runtime.tick());
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(runtime.state.is_none());

    assert!(runtime.tick());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        runtime.generation_record.expect("generation record").seed(),
        [19; 32]
    );
    assert_eq!(runtime.state.as_ref().unwrap().rules().version(), 2);
}

#[test]
fn canceled_countdown_does_not_consume_a_round_seed() {
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&calls);
    let source = crate::round_seed::RoundSeedSource::with_entropy(move |_| {
        observed.fetch_add(1, Ordering::SeqCst);
        Ok(())
    });
    let identity = identity_with_profile(super::super::profile::RoomProfile::GeneratedClassicV2);
    let mut runtime = runtime_with(identity, source);
    let (_one, _two) = ready_duel(&mut runtime);

    for _ in 0..90 {
        assert!(runtime.tick());
    }
    action(&mut runtime, binding(1), 1, RoomAction::Ready(false)).unwrap();
    for _ in 0..180 {
        assert!(runtime.tick());
    }

    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(runtime.state.is_none());
    assert!(runtime.generation_record.is_none());
}

#[test]
fn entropy_failure_interrupts_without_constructing_a_world() {
    let source = crate::round_seed::RoundSeedSource::with_entropy(|_| Err(()));
    let identity = identity_with_profile(super::super::profile::RoomProfile::GeneratedClassicV2);
    let mut runtime = runtime_with(identity, source);
    let (_one, _two) = ready_duel(&mut runtime);

    for _ in 0..180 {
        assert!(runtime.tick());
    }

    assert!(matches!(
        runtime.lifecycle.phase(),
        Phase::Interrupted {
            reason: InterruptReason::InternalError,
            ..
        }
    ));
    assert!(runtime.state.is_none());
    assert!(runtime.generation_record.is_none());
}

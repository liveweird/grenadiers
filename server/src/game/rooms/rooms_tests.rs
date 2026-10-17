use std::sync::{
    Arc,
    atomic::{AtomicU64, AtomicUsize, Ordering},
};

use super::*;
use crate::game::sessions::{ConnectionId, GameSessions};
use grenadiers_core::protocol::{MapId, RoomCode, Seat};

fn deterministic_rooms(now: StdInstant) -> Arc<GameRooms> {
    let codes = AtomicU64::new(0);
    GameRooms::with_random_fill(now, move |bytes| {
        if bytes.len() == 8 {
            bytes.copy_from_slice(&0x1234_5678_9abc_def0_u64.to_be_bytes());
        } else if bytes.len() == 5 {
            let value = codes.fetch_add(1, Ordering::Relaxed).to_be_bytes();
            bytes.copy_from_slice(&value[3..]);
        } else {
            bytes.fill(codes.fetch_add(1, Ordering::Relaxed) as u8);
        }
        Ok(())
    })
}

fn binding(connection: u64, generation: u64) -> BindingKey {
    BindingKey {
        connection_id: ConnectionId(connection),
        generation,
    }
}

async fn wait_until_empty(rooms: &GameRooms) {
    for _ in 0..20 {
        if rooms.lock().rooms.is_empty() {
            return;
        }
        tokio::task::yield_now().await;
    }
    assert!(rooms.lock().rooms.is_empty());
}

#[tokio::test]
async fn identifiers_are_canonical_and_last_seat_has_one_winner() {
    let now = StdInstant::now();
    let rooms = deterministic_rooms(now);
    let owner = rooms.create(binding(1, 1), now).await.unwrap();
    assert_eq!(owner.room_id.as_str(), "123456789abcdef00000000000000001");
    assert_eq!(owner.room.room_code.as_str().len(), 8);
    assert_eq!(owner.seat, Seat::One);
    let code = owner.room.room_code.clone();

    let (left, right) = tokio::join!(
        rooms.join(&code, binding(2, 1), now),
        rooms.join(&code, binding(3, 1), now),
    );
    let successes = [&left, &right]
        .into_iter()
        .filter(|result| result.is_ok())
        .count();
    assert_eq!(successes, 1);
    assert!(matches!(
        (&left, &right),
        (Ok(admission), Err(RoomError::RoomFull))
            | (Err(RoomError::RoomFull), Ok(admission))
            if admission.seat == Seat::Two
    ));
}

#[tokio::test]
async fn token_room_and_round_randomness_are_independent() {
    let now = StdInstant::now();
    let token_calls = Arc::new(AtomicUsize::new(0));
    let observed_tokens = Arc::clone(&token_calls);
    let sessions = GameSessions::with_random_fill(now, move |bytes| {
        observed_tokens.fetch_add(1, Ordering::SeqCst);
        bytes.fill(7);
        Ok(())
    });
    sessions.issue(now).unwrap();

    let room_calls = Arc::new(AtomicUsize::new(0));
    let observed_rooms = Arc::clone(&room_calls);
    let seed_calls = Arc::new(AtomicUsize::new(0));
    let observed_seeds = Arc::clone(&seed_calls);
    let seed_source = crate::round_seed::RoundSeedSource::with_entropy(move |_| {
        observed_seeds.fetch_add(1, Ordering::SeqCst);
        Ok(())
    });
    let rooms = GameRooms::with_random_fill_and_round_seed(
        now,
        move |bytes| {
            observed_rooms.fetch_add(1, Ordering::SeqCst);
            bytes.fill(0);
            Ok(())
        },
        seed_source,
    );
    let admission = rooms.create(binding(1, 1), now).await.unwrap();

    assert_eq!(token_calls.load(Ordering::SeqCst), 1);
    assert_eq!(room_calls.load(Ordering::SeqCst), 3);
    assert_eq!(seed_calls.load(Ordering::SeqCst), 0);
    assert_eq!(admission.room.rules_version.get(), 2);
    assert_eq!(admission.room.map_id, MapId::GeneratedV1);
}

#[tokio::test]
async fn canceled_join_before_mutation_does_not_claim_the_last_seat() {
    let now = StdInstant::now();
    let rooms = deterministic_rooms(now);
    let owner = rooms.create(binding(1, 1), now).await.unwrap();
    let handle = rooms.lock().rooms.get(&owner.room_id).unwrap().clone();
    handle.owner.submit_canceled_join(binding(2, 1));
    tokio::task::yield_now().await;

    let joined = rooms
        .join(&owner.room.room_code, binding(3, 1), now)
        .await
        .unwrap();
    assert_eq!(joined.seat, Seat::Two);
    assert_eq!(owner.room.revision.get(), 2);
    assert_eq!(joined.room.revision.get(), 3);
}

#[tokio::test(start_paused = true)]
async fn dropped_admission_cleans_an_empty_room_even_with_a_full_command_queue() {
    let now = StdInstant::now();
    let rooms = deterministic_rooms(now);
    let admission = rooms.create(binding(1, 1), now).await.unwrap();
    let handle = rooms.lock().rooms.get(&admission.room_id).unwrap().clone();
    for connection in 2..=33 {
        drop(handle.owner.join(binding(connection, 1)));
    }

    drop(admission);
    tokio::time::advance(std::time::Duration::from_millis(17)).await;
    wait_until_empty(&rooms).await;
}

#[tokio::test(start_paused = true)]
async fn exact_expiry_publishes_expired_and_removes_both_lookups() {
    let now = StdInstant::now();
    let rooms = deterministic_rooms(now);
    let admission = rooms.create(binding(1, 1), now).await.unwrap();
    let id = admission.room_id.clone();
    let code = admission.room.room_code.clone();
    let mut notices = admission.notices.clone();

    tokio::time::advance(std::time::Duration::from_secs(300)).await;
    notices.changed().await.unwrap();
    assert!(matches!(
        &*notices.borrow_and_update(),
        RoomNotice::Closing {
            reason: ClosingReason::Expired,
            ..
        }
    ));
    assert!(matches!(
        rooms.join(&code, binding(2, 1), now).await,
        Err(RoomError::RoomNotFound)
    ));
    assert!(matches!(
        rooms
            .leave(
                &admission,
                binding(1, 1),
                admission.room_id.clone(),
                admission.round_id,
            )
            .await,
        Err(RoomError::RoomNotFound)
    ));
    wait_until_empty(&rooms).await;
    let registry = rooms.lock();
    assert!(!registry.rooms.contains_key(&id));
    assert!(!registry.codes.contains_key(&code));
}

#[tokio::test]
async fn room_capacity_and_shared_failed_attempt_rate_limit_fail_closed() {
    let now = StdInstant::now();
    let rooms = deterministic_rooms(now);
    let mut admissions = Vec::new();
    for index in 0..MAX_ROOMS {
        admissions.push(
            rooms
                .create(
                    binding(index as u64 + 1, 1),
                    now + std::time::Duration::from_millis(index as u64 * 125),
                )
                .await
                .unwrap(),
        );
    }
    assert!(matches!(
        rooms
            .create(binding(99, 1), now + std::time::Duration::from_secs(4))
            .await,
        Err(RoomError::CapacityReached)
    ));
    drop(admissions);

    let limited = deterministic_rooms(now);
    let missing = RoomCode::new("ABCDEFGH").unwrap();
    for index in 0..OPERATION_BURST as u64 {
        assert!(matches!(
            limited.join(&missing, binding(index + 1, 1), now).await,
            Err(RoomError::RoomNotFound)
        ));
    }
    assert!(matches!(
        limited.join(&missing, binding(99, 1), now).await,
        Err(RoomError::RateLimited)
    ));
}

#[test]
fn id_and_random_failures_preserve_registry_state() {
    let now = StdInstant::now();
    let rooms = deterministic_rooms(now);
    {
        let mut registry = rooms.lock();
        registry.prefix = Some(1);
        registry.next_serial = u64::MAX;
        assert!(matches!(
            rooms.generate_identity(&mut registry),
            Err(RoomError::CapacityReached)
        ));
        assert_eq!(registry.next_serial, u64::MAX);
        assert!(registry.rooms.is_empty());
    }

    let failing = GameRooms::with_random_fill(now, |_| Err(()));
    let mut registry = failing.lock();
    assert!(matches!(
        failing.generate_identity(&mut registry),
        Err(RoomError::CapacityReached)
    ));
    assert!(registry.prefix.is_none());
    assert_eq!(registry.next_serial, 1);
}

#[tokio::test]
async fn live_code_collisions_preserve_existing_mapping_and_fail_after_retry_bound() {
    let now = StdInstant::now();
    let rooms = GameRooms::with_random_fill(now, |bytes| {
        bytes.fill(0);
        Ok(())
    });
    let existing = rooms.create(binding(1, 1), now).await.unwrap();
    let code = existing.room.room_code.clone();
    let id = existing.room_id.clone();

    assert!(matches!(
        rooms.create(binding(2, 1), now).await,
        Err(RoomError::CapacityReached)
    ));
    let registry = rooms.lock();
    assert_eq!(registry.codes.get(&code), Some(&id));
    assert!(registry.rooms.contains_key(&id));
    assert_eq!(registry.rooms.len(), 1);
}

#[tokio::test]
async fn stale_binding_is_rejected_after_leave_and_rejoin_in_same_round() {
    let now = StdInstant::now();
    let rooms = deterministic_rooms(now);
    let first = rooms.create(binding(1, 1), now).await.unwrap();
    let second = rooms
        .join(&first.room.room_code, binding(2, 1), now)
        .await
        .unwrap();
    rooms
        .leave(&first, binding(1, 1), first.room_id.clone(), first.round_id)
        .await
        .unwrap();
    let rejoined = rooms
        .join(&first.room.room_code, binding(1, 2), now)
        .await
        .unwrap();

    assert_eq!(rejoined.round_id, first.round_id);
    assert!(matches!(
        rooms
            .action(
                &rejoined,
                binding(1, 1),
                rejoined.room_id.clone(),
                rejoined.round_id,
                RoomAction::Ready(false),
            )
            .await,
        Err(RoomError::StaleRoom)
    ));
    rooms
        .action(
            &rejoined,
            binding(1, 2),
            rejoined.room_id.clone(),
            rejoined.round_id,
            RoomAction::Ready(false),
        )
        .await
        .unwrap();
    drop(second);
}

#[tokio::test(start_paused = true)]
async fn scheduler_catches_up_four_due_ticks_without_skipping() {
    let now = StdInstant::now();
    let rooms = deterministic_rooms(now);
    let mut one = rooms.create(binding(1, 1), now).await.unwrap();
    let two = rooms
        .join(&one.room.room_code, binding(2, 1), now)
        .await
        .unwrap();
    rooms
        .action(
            &one,
            binding(1, 1),
            one.room_id.clone(),
            1,
            RoomAction::Ready(true),
        )
        .await
        .unwrap();
    rooms
        .action(
            &two,
            binding(2, 1),
            two.room_id.clone(),
            1,
            RoomAction::Ready(true),
        )
        .await
        .unwrap();

    tokio::time::advance(std::time::Duration::from_millis(67)).await;
    tokio::task::yield_now().await;
    rooms
        .action(
            &one,
            binding(1, 1),
            one.room_id.clone(),
            1,
            RoomAction::Ready(false),
        )
        .await
        .unwrap();
    one.notices.changed().await.unwrap();
    let RoomNotice::Publication(notice) = one.notices.borrow_and_update().clone() else {
        panic!("expected waiting view");
    };
    let view = notice.room;
    assert_eq!(view.tick.get(), 4);
    assert_eq!(view.round_id.get(), 2);
}

#[tokio::test(start_paused = true)]
async fn excessive_lag_interrupts_then_terminal_retention_releases_registration() {
    let now = StdInstant::now();
    let rooms = deterministic_rooms(now);
    let mut admission = rooms.create(binding(1, 1), now).await.unwrap();
    let id = admission.room_id.clone();
    let code = admission.room.room_code.clone();

    tokio::time::advance(std::time::Duration::from_millis(517)).await;
    admission.notices.changed().await.unwrap();
    let RoomNotice::Publication(notice) = admission.notices.borrow_and_update().clone() else {
        panic!("expected interruption view");
    };
    let view = notice.room;
    assert!(matches!(
        view.phase,
        grenadiers_core::protocol::v4::RoomPhase::Interrupted {
            reason: grenadiers_core::protocol::v4::MatchInterruptReason::ServerOverloaded,
            ..
        }
    ));
    assert!(rooms.lock().rooms.contains_key(&id));

    tokio::time::advance(std::time::Duration::from_secs(30)).await;
    admission.notices.changed().await.unwrap();
    assert!(matches!(
        &*admission.notices.borrow_and_update(),
        RoomNotice::Closing {
            reason: ClosingReason::Retired,
            ..
        }
    ));
    wait_until_empty(&rooms).await;
    let registry = rooms.lock();
    assert!(!registry.rooms.contains_key(&id));
    assert!(!registry.codes.contains_key(&code));
}

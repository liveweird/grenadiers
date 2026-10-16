use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use grenadiers_core::protocol::{RoomCode, Seat};
use tokio::sync::oneshot;

use super::*;
use crate::game::sessions::ConnectionId;

fn deterministic_rooms(now: StdInstant) -> Arc<GameRooms> {
    let codes = AtomicU64::new(0);
    GameRooms::with_random_fill(now, move |bytes| {
        if bytes.len() == 8 {
            bytes.copy_from_slice(&0x1234_5678_9abc_def0_u64.to_be_bytes());
        } else {
            let value = codes.fetch_add(1, Ordering::Relaxed).to_be_bytes();
            bytes.copy_from_slice(&value[3..]);
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
    assert_eq!(owner.view.room_code.as_str().len(), 8);
    assert_eq!(owner.seat, Seat::One);
    let code = owner.view.room_code.clone();

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
async fn canceled_join_before_mutation_does_not_claim_the_last_seat() {
    let now = StdInstant::now();
    let rooms = deterministic_rooms(now);
    let owner = rooms.create(binding(1, 1), now).await.unwrap();
    let handle = rooms.lock().rooms.get(&owner.room_id).unwrap().clone();
    let (reply, canceled) = oneshot::channel();
    drop(canceled);
    handle
        .commands
        .try_send(owner::RoomCommand::Join {
            binding: binding(2, 1),
            reply,
        })
        .unwrap();
    tokio::task::yield_now().await;

    let joined = rooms
        .join(&owner.view.room_code, binding(3, 1), now)
        .await
        .unwrap();
    assert_eq!(joined.seat, Seat::Two);
    assert_eq!(joined.view.revision.get(), 2);
}

#[tokio::test]
async fn dropped_admission_cleans_an_empty_room_even_with_a_full_command_queue() {
    let now = StdInstant::now();
    let rooms = deterministic_rooms(now);
    let admission = rooms.create(binding(1, 1), now).await.unwrap();
    let handle = rooms.lock().rooms.get(&admission.room_id).unwrap().clone();
    for connection in 2..=owner::COMMAND_CAPACITY as u64 + 1 {
        let (reply, response) = oneshot::channel();
        drop(response);
        handle
            .commands
            .try_send(owner::RoomCommand::Join {
                binding: binding(connection, 1),
                reply,
            })
            .unwrap();
    }

    drop(admission);
    wait_until_empty(&rooms).await;
}

#[tokio::test(start_paused = true)]
async fn exact_expiry_publishes_expired_and_removes_both_lookups() {
    let now = StdInstant::now();
    let rooms = deterministic_rooms(now);
    let admission = rooms.create(binding(1, 1), now).await.unwrap();
    let id = admission.room_id.clone();
    let code = admission.view.room_code.clone();
    let mut notices = admission.notices.clone();

    tokio::time::advance(owner::ROOM_LIFETIME).await;
    notices.changed().await.unwrap();
    assert_eq!(*notices.borrow_and_update(), RoomNotice::Expired);
    assert!(matches!(
        rooms.join(&code, binding(2, 1), now).await,
        Err(RoomError::RoomNotFound)
    ));
    assert!(matches!(
        rooms.leave(&admission, binding(1, 1)).await,
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

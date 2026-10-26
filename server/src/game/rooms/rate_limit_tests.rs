use std::time::{Duration, Instant};

use grenadiers_core::protocol::{Counter, RoomCode, RoomId, v12::ResumeToken};

use super::{OPERATION_BURST, OperationBudgets};
use crate::game::{
    BindingKey, ConnectionId,
    rooms::{GameRooms, RoomError},
};

fn binding(connection: u64) -> BindingKey {
    BindingKey {
        connection_id: ConnectionId(connection),
        generation: 1,
    }
}

#[test]
fn operation_budgets_are_independent_and_refill_from_explicit_time() {
    let now = Instant::now();
    let mut budgets = OperationBudgets::new(now);

    for _ in 0..OPERATION_BURST as usize {
        assert!(budgets.take_create(now));
    }
    assert!(!budgets.take_create(now));
    assert!(budgets.take_join(now));
    assert!(budgets.take_resume(now));

    for _ in 1..OPERATION_BURST as usize {
        assert!(budgets.take_join(now));
    }
    assert!(!budgets.take_join(now));
    assert!(budgets.take_resume(now));

    for _ in 2..OPERATION_BURST as usize {
        assert!(budgets.take_resume(now));
    }
    assert!(!budgets.take_resume(now));

    let one_token_later = now + Duration::from_millis(125);
    assert!(budgets.take_create(one_token_later));
    assert!(!budgets.take_create(one_token_later));
    assert!(budgets.take_join(one_token_later));
    assert!(!budgets.take_join(one_token_later));
    assert!(budgets.take_resume(one_token_later));
    assert!(!budgets.take_resume(one_token_later));
}

#[tokio::test]
async fn failed_operations_charge_only_their_own_budget() {
    let now = Instant::now();
    let rooms = GameRooms::with_random_fill(now, |_| Err(()));
    let missing_code = RoomCode::new("ABCDEFGH").unwrap();
    let missing_room = RoomId::new("00000000000000000000000000000001").unwrap();
    let missing_token = ResumeToken::new("0".repeat(64)).unwrap();

    for index in 0..OPERATION_BURST as u64 {
        assert!(matches!(
            rooms.create(binding(index + 1), now).await,
            Err(RoomError::CapacityReached)
        ));
    }
    assert!(matches!(
        rooms.create(binding(100), now).await,
        Err(RoomError::RateLimited)
    ));

    for index in 0..OPERATION_BURST as u64 {
        assert!(matches!(
            rooms.join(&missing_code, binding(index + 200), now).await,
            Err(RoomError::RoomNotFound)
        ));
    }
    assert!(matches!(
        rooms.join(&missing_code, binding(300), now).await,
        Err(RoomError::RateLimited)
    ));

    for index in 0..OPERATION_BURST as u64 {
        assert!(matches!(
            rooms
                .resume(
                    missing_room.clone(),
                    binding(index + 400),
                    Counter::new(1).unwrap(),
                    missing_token.clone(),
                    now,
                )
                .await,
            Err(RoomError::ResumeRejected)
        ));
    }
    assert!(matches!(
        rooms
            .resume(
                missing_room,
                binding(500),
                Counter::new(1).unwrap(),
                missing_token,
                now,
            )
            .await,
        Err(RoomError::RateLimited)
    ));
}

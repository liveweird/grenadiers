use std::{collections::VecDeque, time::Duration};

use grenadiers_core::protocol::Counter;
use tokio::sync::oneshot;

use super::{
    runner::stage_admission,
    test_support::{advance_ticks, binding, publication, start_round, started},
    types::{COMMAND_CAPACITY, OwnerCommand},
};
use crate::game::rooms::{match_runtime::MatchAction, types::RoomError};

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn retained_admission_blocks_late_commands_until_all_overdue_ticks_settle() {
    let value = started();
    let (first, second, one, _) = start_round(&value.handle).await;
    let initial_tick = publication(&value.notices).room.tick.get();
    let takeover = value.handle.resume(
        binding(3, 1),
        Counter::new(2).unwrap(),
        first.room_id.clone(),
        first.resume_token.clone(),
    );

    tokio::time::advance(Duration::from_millis(100)).await;
    let first_chunk_tick = publication(&value.notices).room.tick.get();
    assert!(first_chunk_tick >= initial_tick + 3);
    assert!(first_chunk_tick < initial_tick + 6);
    let late_bomb = value.handle.action(
        one,
        first.room_id.clone(),
        first.round_id,
        MatchAction::Bomb(1),
    );

    advance_ticks(1).await;
    let replacement = takeover.await.unwrap().unwrap();
    assert!(matches!(
        late_bomb.await.unwrap(),
        Err(RoomError::StaleRoom)
    ));
    assert!(replacement.state.as_ref().unwrap().bombs.is_empty());
    drop(replacement);
    drop(second);
}

#[tokio::test]
async fn staged_admissions_are_explicitly_bounded_to_one_command_batch() {
    let mut admissions = VecDeque::new();
    let mut replies = Vec::new();
    for index in 0..=COMMAND_CAPACITY {
        let (reply, response) = oneshot::channel();
        stage_admission(
            &mut admissions,
            OwnerCommand::Join {
                binding: binding(index as u64 + 1, 1),
                sequence: Counter::new(1).unwrap(),
                reply,
            },
        );
        replies.push(response);
    }
    assert_eq!(admissions.len(), COMMAND_CAPACITY);
    assert!(matches!(
        replies.pop().unwrap().await.unwrap(),
        Err(RoomError::RateLimited)
    ));
}

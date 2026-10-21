use grenadiers_core::{
    game::Direction,
    protocol::{
        Counter, MAX_COUNTER,
        v9::{ConnectionStatus, ResumeToken},
    },
};
use tokio::sync::oneshot;

use super::{test_support::*, types::MatchOwnerNotice};
use crate::game::rooms::{match_runtime::MatchAction, types::RoomError};

fn sequence(value: u32) -> Counter {
    Counter::new(value).unwrap()
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn connected_takeover_fences_old_binding_and_drop_then_restore_keeps_private_baseline() {
    let value = started();
    let (first, second, one, _two) = start_round(&value.handle).await;
    value
        .handle
        .action(
            one,
            first.room_id.clone(),
            first.round_id,
            MatchAction::Bomb(MAX_COUNTER),
        )
        .await
        .unwrap()
        .unwrap();

    let replacement_binding = binding(3, 1);
    let replacement = value
        .handle
        .resume(
            replacement_binding,
            sequence(9),
            first.room_id.clone(),
            first.resume_token.clone(),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(first.replaced.await, Ok(()));
    assert_eq!(replacement.resume_token, first.resume_token);
    assert_eq!(replacement.last_bomb_action_id, Some(sequence(MAX_COUNTER)));
    let admitted: serde_json::Value = serde_json::from_str(&replacement.personal_frame).unwrap();
    assert!(!admitted["state"].is_null());
    assert_eq!(
        replacement.room.seats[0].connection,
        ConnectionStatus::Connected
    );
    assert!(matches!(
        value
            .handle
            .action(
                one,
                first.room_id.clone(),
                first.round_id,
                MatchAction::Movement(None),
            )
            .await
            .unwrap(),
        Err(RoomError::StaleRoom)
    ));
    assert_eq!(
        value
            .handle
            .action(
                replacement_binding,
                replacement.room_id.clone(),
                replacement.round_id,
                MatchAction::Movement(Some(Direction::East)),
            )
            .await
            .unwrap(),
        Ok(())
    );

    drop(first.guard);
    advance_ticks(1).await;
    assert_eq!(
        publication(&value.notices).room.seats[0].connection,
        ConnectionStatus::Connected
    );
    assert_eq!(
        value
            .handle
            .action(
                replacement_binding,
                replacement.room_id.clone(),
                replacement.round_id,
                MatchAction::Movement(None),
            )
            .await
            .unwrap(),
        Ok(())
    );

    let token = replacement.resume_token.clone();
    let room_id = replacement.room_id.clone();
    drop(replacement.guard);
    advance_ticks(2).await;
    assert!(matches!(
        publication(&value.notices).room.seats[0].connection,
        ConnectionStatus::Disconnected { .. }
    ));
    let restored = value
        .handle
        .resume(binding(4, 1), sequence(1), room_id, token)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        restored.room.seats[0].connection,
        ConnectionStatus::Connected
    );
    assert_eq!(restored.last_bomb_action_id, Some(sequence(MAX_COUNTER)));
    let restored_frame: serde_json::Value = serde_json::from_str(&restored.personal_frame).unwrap();
    assert!(!restored_frame["state"].is_null());
    drop(restored);
    drop(second);
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn canceled_or_wrong_resume_does_not_replace_the_live_binding() {
    let value = started();
    let mut admission = join(&value.handle, binding(1, 1)).await;
    let wrong = ResumeToken::new("f".repeat(64)).unwrap();
    assert!(matches!(
        value
            .handle
            .resume(binding(2, 1), sequence(2), admission.room_id.clone(), wrong,)
            .await
            .unwrap(),
        Err(RoomError::ResumeRejected)
    ));

    let canceled = value.handle.resume(
        binding(3, 1),
        sequence(3),
        admission.room_id.clone(),
        admission.resume_token.clone(),
    );
    drop(canceled);
    tokio::task::yield_now().await;
    assert_eq!(
        admission.replaced.try_recv(),
        Err(oneshot::error::TryRecvError::Empty)
    );
    assert_eq!(
        value
            .handle
            .action(
                binding(1, 1),
                admission.room_id.clone(),
                admission.round_id,
                MatchAction::Ready(false),
            )
            .await
            .unwrap(),
        Ok(()),
    );
    assert!(matches!(
        value.notices.borrow().clone(),
        MatchOwnerNotice::Publication(_)
    ));
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn restore_is_accepted_one_tick_before_deadline_and_rejected_at_deadline() {
    let allowed = started();
    let (first, second, _, _) = start_round(&allowed.handle).await;
    let room_id = first.room_id.clone();
    let token = first.resume_token.clone();
    drop(first);
    advance_ticks(1).await;
    advance_ticks(599).await;
    let restored = allowed
        .handle
        .resume(binding(9, 1), sequence(1), room_id, token)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        restored.room.seats[0].connection,
        ConnectionStatus::Connected
    );
    drop(restored);
    drop(second);

    let expired = started();
    let (first, second, _, _) = start_round(&expired.handle).await;
    let room_id = first.room_id.clone();
    let token = first.resume_token.clone();
    drop(first);
    advance_ticks(1).await;
    advance_ticks(600).await;
    assert!(matches!(
        expired
            .handle
            .resume(binding(10, 1), sequence(1), room_id, token)
            .await
            .unwrap(),
        Err(RoomError::ResumeRejected)
    ));
    assert_eq!(
        publication(&expired.notices).room.seats[0].connection,
        ConnectionStatus::Expired
    );
    drop(second);
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn overload_and_closed_owner_normalize_resume_without_replacing_binding() {
    let overloaded = started();
    let mut admission = join(&overloaded.handle, binding(1, 1)).await;
    tokio::time::advance(std::time::Duration::from_millis(517)).await;
    let response = overloaded.handle.resume(
        binding(2, 1),
        sequence(2),
        admission.room_id.clone(),
        admission.resume_token.clone(),
    );
    let invalid = overloaded.handle.resume(
        binding(3, 1),
        sequence(3),
        admission.room_id.clone(),
        ResumeToken::new("f".repeat(64)).unwrap(),
    );
    tokio::task::yield_now().await;
    assert!(matches!(
        response.await.unwrap(),
        Err(RoomError::ResumeRejected)
    ));
    assert!(matches!(
        invalid.await.unwrap(),
        Err(RoomError::ResumeRejected)
    ));
    assert_eq!(
        admission.replaced.try_recv(),
        Err(oneshot::error::TryRecvError::Empty)
    );

    overloaded.task.abort();
    let _ = overloaded.task.await;
    let closed = overloaded.handle.resume(
        binding(4, 1),
        sequence(4),
        admission.room_id.clone(),
        admission.resume_token.clone(),
    );
    assert!(matches!(
        closed.await.unwrap(),
        Err(RoomError::ResumeRejected)
    ));
}

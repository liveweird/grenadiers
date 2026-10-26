use grenadiers_core::protocol::{
    Seat,
    v12::{MatchResult, MatchWinnerReason, RoomPhase},
};
use tokio::sync::oneshot;

use super::super::{match_runtime::MatchAction, types::RoomError};
use super::test_support::*;

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn both_explicit_departures_commit_together_and_ack_after_one_boundary() {
    for reverse in [false, true] {
        let value = started();
        let (first, second, one, two) = start_round(&value.handle).await;
        assert_running(&value.notices);
        let (mut first_reply, mut second_reply) = if reverse {
            (
                value
                    .handle
                    .leave(two, second.room_id.clone(), second.round_id),
                value
                    .handle
                    .leave(one, first.room_id.clone(), first.round_id),
            )
        } else {
            (
                value
                    .handle
                    .leave(one, first.room_id.clone(), first.round_id),
                value
                    .handle
                    .leave(two, second.room_id.clone(), second.round_id),
            )
        };
        tokio::task::yield_now().await;
        assert_eq!(
            first_reply.try_recv(),
            Err(oneshot::error::TryRecvError::Empty)
        );
        assert_eq!(
            second_reply.try_recv(),
            Err(oneshot::error::TryRecvError::Empty)
        );
        advance_ticks(1).await;
        assert_eq!(first_reply.await.unwrap(), Ok(()));
        assert_eq!(second_reply.await.unwrap(), Ok(()));
        assert!(matches!(
            publication(&value.notices).room.phase,
            RoomPhase::MatchResult {
                result: MatchResult::Abandoned,
                ..
            }
        ));
        value.task.await.unwrap();
    }
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn mixed_explicit_and_transport_departures_forfeit_to_reserved_opponent_and_ack() {
    let value = started();
    let (first, second, one, _two) = start_round(&value.handle).await;
    let reply = value
        .handle
        .leave(one, first.room_id.clone(), first.round_id);
    drop(second);
    tokio::task::yield_now().await;
    advance_ticks(1).await;

    assert_eq!(reply.await.unwrap(), Ok(()));
    assert!(matches!(
        publication(&value.notices).room.phase,
        RoomPhase::MatchResult {
            result: MatchResult::Winner {
                seat: Seat::Two,
                reason: MatchWinnerReason::Forfeit
            },
            ..
        }
    ));
    value.task.await.unwrap();
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn accepted_explicit_reason_wins_over_the_later_guard_drop() {
    let value = started();
    let (first, second, _one, two) = start_round(&value.handle).await;
    let reply = value
        .handle
        .leave(two, second.room_id.clone(), second.round_id);
    tokio::task::yield_now().await;
    drop(second);
    advance_ticks(1).await;

    assert_eq!(reply.await.unwrap(), Ok(()));
    assert!(matches!(
        publication(&value.notices).room.phase,
        RoomPhase::MatchResult {
            result: MatchResult::Winner {
                seat: Seat::One,
                reason: MatchWinnerReason::Forfeit
            },
            ..
        }
    ));
    drop(first);
    value.task.abort();
    let _ = value.task.await;
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn pending_departure_fences_that_binding_but_preserves_opponent_commands() {
    let value = started();
    let (first, second, one, two) = start_round(&value.handle).await;
    let reply = value
        .handle
        .leave(one, first.room_id.clone(), first.round_id);
    tokio::task::yield_now().await;
    assert_eq!(
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
    );
    assert_eq!(
        value
            .handle
            .action(
                two,
                second.room_id.clone(),
                second.round_id,
                MatchAction::Movement(Some(grenadiers_core::game::Direction::West)),
            )
            .await
            .unwrap(),
        Ok(())
    );
    assert_eq!(
        value
            .handle
            .action(
                two,
                second.room_id.clone(),
                second.round_id,
                MatchAction::Rematch(true),
            )
            .await
            .unwrap(),
        Err(RoomError::StaleRoom)
    );
    advance_ticks(1).await;
    assert_eq!(reply.await.unwrap(), Ok(()));
    value.task.abort();
    let _ = value.task.await;
}

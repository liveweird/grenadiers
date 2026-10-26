use std::sync::atomic::Ordering;

use grenadiers_core::{
    game::{Direction, SeatId},
    protocol::{Counter, MAX_COUNTER, v11::ConnectionStatus},
};
use tokio::sync::{oneshot, watch};

use super::{
    MatchAdmission,
    test_support::{Harness, advance_ticks, binding, publication, start_round, started},
};
use crate::game::rooms::{match_runtime::MatchAction, types::RoomError};

fn sequence(value: u32) -> Counter {
    Counter::new(value).unwrap()
}

async fn stop(value: Harness) {
    value.shutdown.send_replace(true);
    value.task.await.unwrap();
    assert_eq!(value.cleaned.load(Ordering::SeqCst), 1);
}

fn assert_no_bomb_event(notices: &watch::Receiver<super::MatchOwnerNotice>) {
    let frame = publication(notices).frame(SeatId::One).unwrap();
    let snapshot: serde_json::Value = serde_json::from_str(&frame).unwrap();
    assert!(
        snapshot["events"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| { event["kind"]["type"] != "bombPlaced" })
    );
}

async fn action(
    value: &Harness,
    admission: &MatchAdmission,
    owner: crate::game::rooms::types::BindingKey,
    action: MatchAction,
) -> Result<(), RoomError> {
    value
        .handle
        .action(owner, admission.room_id.clone(), admission.round_id, action)
        .await
        .unwrap()
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn queued_connected_resumes_commit_fifo_and_only_the_last_binding_survives() {
    for reverse_bindings in [false, true] {
        let value = started();
        let (mut original, opponent, original_binding, _) = start_round(&value.handle).await;
        let room_id = original.room_id.clone();
        let round_id = original.round_id;
        let token = original.resume_token.clone();
        let baseline_revision = publication(&value.notices).room.revision.get();
        let queued = value.handle.action(
            original_binding,
            room_id.clone(),
            round_id,
            MatchAction::Bomb(MAX_COUNTER),
        );
        let [first_binding, final_binding] = if reverse_bindings {
            [binding(4, 1), binding(3, 1)]
        } else {
            [binding(3, 1), binding(4, 1)]
        };
        let first_reply =
            value
                .handle
                .resume(first_binding, sequence(7), room_id.clone(), token.clone());
        let final_reply =
            value
                .handle
                .resume(final_binding, sequence(8), room_id.clone(), token.clone());

        assert_eq!(queued.await.unwrap(), Ok(()));
        let final_admission = final_reply.await.unwrap().unwrap();
        let mut first_admission = first_reply.await.unwrap().unwrap();
        assert_eq!(first_admission.seat, original.seat);
        assert_eq!(final_admission.seat, original.seat);
        assert_eq!(first_admission.round_id, round_id);
        assert_eq!(final_admission.round_id, round_id);
        assert_eq!(first_admission.resume_token, token);
        assert_eq!(final_admission.resume_token, token);
        assert_eq!(
            first_admission.last_bomb_action_id,
            Some(sequence(MAX_COUNTER))
        );
        assert_eq!(
            final_admission.last_bomb_action_id,
            Some(sequence(MAX_COUNTER))
        );
        assert_eq!(first_admission.room.revision.get(), baseline_revision + 1);
        assert_eq!(final_admission.room.revision.get(), baseline_revision + 2);
        assert_eq!((&mut original.replaced).await, Ok(()));
        assert_eq!((&mut first_admission.replaced).await, Ok(()));

        assert!(matches!(
            action(
                &value,
                &original,
                original_binding,
                MatchAction::Movement(None)
            )
            .await,
            Err(RoomError::StaleRoom)
        ));
        assert!(matches!(
            action(
                &value,
                &first_admission,
                first_binding,
                MatchAction::Movement(None)
            )
            .await,
            Err(RoomError::StaleRoom)
        ));
        assert_eq!(
            action(
                &value,
                &final_admission,
                final_binding,
                MatchAction::Movement(Some(Direction::East)),
            )
            .await,
            Ok(())
        );
        assert!(matches!(
            action(
                &value,
                &final_admission,
                final_binding,
                MatchAction::Bomb(MAX_COUNTER),
            )
            .await,
            Err(RoomError::DuplicateAction)
        ));

        drop(original.guard);
        advance_ticks(1).await;
        drop(first_admission.guard);
        advance_ticks(2).await;
        assert_no_bomb_event(&value.notices);
        assert_eq!(
            publication(&value.notices).room.seats[0].connection,
            ConnectionStatus::Connected
        );
        assert_eq!(
            action(
                &value,
                &final_admission,
                final_binding,
                MatchAction::Movement(None)
            )
            .await,
            Ok(())
        );

        drop(final_admission.guard);
        drop(opponent);
        stop(value).await;
    }
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn canceled_queued_resume_never_commits_or_evicts_the_valid_binding() {
    for canceled_first in [true, false] {
        let value = started();
        let mut original = super::test_support::join(&value.handle, binding(1, 1)).await;
        let baseline_revision = original.room.revision.get();
        let canceled_binding = binding(2, 1);
        let valid_binding = binding(3, 1);
        let canceled = || {
            value.handle.resume(
                canceled_binding,
                sequence(2),
                original.room_id.clone(),
                original.resume_token.clone(),
            )
        };
        let valid = || {
            value.handle.resume(
                valid_binding,
                sequence(3),
                original.room_id.clone(),
                original.resume_token.clone(),
            )
        };
        let valid_reply = if canceled_first {
            drop(canceled());
            valid()
        } else {
            let reply = valid();
            drop(canceled());
            reply
        };

        let valid_admission = valid_reply.await.unwrap().unwrap();
        assert_eq!(valid_admission.room.revision.get(), baseline_revision + 1);
        assert_eq!(
            publication(&value.notices).room.revision.get(),
            baseline_revision + 1
        );
        assert_eq!((&mut original.replaced).await, Ok(()));
        assert!(matches!(
            action(
                &value,
                &original,
                canceled_binding,
                MatchAction::Ready(false)
            )
            .await,
            Err(RoomError::StaleRoom)
        ));
        assert_eq!(
            action(
                &value,
                &valid_admission,
                valid_binding,
                MatchAction::Ready(false)
            )
            .await,
            Ok(())
        );
        drop(original.guard);
        advance_ticks(1).await;
        assert_eq!(
            action(
                &value,
                &valid_admission,
                valid_binding,
                MatchAction::Ready(false)
            )
            .await,
            Ok(())
        );

        drop(valid_admission);
        stop(value).await;
    }
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn transport_loss_settles_before_two_queued_resumes_restore_then_replace() {
    let value = started();
    let (mut original, opponent, original_binding, _) = start_round(&value.handle).await;
    let room_id = original.room_id.clone();
    let round_id = original.round_id;
    let token = original.resume_token.clone();
    drop(original.guard);
    let first_binding = binding(3, 1);
    let final_binding = binding(4, 1);
    let mut first_reply =
        value
            .handle
            .resume(first_binding, sequence(4), room_id.clone(), token.clone());
    let mut final_reply =
        value
            .handle
            .resume(final_binding, sequence(5), room_id.clone(), token.clone());

    tokio::task::yield_now().await;
    assert!(matches!(
        first_reply.try_recv(),
        Err(oneshot::error::TryRecvError::Empty)
    ));
    assert!(matches!(
        final_reply.try_recv(),
        Err(oneshot::error::TryRecvError::Empty)
    ));
    advance_ticks(1).await;
    let mut first_admission = first_reply.await.unwrap().unwrap();
    let final_admission = final_reply.await.unwrap().unwrap();
    assert_eq!(first_admission.resume_token, token);
    assert_eq!(final_admission.resume_token, token);
    assert_eq!(first_admission.round_id, round_id);
    assert_eq!(final_admission.round_id, round_id);
    assert!(first_admission.room.revision < final_admission.room.revision);
    assert_eq!((&mut first_admission.replaced).await, Ok(()));
    assert!(matches!(
        original.replaced.try_recv(),
        Err(oneshot::error::TryRecvError::Closed)
    ));
    assert!(matches!(
        value
            .handle
            .action(
                original_binding,
                room_id,
                round_id,
                MatchAction::Movement(None),
            )
            .await
            .unwrap(),
        Err(RoomError::StaleRoom)
    ));
    assert!(matches!(
        action(
            &value,
            &first_admission,
            first_binding,
            MatchAction::Movement(None)
        )
        .await,
        Err(RoomError::StaleRoom)
    ));
    assert_eq!(
        action(
            &value,
            &final_admission,
            final_binding,
            MatchAction::Movement(None)
        )
        .await,
        Ok(())
    );

    drop(first_admission.guard);
    drop(final_admission.guard);
    drop(opponent);
    stop(value).await;
}

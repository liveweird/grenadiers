use std::sync::atomic::Ordering;

use grenadiers_core::{game::SpeedLevel, protocol::RoomId, room::RoomSettings};

use super::super::{match_runtime::MatchAction, types::RoomError};
use super::test_support::*;

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn admission_validates_room_epoch_binding_and_generation() {
    let value = started();
    let first_binding = binding(1, 7);
    let first = join(&value.handle, first_binding).await;
    assert_eq!(first.seat, grenadiers_core::protocol::Seat::One);
    assert_eq!(first.room.room_id, first.room_id);
    assert!(matches!(
        &*first.notices.borrow(),
        super::MatchOwnerNotice::Publication(_)
    ));
    let _guard_is_retained = &first.guard;
    let wrong_room = RoomId::new("00000000000000000000000000000002").unwrap();

    assert_eq!(
        value
            .handle
            .action(
                binding(1, 6),
                first.room_id.clone(),
                first.round_id,
                MatchAction::Ready(true),
            )
            .await
            .unwrap(),
        Err(RoomError::StaleRoom)
    );
    assert_eq!(
        value
            .handle
            .action(
                first_binding,
                wrong_room,
                first.round_id,
                MatchAction::Ready(true),
            )
            .await
            .unwrap(),
        Err(RoomError::StaleRoom)
    );
    assert_eq!(
        value
            .handle
            .action(
                first_binding,
                first.room_id.clone(),
                first.round_id + 1,
                MatchAction::Ready(true),
            )
            .await
            .unwrap(),
        Err(RoomError::StaleRoom)
    );
    assert_eq!(
        value
            .handle
            .action(
                first_binding,
                first.room_id.clone(),
                first.round_id,
                MatchAction::Ready(true),
            )
            .await
            .unwrap(),
        Ok(())
    );

    value.task.abort();
    let _ = value.task.await;
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn settings_update_rejects_a_forged_binding_before_editor_authorization() {
    let (handle, owner, shutdown) = prepared_with(
        super::super::profile::RoomProfile::GeneratedConfigurableV4,
        crate::round_seed::RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(9);
            Ok(())
        }),
    );
    let value = start_prepared(handle, owner, shutdown);
    let creator_binding = binding(1, 7);
    let creator = join(&value.handle, creator_binding).await;
    let action = MatchAction::UpdateSettings {
        settings_revision: 1,
        settings: RoomSettings::new(SpeedLevel::One, 192, 2, 150).unwrap(),
    };
    assert_eq!(
        value
            .handle
            .action(
                binding(1, 6),
                creator.room_id.clone(),
                creator.round_id,
                action,
            )
            .await
            .unwrap(),
        Err(RoomError::StaleRoom)
    );
    assert_eq!(
        value
            .handle
            .action(
                creator_binding,
                creator.room_id.clone(),
                creator.round_id,
                action,
            )
            .await
            .unwrap(),
        Ok(())
    );
    value.task.abort();
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn dropped_admission_reply_rolls_back_on_the_next_boundary_then_cleans_up() {
    let value = started();
    let response = value.handle.join(binding(1, 1));
    drop(response);
    tokio::task::yield_now().await;
    advance_ticks(1).await;

    value.task.await.unwrap();
    assert_eq!(value.cleaned.load(Ordering::SeqCst), 1);
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn canceled_queued_join_does_not_mutate_before_the_next_join_in_the_same_batch() {
    let (handle, owner, _shutdown) = prepared();
    let canceled = handle.join(binding(1, 1));
    drop(canceled);
    let accepted = handle.join(binding(2, 1));
    let task = owner.start(|| {});

    let admission = accepted.await.unwrap().unwrap();
    assert_eq!(admission.seat, grenadiers_core::protocol::Seat::One);
    assert_eq!(admission.room.revision.get(), 2);
    assert!(admission.room.seats[0].occupied);
    assert!(!admission.room.seats[1].occupied);

    task.abort();
    let _ = task.await;
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn abort_before_first_poll_runs_cleanup_exactly_once() {
    let (handle, owner, _shutdown) = prepared();
    drop(handle);
    let cleaned = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let observed = cleaned.clone();
    let task = owner.start(move || {
        observed.fetch_add(1, Ordering::SeqCst);
    });
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    assert_eq!(cleaned.load(Ordering::SeqCst), 1);
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn full_command_queue_returns_a_synchronous_rate_limit() {
    let (handle, _owner, _shutdown) = prepared();
    let mut pending = Vec::new();
    for generation in 1..=32 {
        pending.push(handle.join(binding(generation, 1)));
    }
    let mut rejected = handle.join(binding(33, 1));
    assert!(matches!(
        rejected.try_recv(),
        Ok(Err(RoomError::RateLimited))
    ));
    drop(pending);
}

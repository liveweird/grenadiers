use std::{sync::atomic::Ordering, time::Duration};

use grenadiers_core::protocol::v8::{MatchInterruptReason, MatchResult, RoomPhase};
use tokio::time::advance;

use super::{ClosingReason, MatchOwnerNotice, test_support::*};

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn empty_lobby_expires_at_creation_deadline_and_retains_the_closing_view() {
    let value = started();
    advance(Duration::from_secs(300)).await;
    tokio::task::yield_now().await;
    value.task.await.unwrap();

    assert!(
        matches!(&*value.notices.borrow(), MatchOwnerNotice::Closing {
        reason: ClosingReason::Expired, last, ..
    } if matches!(last.room.phase, RoomPhase::Waiting))
    );
    assert_eq!(value.cleaned.load(Ordering::SeqCst), 1);
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn overload_boundary_interrupts_before_overdue_countdown_ticks() {
    let value = started();
    let first = join(&value.handle, binding(1, 1)).await;
    let second = join(&value.handle, binding(2, 1)).await;
    value
        .handle
        .action(
            binding(1, 1),
            first.room_id.clone(),
            first.round_id,
            super::super::match_runtime::MatchAction::Ready(true),
        )
        .await
        .unwrap()
        .unwrap();
    value
        .handle
        .action(
            binding(2, 1),
            second.room_id.clone(),
            second.round_id,
            super::super::match_runtime::MatchAction::Ready(true),
        )
        .await
        .unwrap()
        .unwrap();

    advance(Duration::from_millis(517)).await;
    tokio::task::yield_now().await;
    assert!(matches!(
        publication(&value.notices).room.phase,
        RoomPhase::Interrupted {
            reason: MatchInterruptReason::ServerOverloaded,
            ..
        }
    ));
    assert_eq!(publication(&value.notices).room.tick.get(), 0);
    value.task.abort();
    let _ = value.task.await;
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn overload_interrupt_commits_an_accepted_prematch_leave_without_requesting_entropy() {
    let entropy = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let observed = entropy.clone();
    let source = crate::round_seed::RoundSeedSource::with_entropy(move |_| {
        observed.fetch_add(1, Ordering::SeqCst);
        Ok(())
    });
    let (handle, owner, shutdown) = prepared_with(
        super::super::profile::RoomProfile::GeneratedClassicV2,
        source,
    );
    let value = start_prepared(handle, owner, shutdown);
    let first = join(&value.handle, binding(1, 1)).await;
    let second = join(&value.handle, binding(2, 1)).await;
    for (binding, admission) in [(binding(1, 1), &first), (binding(2, 1), &second)] {
        value
            .handle
            .action(
                binding,
                admission.room_id.clone(),
                admission.round_id,
                super::super::match_runtime::MatchAction::Ready(true),
            )
            .await
            .unwrap()
            .unwrap();
    }
    let leave = value
        .handle
        .leave(binding(1, 1), first.room_id.clone(), first.round_id);
    tokio::task::yield_now().await;

    advance(Duration::from_millis(517)).await;
    tokio::task::yield_now().await;
    assert_eq!(leave.await.unwrap(), Ok(()));
    let room = publication(&value.notices).room;
    assert!(matches!(
        room.phase,
        RoomPhase::Interrupted {
            reason: MatchInterruptReason::ServerOverloaded,
            ..
        }
    ));
    assert!(!room.seats[0].occupied);
    assert!(room.seats[1].occupied);
    assert_eq!(entropy.load(Ordering::SeqCst), 0);
    value.task.abort();
    let _ = value.task.await;
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn sticky_shutdown_bypasses_commands_acks_pending_leave_and_retains_final_closing() {
    let value = started();
    let (first, second, one, _two) = start_round(&value.handle).await;
    drop(second.guard);
    let leave = value
        .handle
        .leave(one, first.room_id.clone(), first.round_id);
    tokio::task::yield_now().await;
    value.shutdown.send_replace(true);
    tokio::task::yield_now().await;

    assert_eq!(leave.await.unwrap(), Ok(()));
    value.task.await.unwrap();
    assert!(
        matches!(&*value.notices.borrow(), MatchOwnerNotice::Closing {
            reason: ClosingReason::ServerStopped,
            last,
        } if matches!(last.room.phase, RoomPhase::MatchResult {
            result: MatchResult::Interrupted { reason: MatchInterruptReason::ServerStopped }, ..
        }) && last.room.seats.iter().all(|seat| !seat.occupied))
    );
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn closed_shutdown_sender_is_sticky_before_admission() {
    let (handle, owner, shutdown) = prepared();
    let notices = owner.notices.subscribe();
    drop(shutdown);
    let task = owner.start(|| {});
    let response = handle.join(binding(1, 1));
    tokio::task::yield_now().await;

    assert!(matches!(
        response.await.unwrap(),
        Err(super::super::types::RoomError::RoomNotFound)
    ));
    task.await.unwrap();
    assert!(matches!(
        &*notices.borrow(),
        MatchOwnerNotice::Closing {
            reason: ClosingReason::ServerStopped,
            ..
        }
    ));
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn countdown_publication_runs_at_twenty_hertz_while_ticks_run_at_sixty() {
    let value = started();
    let first = join(&value.handle, binding(1, 1)).await;
    let second = join(&value.handle, binding(2, 1)).await;
    value
        .handle
        .action(
            binding(1, 1),
            first.room_id.clone(),
            first.round_id,
            super::super::match_runtime::MatchAction::Ready(true),
        )
        .await
        .unwrap()
        .unwrap();
    value
        .handle
        .action(
            binding(2, 1),
            second.room_id.clone(),
            second.round_id,
            super::super::match_runtime::MatchAction::Ready(true),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(publication(&value.notices).room.tick.get(), 0);
    advance_ticks(2).await;
    assert_eq!(publication(&value.notices).room.tick.get(), 0);
    advance_ticks(1).await;
    assert_eq!(publication(&value.notices).room.tick.get(), 3);
    value.task.abort();
    let _ = value.task.await;
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn repeated_full_command_batches_do_not_starve_running_ticks() {
    let value = started();
    let (first, _second, one, _two) = start_round(&value.handle).await;
    let initial_tick = publication(&value.notices).room.tick.get();
    for _ in 0..4 {
        let replies = (0..32)
            .map(|_| {
                value.handle.action(
                    one,
                    first.room_id.clone(),
                    first.round_id,
                    super::super::match_runtime::MatchAction::Movement(None),
                )
            })
            .collect::<Vec<_>>();
        advance_ticks(3).await;
        for reply in replies {
            assert!(matches!(
                reply.await.unwrap(),
                Ok(()) | Err(super::super::types::RoomError::RateLimited)
            ));
        }
    }
    assert!(publication(&value.notices).room.tick.get() >= initial_tick + 12);
    value.task.abort();
    let _ = value.task.await;
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn shutdown_closing_retains_the_snapshot_that_flushes_pending_reports() {
    let value = started();
    let (first, _second, one, _two) = start_round(&value.handle).await;
    value
        .handle
        .action(
            one,
            first.room_id.clone(),
            first.round_id,
            super::super::match_runtime::MatchAction::Bomb(1),
        )
        .await
        .unwrap()
        .unwrap();
    tokio::time::advance(Duration::from_millis(34)).await;
    for _ in 0..3 {
        tokio::task::yield_now().await;
    }
    let leave = value
        .handle
        .leave(one, first.room_id.clone(), first.round_id);
    tokio::task::yield_now().await;
    value.shutdown.send_replace(true);
    tokio::task::yield_now().await;
    assert_eq!(leave.await.unwrap(), Ok(()));
    value.task.await.unwrap();

    let MatchOwnerNotice::Closing { last, .. } = &*value.notices.borrow() else {
        panic!("shutdown must retain a closing notice");
    };
    let snapshot: serde_json::Value = serde_json::from_str(
        last.frame(grenadiers_core::game::SeatId::One)
            .as_ref()
            .expect("shutdown flushes the live state"),
    )
    .unwrap();
    assert!(!last.room.seats[0].occupied);
    assert!(
        snapshot["events"]
            .as_array()
            .is_some_and(|events| !events.is_empty())
    );
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn shutdown_after_a_final_match_keeps_the_winner_while_committing_vacancy() {
    let value = started();
    let (first, second, one, two) = start_round(&value.handle).await;
    for completed in 0..3 {
        finish_round(&value.handle, &value.notices, two, &second).await;
        if completed < 2 {
            advance_to_running(&value.notices).await;
        }
    }
    let epoch = publication(&value.notices).room.round_id.get() as u64;
    let leave = value.handle.leave(one, first.room_id.clone(), epoch);
    tokio::task::yield_now().await;
    value.shutdown.send_replace(true);
    tokio::task::yield_now().await;

    assert_eq!(leave.await.unwrap(), Ok(()));
    value.task.await.unwrap();
    assert!(matches!(
        &*value.notices.borrow(),
        MatchOwnerNotice::Closing { last, .. }
            if !last.room.seats[0].occupied
                && matches!(last.room.phase, RoomPhase::MatchResult {
                    result: MatchResult::Winner { .. }, ..
                })
    ));
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn sticky_shutdown_rejects_every_waiter_even_when_the_command_queue_is_full() {
    let (handle, owner, shutdown) = prepared();
    let notices = owner.notices.subscribe();
    let responses = (1..=32)
        .map(|id| handle.join(binding(id, 1)))
        .collect::<Vec<_>>();
    shutdown.send_replace(true);
    let task = owner.start(|| {});
    task.await.unwrap();

    for response in responses {
        assert!(matches!(
            response.await.unwrap(),
            Err(super::super::types::RoomError::RoomNotFound)
        ));
    }
    assert!(matches!(
        &*notices.borrow(),
        MatchOwnerNotice::Closing {
            reason: ClosingReason::ServerStopped,
            ..
        }
    ));
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn terminal_match_retires_once_after_thirty_seconds_with_its_final_result() {
    let value = started();
    let (_first, second, _one, two) = start_round(&value.handle).await;
    for completed in 0..3 {
        finish_round(&value.handle, &value.notices, two, &second).await;
        if completed < 2 {
            advance_to_running(&value.notices).await;
        }
    }
    tokio::time::advance(Duration::from_secs(30)).await;
    tokio::task::yield_now().await;
    value.task.await.unwrap();

    assert!(
        matches!(&*value.notices.borrow(), MatchOwnerNotice::Closing {
            reason: ClosingReason::Retired,
            last,
        } if matches!(last.room.phase, RoomPhase::MatchResult {
            result: MatchResult::Winner { .. }, ..
        }))
    );
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn a_nonfinal_round_result_does_not_start_the_match_retirement_deadline() {
    let value = started();
    let (_first, second, _one, two) = start_round(&value.handle).await;
    finish_round(&value.handle, &value.notices, two, &second).await;
    for _ in 0..78 {
        advance(Duration::from_millis(400)).await;
        for _ in 0..8 {
            tokio::task::yield_now().await;
        }
    }

    assert!(!value.task.is_finished());
    assert!(!matches!(
        &*value.notices.borrow(),
        MatchOwnerNotice::Closing { .. }
    ));
    value.task.abort();
    let _ = value.task.await;
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn started_match_survives_beyond_the_original_creation_expiry() {
    let value = started();
    let _admissions = start_round(&value.handle).await;
    for _ in 0..753 {
        tokio::time::advance(Duration::from_millis(400)).await;
        for _ in 0..8 {
            tokio::task::yield_now().await;
        }
    }

    assert!(!value.task.is_finished());
    assert!(matches!(
        &*value.notices.borrow(),
        MatchOwnerNotice::Publication(_)
    ));
    value.task.abort();
    let _ = value.task.await;
}

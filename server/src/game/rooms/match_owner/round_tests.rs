use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

use grenadiers_core::protocol::{
    Seat,
    v12::{CountdownKind, MatchResult, MatchWinnerReason, RoomPhase},
};

use super::test_support::*;
use crate::{
    game::rooms::{match_runtime::MatchAction, profile::RoomProfile},
    round_seed::RoundSeedSource,
};

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn departure_after_one_or_two_wins_retains_scores_but_has_a_forfeit_match_outcome() {
    for expected_wins in [1, 2] {
        let value = started();
        let (first, second, _one, two) = start_round(&value.handle).await;
        for completed in 0..expected_wins {
            finish_round(&value.handle, &value.notices, two, &second).await;
            if completed + 1 < expected_wins {
                advance_to_running(&value.notices).await;
            }
        }
        let mut leave = value.handle.leave(
            two,
            second.room_id.clone(),
            publication(&value.notices).room.round_id.get() as u64,
        );
        tokio::task::yield_now().await;
        assert!(matches!(
            leave.try_recv(),
            Err(tokio::sync::oneshot::error::TryRecvError::Empty)
        ));
        advance_ticks(1).await;
        assert_eq!(leave.await.unwrap(), Ok(()));
        let room = publication(&value.notices).room;
        assert_eq!(room.scores.one.get(), expected_wins);
        assert!(matches!(
            room.phase,
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
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn third_real_round_win_is_final_before_any_departure_batch() {
    let value = started();
    let (_first, second, _one, two) = start_round(&value.handle).await;
    for completed in 0..3 {
        finish_round(&value.handle, &value.notices, two, &second).await;
        if completed < 2 {
            advance_to_running(&value.notices).await;
        }
    }
    let room = publication(&value.notices).room;
    assert_eq!(room.scores.one.get(), 3);
    assert!(matches!(
        room.phase,
        RoomPhase::MatchResult {
            result: MatchResult::Winner {
                seat: Seat::One,
                reason: MatchWinnerReason::RoundWins
            },
            ..
        }
    ));
    value.task.abort();
    let _ = value.task.await;
}

fn generated(counter: Arc<AtomicUsize>) -> super::test_support::Harness {
    let source = RoundSeedSource::with_entropy(move |bytes| {
        let call = counter.fetch_add(1, Ordering::SeqCst);
        bytes.fill((call + 1) as u8);
        Ok(())
    });
    let (handle, owner, shutdown) = prepared_with(RoomProfile::GeneratedClassicV2, source);
    start_prepared(handle, owner, shutdown)
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn departure_on_initial_countdown_boundary_consumes_no_seed_or_actual_round() {
    let counter = Arc::new(AtomicUsize::new(0));
    let value = generated(counter.clone());
    let first = join(&value.handle, binding(1, 1)).await;
    let second = join(&value.handle, binding(2, 1)).await;
    value
        .handle
        .action(
            binding(1, 1),
            first.room_id.clone(),
            first.round_id,
            MatchAction::Ready(true),
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
            MatchAction::Ready(true),
        )
        .await
        .unwrap()
        .unwrap();
    advance_ticks(179).await;
    let leave = value
        .handle
        .leave(binding(2, 1), second.room_id.clone(), second.round_id);
    tokio::task::yield_now().await;
    advance_ticks(1).await;

    assert_eq!(leave.await.unwrap(), Ok(()));
    assert_eq!(counter.load(Ordering::SeqCst), 0);
    let room = publication(&value.notices).room;
    assert_eq!(room.round_number.get(), 0);
    assert!(matches!(room.phase, RoomPhase::Waiting));
    value.task.abort();
    let _ = value.task.await;
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn departure_on_inter_round_boundary_does_not_request_the_next_seed() {
    let counter = Arc::new(AtomicUsize::new(0));
    let value = generated(counter.clone());
    let (_first, second, _one, two) = start_round(&value.handle).await;
    assert_eq!(counter.load(Ordering::SeqCst), 1);
    finish_round(&value.handle, &value.notices, two, &second).await;
    for _ in 0..200 {
        advance_ticks(1).await;
        if matches!(
            publication(&value.notices).room.phase,
            RoomPhase::Countdown {
                kind: CountdownKind::InterRound,
                ..
            }
        ) {
            break;
        }
    }
    assert!(matches!(
        publication(&value.notices).room.phase,
        RoomPhase::Countdown {
            kind: CountdownKind::InterRound,
            ..
        }
    ));
    advance_ticks(179).await;
    let epoch = publication(&value.notices).room.round_id.get() as u64;
    let leave = value.handle.leave(two, second.room_id.clone(), epoch);
    tokio::task::yield_now().await;
    advance_ticks(1).await;

    assert_eq!(leave.await.unwrap(), Ok(()));
    assert_eq!(counter.load(Ordering::SeqCst), 1);
    assert!(matches!(
        publication(&value.notices).room.phase,
        RoomPhase::MatchResult { .. }
    ));
    value.task.abort();
    let _ = value.task.await;
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn successful_rematch_retains_global_clock_and_starts_a_fresh_lobby_deadline() {
    let value = started();
    let (_first, second, one, two) = start_round(&value.handle).await;
    for completed in 0..3 {
        finish_round(&value.handle, &value.notices, two, &second).await;
        if completed < 2 {
            advance_to_running(&value.notices).await;
        }
    }
    let terminal_tick = publication(&value.notices).room.tick.get();
    advance_ticks(180).await;
    let terminal = publication(&value.notices).room;
    let epoch = terminal.round_id.get() as u64;
    value
        .handle
        .action(
            one,
            terminal.room_id.clone(),
            epoch,
            MatchAction::Rematch(true),
        )
        .await
        .unwrap()
        .unwrap();
    value
        .handle
        .action(two, terminal.room_id, epoch, MatchAction::Rematch(true))
        .await
        .unwrap()
        .unwrap();

    let fresh = publication(&value.notices).room;
    assert!(matches!(fresh.phase, RoomPhase::Waiting));
    assert_eq!(fresh.round_number.get(), 0);
    assert_eq!(fresh.scores.one.get(), 0);
    assert!(fresh.tick.get() >= terminal_tick + 180);

    tokio::time::advance(Duration::from_secs(300)).await;
    tokio::task::yield_now().await;
    value.task.await.unwrap();
    assert!(matches!(
        &*value.notices.borrow(),
        super::MatchOwnerNotice::Closing {
            reason: super::ClosingReason::Expired,
            ..
        }
    ));
}

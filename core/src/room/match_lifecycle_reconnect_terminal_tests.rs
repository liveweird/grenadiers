use super::*;
use crate::room::reconnect::ReservationState;
use crate::room::series::MatchResult;

fn occupied() -> MatchLifecycle {
    let mut lifecycle = MatchLifecycle::new();
    lifecycle.occupy(SeatId::One).unwrap();
    lifecycle.occupy(SeatId::Two).unwrap();
    lifecycle
}

fn advance_idle(lifecycle: &mut MatchLifecycle, ticks: u64) {
    for _ in 0..ticks {
        lifecycle.advance_idle_tick().unwrap();
    }
}

fn advance_running(lifecycle: &mut MatchLifecycle, ticks: u64) {
    for _ in 0..ticks {
        lifecycle.complete_running_tick(None).unwrap();
    }
}

fn start(lifecycle: &mut MatchLifecycle) {
    lifecycle.set_ready(SeatId::One, true).unwrap();
    lifecycle.set_ready(SeatId::Two, true).unwrap();
    advance_idle(lifecycle, COUNTDOWN_TICKS);
}

fn running_with_wins(wins: u8) -> MatchLifecycle {
    let mut lifecycle = occupied();
    start(&mut lifecycle);
    for _ in 0..wins {
        lifecycle
            .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
            .unwrap();
        advance_idle(&mut lifecycle, COUNTDOWN_TICKS * 2);
    }
    lifecycle
}

fn score_finished() -> MatchLifecycle {
    let mut lifecycle = running_with_wins(2);
    lifecycle
        .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    lifecycle
}

#[test]
fn round_scoring_precedes_expiry_and_third_win_precedes_forfeit() {
    for prior_wins in [0, 1] {
        let mut lifecycle = running_with_wins(prior_wins);
        lifecycle
            .depart_with_grace([None, Some(DepartureReason::TransportLoss)])
            .unwrap();
        advance_running(&mut lifecycle, 599);
        lifecycle
            .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
            .unwrap();
        let record = lifecycle.series().latest_completed_round().unwrap();
        let expected_score = prior_wins + 1;
        let MatchChange::MatchFinished(outcome) = lifecycle.expire_reconnect_grace().unwrap()
        else {
            panic!("expiry after nonfinal score must forfeit");
        };
        assert_eq!(outcome.result(), MatchResult::ForfeitWinner(SeatId::One));
        assert_eq!(lifecycle.series().scores(), [expected_score, 0]);
        assert_eq!(record.result(), RoundResult::Winner(SeatId::One));
    }

    let mut third = running_with_wins(2);
    third
        .depart_with_grace([None, Some(DepartureReason::TransportLoss)])
        .unwrap();
    advance_running(&mut third, 599);
    third
        .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    let outcome = third.series().outcome().unwrap();
    assert_eq!(outcome.result(), MatchResult::ScoreWinner(SeatId::One));
    assert_eq!(third.expire_reconnect_grace(), Ok(MatchChange::Changed));
    assert_eq!(third.series().outcome(), Some(outcome));
    assert_eq!(third.series().scores(), [3, 0]);
}

#[test]
fn draw_on_expiry_tick_is_recorded_before_the_match_forfeit() {
    let mut lifecycle = occupied();
    start(&mut lifecycle);
    lifecycle
        .depart_with_grace([None, Some(DepartureReason::TransportLoss)])
        .unwrap();
    advance_running(&mut lifecycle, 599);
    let draw = RoundResult::Draw(crate::game::DrawReason::MutualElimination);
    lifecycle.complete_running_tick(Some(draw)).unwrap();
    let MatchChange::MatchFinished(outcome) = lifecycle.expire_reconnect_grace().unwrap() else {
        panic!("expiry settles after drawing round");
    };
    assert_eq!(outcome.result(), MatchResult::ForfeitWinner(SeatId::One));
    assert_eq!(lifecycle.series().scores(), [0, 0]);
    assert_eq!(
        lifecycle
            .series()
            .latest_completed_round()
            .unwrap()
            .result(),
        draw
    );
}

#[test]
fn whole_same_tick_loss_batch_is_applied_before_expiry() {
    for (due, later) in [(SeatId::One, SeatId::Two), (SeatId::Two, SeatId::One)] {
        let mut lifecycle = occupied();
        start(&mut lifecycle);
        let mut first = [None; 2];
        first[due.index()] = Some(DepartureReason::TransportLoss);
        lifecycle.depart_with_grace(first).unwrap();
        advance_running(&mut lifecycle, 600);

        let mut batch = [None; 2];
        batch[later.index()] = Some(DepartureReason::TransportLoss);
        assert_eq!(lifecycle.depart_with_grace(batch), Ok(MatchChange::Changed));
        assert_eq!(lifecycle.series().outcome(), None);
        assert_eq!(
            lifecycle.reconnect_grace().seat(due),
            Some(ReservationState::Expired)
        );
        assert!(matches!(
            lifecycle.reconnect_grace().seat(later),
            Some(ReservationState::Disconnected { .. })
        ));
    }
}

#[test]
fn grace_departure_policy_handles_explicit_and_mixed_batches() {
    let cases = [
        (
            [Some(DepartureReason::Explicit), None],
            MatchResult::ForfeitWinner(SeatId::Two),
        ),
        (
            [None, Some(DepartureReason::Explicit)],
            MatchResult::ForfeitWinner(SeatId::One),
        ),
        (
            [
                Some(DepartureReason::Explicit),
                Some(DepartureReason::Explicit),
            ],
            MatchResult::Abandoned,
        ),
        (
            [
                Some(DepartureReason::Explicit),
                Some(DepartureReason::TransportLoss),
            ],
            MatchResult::ForfeitWinner(SeatId::Two),
        ),
        (
            [
                Some(DepartureReason::TransportLoss),
                Some(DepartureReason::Explicit),
            ],
            MatchResult::ForfeitWinner(SeatId::One),
        ),
    ];
    for (batch, expected) in cases {
        let mut lifecycle = occupied();
        start(&mut lifecycle);
        let MatchChange::MatchFinished(outcome) = lifecycle.depart_with_grace(batch).unwrap()
        else {
            panic!("explicit departure after start is terminal");
        };
        assert_eq!(outcome.result(), expected);
    }

    let mut legacy = occupied();
    start(&mut legacy);
    let MatchChange::MatchFinished(outcome) = legacy
        .depart_batch([
            Some(DepartureReason::Explicit),
            Some(DepartureReason::TransportLoss),
        ])
        .unwrap()
    else {
        panic!("legacy departure remains terminal");
    };
    assert_eq!(
        outcome.result(),
        MatchResult::Interrupted(InterruptReason::ConnectionLost)
    );
}

#[test]
fn explicit_departure_against_expired_opponent_abandons_and_invalid_explicit_is_atomic() {
    let mut lifecycle = occupied();
    start(&mut lifecycle);
    lifecycle
        .depart_with_grace([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    advance_running(&mut lifecycle, 600);
    let MatchChange::MatchFinished(outcome) = lifecycle
        .depart_with_grace([None, Some(DepartureReason::Explicit)])
        .unwrap()
    else {
        panic!("no eligible participant remains");
    };
    assert_eq!(outcome.result(), MatchResult::Abandoned);

    let mut invalid = occupied();
    start(&mut invalid);
    invalid
        .depart_with_grace([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    let before = invalid.clone();
    assert_eq!(
        invalid.depart_with_grace([Some(DepartureReason::Explicit), None]),
        Err(MatchError::Reconnect(ReconnectError::SeatNotConnected(
            SeatId::One
        )))
    );
    assert_eq!(invalid, before);
}

#[test]
fn terminal_reservation_can_resume_but_new_terminal_loss_releases_immediately() {
    let mut lifecycle = running_with_wins(2);
    lifecycle
        .depart_with_grace([None, Some(DepartureReason::TransportLoss)])
        .unwrap();
    lifecycle
        .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    let outcome = lifecycle.series().outcome();
    assert_eq!(
        lifecycle.resume_connection(SeatId::Two),
        Ok(MatchChange::Changed)
    );
    assert_eq!(lifecycle.series().outcome(), outcome);
    assert_eq!(
        lifecycle.depart_with_grace([None, Some(DepartureReason::TransportLoss)]),
        Ok(MatchChange::Changed)
    );
    assert_eq!(
        lifecycle.reconnect_grace().seat(SeatId::Two),
        Some(ReservationState::Vacant)
    );
    assert_eq!(lifecycle.series().outcome(), outcome);
    assert_eq!(
        lifecycle.resume_connection(SeatId::Two),
        Err(MatchError::Reconnect(ReconnectError::SeatVacant(
            SeatId::Two
        )))
    );
}

#[test]
fn votes_require_connection_and_rematch_retains_grace_clock() {
    let mut lifecycle = score_finished();
    advance_idle(&mut lifecycle, COUNTDOWN_TICKS);
    assert_eq!(
        lifecycle.expire_reconnect_grace(),
        Ok(MatchChange::Unchanged)
    );
    let grace_tick = lifecycle.reconnect_grace().latest_tick();
    lifecycle.set_rematch_vote(SeatId::One, true).unwrap();
    lifecycle
        .depart_with_grace([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    assert_eq!(lifecycle.series().rematch_votes(), [false, false]);
    assert_eq!(
        lifecycle.set_rematch_vote(SeatId::One, true),
        Err(MatchError::SeatVacant(SeatId::One))
    );

    let mut rematch = score_finished();
    advance_idle(&mut rematch, COUNTDOWN_TICKS);
    rematch.expire_reconnect_grace().unwrap();
    rematch.set_rematch_vote(SeatId::One, true).unwrap();
    assert_eq!(
        rematch.set_rematch_vote(SeatId::Two, true),
        Ok(MatchChange::RematchStarted)
    );
    assert_eq!(rematch.reconnect_grace().latest_tick(), grace_tick);
    assert_eq!(rematch.series().scores(), [0, 0]);
    assert_eq!(rematch.series().actual_round_number(), 0);
    assert_eq!(
        rematch.depart_with_grace([Some(DepartureReason::TransportLoss), None]),
        Ok(MatchChange::Changed)
    );
    assert_eq!(
        rematch.reconnect_grace().seat(SeatId::One),
        Some(ReservationState::Vacant)
    );
}

#[test]
fn grace_deadline_cancellation_and_terminal_hold_overflow_are_atomic() {
    let at = Tick(u64::MAX - 599);
    let mut deadline = MatchLifecycle::new();
    deadline.tick = at;
    deadline.occupy(SeatId::One).unwrap();
    deadline.occupy(SeatId::Two).unwrap();
    deadline.series.start_round(1, at).unwrap();
    deadline.phase = MatchPhase::Running;
    let before = deadline.clone();
    assert_eq!(
        deadline.depart_with_grace([Some(DepartureReason::TransportLoss), None]),
        Err(MatchError::Reconnect(ReconnectError::TickOverflow))
    );
    assert_eq!(deadline, before);

    let mut cancellation = running_with_wins(0);
    cancellation
        .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    advance_idle(&mut cancellation, COUNTDOWN_TICKS);
    cancellation.round_attempt = u64::MAX;
    let before = cancellation.clone();
    assert_eq!(
        cancellation.depart_with_grace([Some(DepartureReason::TransportLoss), None]),
        Err(MatchError::RoundAttemptOverflow)
    );
    assert_eq!(cancellation, before);

    let at = Tick(u64::MAX - COUNTDOWN_TICKS + 1);
    let mut hold = MatchLifecycle::new();
    hold.tick = at;
    hold.occupy(SeatId::One).unwrap();
    hold.occupy(SeatId::Two).unwrap();
    hold.series.start_round(1, at).unwrap();
    hold.phase = MatchPhase::Running;
    let before = hold.clone();
    assert_eq!(
        hold.depart_with_grace([Some(DepartureReason::Explicit), None]),
        Err(MatchError::Series(SeriesError::TickOverflow))
    );
    assert_eq!(hold, before);
}

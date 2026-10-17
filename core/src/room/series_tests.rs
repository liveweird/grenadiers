use super::*;
use crate::game::DrawReason;

fn finish_round(
    series: &mut MatchSeries,
    epoch: u64,
    started_at: u64,
    ended_at: u64,
    result: RoundResult,
) {
    series.start_round(epoch, Tick(started_at)).unwrap();
    series
        .complete_round(epoch, result, Tick(ended_at))
        .unwrap();
}

fn score_finished_series(winner: SeatId) -> MatchSeries {
    let mut series = MatchSeries::new();
    for (epoch, started_at) in [(2, 0), (9, 181), (20, 362)] {
        finish_round(
            &mut series,
            epoch,
            started_at,
            started_at + 1,
            RoundResult::Winner(winner),
        );
    }
    series
}

#[test]
fn new_series_has_only_bounded_empty_facts() {
    let series = MatchSeries::new();

    assert_eq!(series.scores(), [0, 0]);
    assert_eq!(series.actual_round_number(), 0);
    assert_eq!(series.last_command_epoch(), 0);
    assert_eq!(series.active_round(), None);
    assert_eq!(series.latest_completed_round(), None);
    assert_eq!(series.hold_deadline(), None);
    assert_eq!(series.outcome(), None);
    assert_eq!(series.rematch_votes(), [false; 2]);
    assert!(series.ready_for_next(Tick(0)));
    assert_eq!(series, MatchSeries::default());
}

#[test]
fn result_hold_rejects_tick_179_and_accepts_tick_180() {
    let mut series = MatchSeries::new();
    let first = series.start_round(4, Tick(10)).unwrap();
    assert_eq!(first.round_number().get(), 1);
    assert_eq!(
        series.complete_round(
            4,
            RoundResult::Draw(DrawReason::MutualElimination),
            Tick(20)
        ),
        Ok(None)
    );
    assert_eq!(series.hold_deadline(), Some(Tick(200)));
    assert!(!series.ready_for_next(Tick(199)));

    let before = series.clone();
    assert_eq!(
        series.start_round(8, Tick(199)),
        Err(SeriesError::ResultHoldActive {
            ready_at: Tick(200)
        })
    );
    assert_eq!(series, before);
    assert!(series.ready_for_next(Tick(200)));

    let second = series.start_round(8, Tick(200)).unwrap();
    assert_eq!(second.round_number().get(), 2);
    assert_eq!(series.active_round(), Some(second));
    assert_eq!(
        series.latest_completed_round(),
        before.latest_completed_round()
    );
    assert_eq!(series.hold_deadline(), None);
}

#[test]
fn draws_advance_actual_rounds_without_awarding_points() {
    let mut series = MatchSeries::new();
    for (number, epoch, start) in [(1, 1, 0), (2, 17, 180), (3, 80, 360)] {
        let assignment = series.start_round(epoch, Tick(start)).unwrap();
        assert_eq!(assignment.round_number().get(), number);
        let expected_priority = if number % 2 == 1 {
            SeatId::One
        } else {
            SeatId::Two
        };
        assert_eq!(assignment.priority_seat(), expected_priority);
        series
            .complete_round(epoch, RoundResult::Draw(DrawReason::Timeout), Tick(start))
            .unwrap();
    }

    assert_eq!(series.scores(), [0, 0]);
    assert_eq!(series.actual_round_number(), 3);
    assert_eq!(series.outcome(), None);
    assert_eq!(series.last_command_epoch(), 80);
}

#[test]
fn third_win_finalizes_immediately_and_keeps_round_result_distinct() {
    for winner in SeatId::ALL {
        let series = score_finished_series(winner);
        let completed = series.latest_completed_round().unwrap();
        let outcome = series.outcome().unwrap();

        assert_eq!(series.scores()[winner.index()], 3);
        assert_eq!(completed.assignment().round_number().get(), 3);
        assert_eq!(completed.result(), RoundResult::Winner(winner));
        assert_eq!(outcome.ended_at(), completed.ended_at());
        assert_eq!(outcome.result(), MatchResult::ScoreWinner(winner));
        assert!(!series.ready_for_next(Tick(u64::MAX)));
    }
}

#[test]
fn epoch_gaps_do_not_consume_round_numbers_and_failures_are_atomic() {
    let mut series = MatchSeries::new();
    let first = series.start_round(7, Tick(5)).unwrap();
    assert_eq!(first.round_number().get(), 1);

    for (epoch, expected) in [
        (0, SeriesError::ZeroCommandEpoch),
        (
            6,
            SeriesError::StaleCommandEpoch {
                received: 6,
                last: 7,
            },
        ),
        (9, SeriesError::RoundAlreadyActive),
    ] {
        let before = series.clone();
        assert_eq!(series.start_round(epoch, Tick(5)), Err(expected));
        assert_eq!(series, before);
    }

    let before = series.clone();
    assert_eq!(
        series.complete_round(8, RoundResult::Winner(SeatId::One), Tick(6)),
        Err(SeriesError::WrongRoundEpoch {
            received: 8,
            active: 7,
        })
    );
    assert_eq!(series, before);
    series
        .complete_round(7, RoundResult::Winner(SeatId::One), Tick(6))
        .unwrap();
    let second = series.start_round(100, Tick(186)).unwrap();
    assert_eq!(second.round_number().get(), 2);
    assert_eq!(series.last_command_epoch(), 100);
}

#[test]
fn single_explicit_departure_forfeits_and_preserves_completed_facts() {
    for (departed, winner) in [(SeatId::One, SeatId::Two), (SeatId::Two, SeatId::One)] {
        let mut series = MatchSeries::new();
        finish_round(&mut series, 1, 10, 11, RoundResult::Winner(SeatId::One));
        let completed = series.latest_completed_round();
        let scores = series.scores();
        series.start_round(4, Tick(191)).unwrap();

        let mut departures = [None; 2];
        departures[departed.index()] = Some(DepartureReason::Explicit);
        let outcome = series.resolve_departures(departures, Tick(200)).unwrap();

        assert_eq!(
            outcome.unwrap().result(),
            MatchResult::ForfeitWinner(winner)
        );
        assert_eq!(series.latest_completed_round(), completed);
        assert_eq!(series.scores(), scores);
        assert_eq!(series.active_round(), None);
        assert_eq!(series.hold_deadline(), Some(Tick(380)));
    }
}

#[test]
fn any_transport_loss_interrupts_even_when_mixed_with_explicit_departure() {
    for departures in [
        [
            Some(DepartureReason::Explicit),
            Some(DepartureReason::TransportLoss),
        ],
        [
            Some(DepartureReason::TransportLoss),
            Some(DepartureReason::Explicit),
        ],
        [Some(DepartureReason::TransportLoss), None],
        [None, Some(DepartureReason::TransportLoss)],
    ] {
        let mut series = MatchSeries::new();
        series.start_round(1, Tick(10)).unwrap();
        let outcome = series.resolve_departures(departures, Tick(12)).unwrap();
        assert_eq!(
            outcome.unwrap().result(),
            MatchResult::Interrupted(InterruptReason::ConnectionLost)
        );
    }
}

#[test]
fn simultaneous_explicit_departures_abandon_without_a_winner() {
    let mut series = MatchSeries::new();
    series.start_round(1, Tick(10)).unwrap();

    let outcome = series
        .resolve_departures(
            [
                Some(DepartureReason::Explicit),
                Some(DepartureReason::Explicit),
            ],
            Tick(11),
        )
        .unwrap()
        .unwrap();

    assert_eq!(outcome.result(), MatchResult::Abandoned);
    assert_eq!(outcome.ended_at(), Tick(11));
}

#[test]
fn departure_before_first_actual_start_does_not_fabricate_a_result() {
    let mut series = MatchSeries::new();
    let before = series.clone();

    assert_eq!(
        series.resolve_departures([Some(DepartureReason::Explicit), None], Tick(u64::MAX),),
        Ok(None)
    );
    assert_eq!(series, before);
    assert_eq!(
        series.interrupt(InterruptReason::ServerStopped, Tick(u64::MAX)),
        Ok(None)
    );
    assert_eq!(series, before);
}

#[test]
fn finalized_match_wins_over_later_departures_and_interruptions() {
    let mut series = score_finished_series(SeatId::Two);
    let finalized = series.clone();

    assert_eq!(
        series.resolve_departures(
            [
                Some(DepartureReason::Explicit),
                Some(DepartureReason::TransportLoss)
            ],
            Tick(0),
        ),
        Ok(finalized.outcome())
    );
    assert_eq!(series, finalized);
    assert_eq!(
        series.interrupt(InterruptReason::InternalError, Tick(0)),
        Ok(finalized.outcome())
    );
    assert_eq!(series, finalized);
}

#[test]
fn final_hold_blocks_votes_at_179_and_allows_idempotence_and_withdrawal_at_180() {
    let mut series = score_finished_series(SeatId::One);
    let terminal_tick = series.outcome().unwrap().ended_at();
    let ready_at = Tick(terminal_tick.0 + RESULT_HOLD_TICKS);
    let last_epoch = series.last_command_epoch();

    let before = series.clone();
    assert_eq!(
        series.set_rematch_vote(SeatId::One, true, Tick(ready_at.0 - 1)),
        Err(SeriesError::ResultHoldActive { ready_at })
    );
    assert_eq!(series, before);

    assert_eq!(
        series.set_rematch_vote(SeatId::One, true, ready_at),
        Ok(RematchStatus::WaitingForOpponent)
    );
    assert_eq!(series.rematch_votes(), [true, false]);
    assert_eq!(series.last_command_epoch(), last_epoch);
    assert_eq!(
        series.set_rematch_vote(SeatId::One, true, Tick(ready_at.0 + 1)),
        Ok(RematchStatus::WaitingForOpponent)
    );
    assert_eq!(series.rematch_votes(), [true, false]);
    assert_eq!(
        series.set_rematch_vote(SeatId::One, false, Tick(ready_at.0 + 2)),
        Ok(RematchStatus::WaitingForOpponent)
    );
    assert_eq!(series.rematch_votes(), [false, false]);
}

#[test]
fn both_rematch_votes_reset_match_facts_but_retain_round_epoch_and_time() {
    let mut series = score_finished_series(SeatId::One);
    let terminal_tick = series.outcome().unwrap().ended_at();
    let ready_at = Tick(terminal_tick.0 + RESULT_HOLD_TICKS);
    let last_epoch = series.last_command_epoch();

    assert_eq!(
        series.set_rematch_vote(SeatId::Two, true, ready_at),
        Ok(RematchStatus::WaitingForOpponent)
    );
    assert_eq!(
        series.set_rematch_vote(SeatId::One, true, Tick(ready_at.0 + 1)),
        Ok(RematchStatus::Started)
    );

    assert_eq!(series.scores(), [0, 0]);
    assert_eq!(series.actual_round_number(), 0);
    assert_eq!(series.latest_completed_round(), None);
    assert_eq!(series.outcome(), None);
    assert_eq!(series.rematch_votes(), [false; 2]);
    assert_eq!(series.last_command_epoch(), last_epoch);
    assert!(!series.ready_for_next(ready_at));
    assert!(series.ready_for_next(Tick(ready_at.0 + 1)));
    let before = series.clone();
    assert_eq!(
        series.start_round(last_epoch + 1, ready_at),
        Err(SeriesError::NonMonotonicTick {
            received: ready_at,
            earliest: Tick(ready_at.0 + 1),
        })
    );
    assert_eq!(series, before);
    assert_eq!(
        series
            .start_round(last_epoch + 1, Tick(ready_at.0 + 1))
            .unwrap()
            .round_number()
            .get(),
        1
    );
}

#[test]
fn rematch_votes_require_a_terminal_match() {
    let mut series = MatchSeries::new();
    let before = series.clone();
    assert_eq!(
        series.set_rematch_vote(SeatId::One, true, Tick(0)),
        Err(SeriesError::RematchUnavailable)
    );
    assert_eq!(series, before);
}

#[test]
fn checked_arithmetic_and_time_failures_are_atomic() {
    let mut deadline = MatchSeries::new();
    deadline.start_round(1, Tick(4)).unwrap();
    let before = deadline.clone();
    assert_eq!(
        deadline.complete_round(
            1,
            RoundResult::Winner(SeatId::One),
            Tick(u64::MAX - RESULT_HOLD_TICKS + 1),
        ),
        Err(SeriesError::TickOverflow)
    );
    assert_eq!(deadline, before);

    let mut backwards = MatchSeries::new();
    backwards.start_round(3, Tick(10)).unwrap();
    let before = backwards.clone();
    assert_eq!(
        backwards.complete_round(3, RoundResult::Winner(SeatId::One), Tick(9)),
        Err(SeriesError::NonMonotonicTick {
            received: Tick(9),
            earliest: Tick(10),
        })
    );
    assert_eq!(backwards, before);

    let mut round_number = MatchSeries::new();
    round_number.actual_round_number = u32::MAX;
    let before = round_number.clone();
    assert_eq!(
        round_number.start_round(1, Tick(0)),
        Err(SeriesError::RoundNumberOverflow)
    );
    assert_eq!(round_number, before);
}

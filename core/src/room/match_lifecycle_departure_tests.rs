use super::*;
use crate::room::series::MatchResult;

fn occupied() -> MatchLifecycle {
    let mut lifecycle = MatchLifecycle::new();
    lifecycle.occupy(SeatId::One).unwrap();
    lifecycle.occupy(SeatId::Two).unwrap();
    lifecycle
}

fn advance(lifecycle: &mut MatchLifecycle, ticks: u64) {
    for _ in 0..ticks {
        lifecycle.advance_idle_tick().unwrap();
    }
}

fn start_initial(lifecycle: &mut MatchLifecycle) {
    lifecycle.set_ready(SeatId::One, true).unwrap();
    lifecycle.set_ready(SeatId::Two, true).unwrap();
    advance(lifecycle, COUNTDOWN_TICKS);
    assert_eq!(lifecycle.phase(), MatchPhase::Running);
}

fn start_next(lifecycle: &mut MatchLifecycle) {
    advance(lifecycle, COUNTDOWN_TICKS);
    assert!(matches!(lifecycle.phase(), MatchPhase::Countdown { .. }));
    advance(lifecycle, COUNTDOWN_TICKS);
    assert_eq!(lifecycle.phase(), MatchPhase::Running);
}

fn running_after_one_win() -> MatchLifecycle {
    let mut lifecycle = occupied();
    start_initial(&mut lifecycle);
    lifecycle
        .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    start_next(&mut lifecycle);
    lifecycle
}

fn score_finished() -> MatchLifecycle {
    let mut lifecycle = occupied();
    start_initial(&mut lifecycle);
    for win in 1..=3 {
        lifecycle
            .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
            .unwrap();
        if win < 3 {
            start_next(&mut lifecycle);
        }
    }
    lifecycle
}

#[test]
fn prestart_departures_cancel_once_without_a_forfeit_or_actual_round() {
    let mut waiting = occupied();
    waiting.set_ready(SeatId::Two, true).unwrap();
    assert_eq!(
        waiting.depart_batch([Some(DepartureReason::Explicit), None]),
        Ok(MatchChange::Changed)
    );
    assert_eq!(waiting.phase(), MatchPhase::Waiting);
    assert_eq!(waiting.round_attempt(), 1);
    assert_eq!(waiting.series().outcome(), None);
    assert_eq!(waiting.series().actual_round_number(), 0);
    assert!(waiting.seats().iter().all(|seat| !seat.ready()));

    let mut countdown = occupied();
    countdown.set_ready(SeatId::One, true).unwrap();
    countdown.set_ready(SeatId::Two, true).unwrap();
    assert_eq!(
        countdown.depart_batch([
            Some(DepartureReason::Explicit),
            Some(DepartureReason::Explicit)
        ]),
        Ok(MatchChange::Changed)
    );
    assert_eq!(countdown.phase(), MatchPhase::Waiting);
    assert_eq!(countdown.round_attempt(), 2);
    assert_eq!(countdown.series().actual_round_number(), 0);
    assert_eq!(countdown.series().outcome(), None);
    assert!(
        countdown
            .seats()
            .iter()
            .all(|seat| !seat.occupied() && !seat.ready())
    );
}

#[test]
fn started_departure_batches_apply_precedence_in_both_seat_orders() {
    let base = running_after_one_win();
    let completed = base.series().latest_completed_round();
    let scores = base.series().scores();
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
            MatchResult::Interrupted(InterruptReason::ConnectionLost),
        ),
        (
            [
                Some(DepartureReason::TransportLoss),
                Some(DepartureReason::Explicit),
            ],
            MatchResult::Interrupted(InterruptReason::ConnectionLost),
        ),
        (
            [Some(DepartureReason::TransportLoss), None],
            MatchResult::Interrupted(InterruptReason::ConnectionLost),
        ),
        (
            [None, Some(DepartureReason::TransportLoss)],
            MatchResult::Interrupted(InterruptReason::ConnectionLost),
        ),
    ];

    for (departures, expected) in cases {
        let mut lifecycle = base.clone();
        let MatchChange::MatchFinished(outcome) = lifecycle.depart_batch(departures).unwrap()
        else {
            panic!("started departure must finish the match");
        };
        assert_eq!(outcome.result(), expected);
        assert_eq!(lifecycle.series().latest_completed_round(), completed);
        assert_eq!(lifecycle.series().scores(), scores);
        assert_eq!(lifecycle.series().active_round(), None);
        for (index, departure) in departures.into_iter().enumerate() {
            assert_eq!(lifecycle.seats()[index].occupied(), departure.is_none());
        }
    }
}

#[test]
fn finalized_third_win_precedes_later_departure_batch() {
    let mut lifecycle = score_finished();
    let phase = lifecycle.phase();
    let outcome = lifecycle.series().outcome();
    let record = lifecycle.series().latest_completed_round();
    let scores = lifecycle.series().scores();

    assert_eq!(
        lifecycle.depart_batch([
            Some(DepartureReason::TransportLoss),
            Some(DepartureReason::Explicit)
        ]),
        Ok(MatchChange::Changed)
    );
    assert_eq!(lifecycle.phase(), phase);
    assert_eq!(lifecycle.series().outcome(), outcome);
    assert_eq!(lifecycle.series().latest_completed_round(), record);
    assert_eq!(lifecycle.series().scores(), scores);
    assert!(lifecycle.seats().iter().all(|seat| !seat.occupied()));
    assert_eq!(
        lifecycle.depart_batch([Some(DepartureReason::Explicit), None]),
        Ok(MatchChange::Unchanged)
    );
}

#[test]
fn finalized_departure_clears_only_that_seats_vote_and_cannot_start_rematch() {
    for (departed, remaining) in [(SeatId::One, SeatId::Two), (SeatId::Two, SeatId::One)] {
        let mut lifecycle = score_finished();
        advance(&mut lifecycle, COUNTDOWN_TICKS);
        let outcome = lifecycle.series().outcome();
        let terminal_phase = lifecycle.phase();
        let epoch = lifecycle.round_attempt();
        assert_eq!(
            lifecycle.set_rematch_vote(departed, true),
            Ok(MatchChange::Changed)
        );

        let mut departures = [None; 2];
        departures[departed.index()] = Some(DepartureReason::Explicit);
        assert_eq!(lifecycle.depart_batch(departures), Ok(MatchChange::Changed));
        assert!(!lifecycle.seats()[departed.index()].occupied());
        assert!(!lifecycle.series().rematch_votes()[departed.index()]);
        assert_eq!(lifecycle.series().outcome(), outcome);
        assert_eq!(lifecycle.round_attempt(), epoch);

        assert_eq!(
            lifecycle.set_rematch_vote(remaining, true),
            Ok(MatchChange::Changed)
        );
        assert_eq!(lifecycle.phase(), terminal_phase);
        assert_eq!(lifecycle.round_attempt(), epoch);
        assert_eq!(lifecycle.series().outcome(), outcome);
        assert!(lifecycle.series().rematch_votes()[remaining.index()]);
        assert!(!lifecycle.series().rematch_votes()[departed.index()]);
    }
}

#[test]
fn prematch_interruption_is_distinct_and_frozen() {
    for reason in [
        InterruptReason::ConnectionLost,
        InterruptReason::ServerOverloaded,
        InterruptReason::ServerStopped,
        InterruptReason::InternalError,
    ] {
        let mut lifecycle = occupied();
        lifecycle.advance_idle_tick().unwrap();
        assert_eq!(lifecycle.interrupt(reason), Ok(MatchChange::Changed));
        assert_eq!(
            lifecycle.phase(),
            MatchPhase::Interrupted {
                ended_at: Tick(1),
                reason
            }
        );
        assert_eq!(lifecycle.advance_idle_tick(), Ok(MatchChange::Unchanged));
        assert_eq!(lifecycle.tick(), Tick(1));
        assert_eq!(lifecycle.series().outcome(), None);
        assert_eq!(lifecycle.interrupt(reason), Ok(MatchChange::Unchanged));
    }
}

#[test]
fn started_administrative_interruption_becomes_a_match_result() {
    let mut lifecycle = occupied();
    start_initial(&mut lifecycle);
    let MatchChange::MatchFinished(outcome) = lifecycle
        .interrupt(InterruptReason::ServerOverloaded)
        .unwrap()
    else {
        panic!("started interruption must finish match");
    };

    assert_eq!(
        outcome.result(),
        MatchResult::Interrupted(InterruptReason::ServerOverloaded)
    );
    assert!(matches!(lifecycle.phase(), MatchPhase::MatchResult { .. }));
}

#[test]
fn participant_left_interrupt_is_rejected_atomically() {
    let mut lifecycle = occupied();
    let before = lifecycle.clone();
    assert_eq!(
        lifecycle.interrupt(InterruptReason::ParticipantLeft),
        Err(MatchError::InvalidInterruptReason)
    );
    assert_eq!(lifecycle, before);
}

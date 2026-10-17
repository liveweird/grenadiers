use super::*;

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
}

fn score_finished() -> MatchLifecycle {
    let mut lifecycle = occupied();
    start_initial(&mut lifecycle);
    for win in 1..=3 {
        lifecycle
            .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
            .unwrap();
        if win < 3 {
            advance(&mut lifecycle, COUNTDOWN_TICKS * 2);
        }
    }
    lifecycle
}

#[test]
fn waiting_tick_and_initial_deadline_overflows_are_atomic() {
    let mut ticking = MatchLifecycle::new();
    ticking.tick = Tick(u64::MAX);
    let before = ticking.clone();
    assert_eq!(ticking.advance_idle_tick(), Err(MatchError::TickOverflow));
    assert_eq!(ticking, before);

    let mut deadline = occupied();
    deadline.tick = Tick(u64::MAX - COUNTDOWN_TICKS + 1);
    deadline.set_ready(SeatId::One, true).unwrap();
    let before = deadline.clone();
    assert_eq!(
        deadline.set_ready(SeatId::Two, true),
        Err(MatchError::TickOverflow)
    );
    assert_eq!(deadline, before);
}

#[test]
fn cancellation_epoch_overflow_is_atomic() {
    let mut lifecycle = occupied();
    lifecycle.set_ready(SeatId::One, true).unwrap();
    lifecycle.set_ready(SeatId::Two, true).unwrap();
    lifecycle.round_attempt = u64::MAX;
    let before = lifecycle.clone();

    assert_eq!(
        lifecycle.set_ready(SeatId::One, false),
        Err(MatchError::RoundAttemptOverflow)
    );
    assert_eq!(lifecycle, before);
    assert_eq!(
        lifecycle.depart_batch([Some(DepartureReason::Explicit), None]),
        Err(MatchError::RoundAttemptOverflow)
    );
    assert_eq!(lifecycle, before);
}

#[test]
fn round_completion_and_interround_deadline_overflows_are_atomic() {
    let mut completion = occupied();
    start_initial(&mut completion);
    completion.tick = Tick(u64::MAX - COUNTDOWN_TICKS);
    let before = completion.clone();
    assert_eq!(
        completion.complete_running_tick(Some(RoundResult::Winner(SeatId::One))),
        Err(MatchError::Series(SeriesError::TickOverflow))
    );
    assert_eq!(completion, before);

    let mut countdown = MatchLifecycle::new();
    countdown.tick = Tick(u64::MAX - 101);
    countdown.phase = MatchPhase::Result {
        next_countdown_at: Tick(u64::MAX - 100),
    };
    let before = countdown.clone();
    assert_eq!(countdown.advance_idle_tick(), Err(MatchError::TickOverflow));
    assert_eq!(countdown, before);
}

#[test]
fn rematch_epoch_overflow_does_not_consume_the_second_vote() {
    let mut lifecycle = score_finished();
    advance(&mut lifecycle, COUNTDOWN_TICKS);
    lifecycle.round_attempt = u64::MAX;
    assert_eq!(
        lifecycle.set_rematch_vote(SeatId::One, true),
        Ok(MatchChange::Changed)
    );
    let before = lifecycle.clone();

    assert_eq!(
        lifecycle.set_rematch_vote(SeatId::Two, true),
        Err(MatchError::RoundAttemptOverflow)
    );
    assert_eq!(lifecycle, before);
    assert_eq!(lifecycle.series().rematch_votes(), [true, false]);
}

#[test]
fn match_result_clock_overflow_is_atomic() {
    let mut lifecycle = score_finished();
    lifecycle.tick = Tick(u64::MAX);
    let before = lifecycle.clone();

    assert_eq!(lifecycle.advance_idle_tick(), Err(MatchError::TickOverflow));
    assert_eq!(lifecycle, before);
}

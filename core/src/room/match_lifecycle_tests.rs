use super::*;
use crate::game::DrawReason;
use crate::room::series::MatchResult;

fn occupied() -> MatchLifecycle {
    let mut lifecycle = MatchLifecycle::new();
    lifecycle.occupy(SeatId::One).unwrap();
    lifecycle.occupy(SeatId::Two).unwrap();
    lifecycle
}

fn begin_countdown(lifecycle: &mut MatchLifecycle) {
    assert_eq!(
        lifecycle.set_ready(SeatId::One, true),
        Ok(MatchChange::Changed)
    );
    assert_eq!(
        lifecycle.set_ready(SeatId::Two, true),
        Ok(MatchChange::CountdownStarted)
    );
}

fn advance(lifecycle: &mut MatchLifecycle, ticks: u64) {
    for _ in 0..ticks {
        lifecycle.advance_idle_tick().unwrap();
    }
}

fn start_initial_round(lifecycle: &mut MatchLifecycle) -> RoundAssignment {
    begin_countdown(lifecycle);
    advance(lifecycle, COUNTDOWN_TICKS - 1);
    let MatchChange::RoundStarted(assignment) = lifecycle.advance_idle_tick().unwrap() else {
        panic!("countdown boundary must start a round");
    };
    assignment
}

fn start_next_round(lifecycle: &mut MatchLifecycle) -> RoundAssignment {
    let MatchPhase::Result { next_countdown_at } = lifecycle.phase() else {
        panic!("completed nonfinal round must show its result");
    };
    while lifecycle.tick().0 + 1 < next_countdown_at.0 {
        lifecycle.advance_idle_tick().unwrap();
    }
    assert_eq!(
        lifecycle.advance_idle_tick(),
        Ok(MatchChange::CountdownStarted)
    );
    let MatchPhase::Countdown {
        kind: CountdownKind::InterRound,
        ends_at,
    } = lifecycle.phase()
    else {
        panic!("result boundary must begin inter-round countdown");
    };
    while lifecycle.tick().0 + 1 < ends_at.0 {
        lifecycle.advance_idle_tick().unwrap();
    }
    let MatchChange::RoundStarted(assignment) = lifecycle.advance_idle_tick().unwrap() else {
        panic!("inter-round boundary must start a round");
    };
    assignment
}

#[test]
fn defaults_to_a_vacant_unstarted_match() {
    let lifecycle = MatchLifecycle::new();

    assert_eq!(lifecycle.tick(), Tick(0));
    assert_eq!(lifecycle.round_attempt(), 1);
    assert_eq!(lifecycle.phase(), MatchPhase::Waiting);
    assert_eq!(lifecycle.series(), &MatchSeries::new());
    assert!(
        lifecycle
            .seats()
            .iter()
            .all(|seat| !seat.occupied() && !seat.ready())
    );
    assert_eq!(lifecycle, MatchLifecycle::default());
}

#[test]
fn initial_countdown_starts_at_the_exact_180_tick_boundary() {
    let mut lifecycle = occupied();
    begin_countdown(&mut lifecycle);
    assert_eq!(
        lifecycle.phase(),
        MatchPhase::Countdown {
            kind: CountdownKind::Initial,
            ends_at: Tick(180)
        }
    );

    advance(&mut lifecycle, 179);
    assert_eq!(lifecycle.tick(), Tick(179));
    assert_eq!(lifecycle.series().actual_round_number(), 0);
    assert_eq!(
        lifecycle.advance_idle_tick(),
        Ok(MatchChange::RoundStarted(RoundAssignment::first()))
    );
    assert_eq!(lifecycle.tick(), Tick(180));
    assert_eq!(lifecycle.phase(), MatchPhase::Running);
    assert_eq!(lifecycle.series().actual_round_number(), 1);
    assert!(lifecycle.seats().iter().all(|seat| !seat.ready()));
}

#[test]
fn admission_and_readiness_are_rejected_outside_their_phases() {
    let mut lifecycle = occupied();
    start_initial_round(&mut lifecycle);
    let before = lifecycle.clone();
    assert_eq!(lifecycle.occupy(SeatId::One), Err(MatchError::InvalidPhase));
    assert_eq!(
        lifecycle.set_ready(SeatId::One, true),
        Err(MatchError::InvalidPhase)
    );
    assert_eq!(lifecycle, before);

    lifecycle
        .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    assert_eq!(
        lifecycle.set_ready(SeatId::One, true),
        Err(MatchError::InvalidPhase)
    );
    advance(&mut lifecycle, COUNTDOWN_TICKS);
    assert!(matches!(lifecycle.phase(), MatchPhase::Countdown { .. }));
    assert_eq!(
        lifecycle.set_ready(SeatId::One, true),
        Err(MatchError::InvalidPhase)
    );
}

#[test]
fn cancellation_consumes_epochs_without_consuming_actual_rounds_or_parity() {
    let mut lifecycle = occupied();
    for expected_epoch in [2, 3] {
        begin_countdown(&mut lifecycle);
        assert_eq!(
            lifecycle.set_ready(SeatId::One, false),
            Ok(MatchChange::Changed)
        );
        assert_eq!(lifecycle.phase(), MatchPhase::Waiting);
        assert_eq!(lifecycle.round_attempt(), expected_epoch);
        assert_eq!(lifecycle.series().actual_round_number(), 0);
        assert!(lifecycle.seats().iter().all(|seat| !seat.ready()));
    }

    let first = start_initial_round(&mut lifecycle);
    assert_eq!(first.round_number().get(), 1);
    assert_eq!(first.priority_seat(), SeatId::One);
    assert_eq!(lifecycle.series().last_command_epoch(), 3);
    lifecycle
        .complete_running_tick(Some(RoundResult::Draw(DrawReason::Timeout)))
        .unwrap();
    let second = start_next_round(&mut lifecycle);
    assert_eq!(second.round_number().get(), 2);
    assert_eq!(second.priority_seat(), SeatId::Two);
    assert_eq!(lifecycle.round_attempt(), 4);
}

#[test]
fn draw_and_three_wins_follow_both_179_180_holds_and_countdowns() {
    let mut lifecycle = occupied();
    let first = start_initial_round(&mut lifecycle);
    assert_eq!(first.round_number().get(), 1);
    assert_eq!(
        lifecycle.complete_running_tick(Some(RoundResult::Draw(DrawReason::Timeout))),
        Ok(MatchChange::RoundFinished)
    );
    assert_eq!(lifecycle.series().scores(), [0, 0]);

    for expected_round in 2..=4 {
        let result_deadline = lifecycle.series().hold_deadline().unwrap();
        advance(&mut lifecycle, 179);
        assert_eq!(lifecycle.tick(), Tick(result_deadline.0 - 1));
        assert!(matches!(lifecycle.phase(), MatchPhase::Result { .. }));
        assert_eq!(
            lifecycle.advance_idle_tick(),
            Ok(MatchChange::CountdownStarted)
        );
        let MatchPhase::Countdown { ends_at, .. } = lifecycle.phase() else {
            panic!("hold boundary must start countdown");
        };
        advance(&mut lifecycle, 179);
        assert_eq!(lifecycle.tick(), Tick(ends_at.0 - 1));
        let MatchChange::RoundStarted(assignment) = lifecycle.advance_idle_tick().unwrap() else {
            panic!("countdown boundary must start round");
        };
        assert_eq!(assignment.round_number().get(), expected_round);

        let change = lifecycle
            .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
            .unwrap();
        if expected_round < 4 {
            assert_eq!(change, MatchChange::RoundFinished);
        } else {
            let MatchChange::MatchFinished(outcome) = change else {
                panic!("third win must finish match immediately");
            };
            assert_eq!(outcome.result(), MatchResult::ScoreWinner(SeatId::One));
            assert!(matches!(lifecycle.phase(), MatchPhase::MatchResult { .. }));
        }
    }

    assert_eq!(lifecycle.series().scores(), [3, 0]);
    assert_eq!(lifecycle.series().actual_round_number(), 4);
}

#[test]
fn caller_supplied_timeout_follows_exactly_10800_running_updates() {
    let mut lifecycle = occupied();
    start_initial_round(&mut lifecycle);
    for _ in 0..10_799 {
        assert_eq!(
            lifecycle.complete_running_tick(None),
            Ok(MatchChange::Changed)
        );
    }
    assert_eq!(lifecycle.tick(), Tick(10_979));
    assert_eq!(lifecycle.phase(), MatchPhase::Running);

    assert_eq!(
        lifecycle.complete_running_tick(Some(RoundResult::Draw(DrawReason::Timeout))),
        Ok(MatchChange::RoundFinished)
    );
    assert_eq!(lifecycle.tick(), Tick(10_980));
    assert_eq!(
        lifecycle
            .series()
            .latest_completed_round()
            .unwrap()
            .result(),
        RoundResult::Draw(DrawReason::Timeout)
    );
}

#[test]
fn running_match_has_no_creation_age_deadline() {
    let mut lifecycle = occupied();
    start_initial_round(&mut lifecycle);
    for _ in 0..18_001 {
        lifecycle.complete_running_tick(None).unwrap();
    }

    assert!(lifecycle.tick().0 > 300 * 60);
    assert_eq!(lifecycle.phase(), MatchPhase::Running);
}

#[test]
fn rematch_votes_are_idempotent_withdrawable_and_advance_epoch_once() {
    let mut lifecycle = occupied();
    start_initial_round(&mut lifecycle);
    for _ in 0..3 {
        lifecycle
            .complete_running_tick(Some(RoundResult::Winner(SeatId::Two)))
            .unwrap();
        if lifecycle.series().outcome().is_none() {
            start_next_round(&mut lifecycle);
        }
    }
    let final_epoch = lifecycle.round_attempt();
    let final_series_epoch = lifecycle.series().last_command_epoch();
    let MatchPhase::MatchResult { rematch_at, .. } = lifecycle.phase() else {
        panic!("third win must be terminal");
    };
    advance(&mut lifecycle, 179);
    let before = lifecycle.clone();
    assert_eq!(
        lifecycle.set_rematch_vote(SeatId::One, true),
        Err(MatchError::Series(SeriesError::ResultHoldActive {
            ready_at: rematch_at
        }))
    );
    assert_eq!(lifecycle, before);
    lifecycle.advance_idle_tick().unwrap();

    assert_eq!(
        lifecycle.set_rematch_vote(SeatId::One, true),
        Ok(MatchChange::Changed)
    );
    assert_eq!(
        lifecycle.set_rematch_vote(SeatId::One, true),
        Ok(MatchChange::Unchanged)
    );
    assert_eq!(
        lifecycle.set_rematch_vote(SeatId::One, false),
        Ok(MatchChange::Changed)
    );
    lifecycle.set_rematch_vote(SeatId::One, true).unwrap();
    assert_eq!(lifecycle.round_attempt(), final_epoch);
    assert_eq!(
        lifecycle.set_rematch_vote(SeatId::Two, true),
        Ok(MatchChange::RematchStarted)
    );

    assert_eq!(lifecycle.phase(), MatchPhase::Waiting);
    assert_eq!(lifecycle.round_attempt(), final_epoch + 1);
    assert_eq!(lifecycle.series().last_command_epoch(), final_series_epoch);
    assert_eq!(lifecycle.series().actual_round_number(), 0);
    assert_eq!(lifecycle.series().scores(), [0, 0]);
    assert_eq!(lifecycle.series().latest_completed_round(), None);
    assert!(
        lifecycle
            .seats()
            .iter()
            .all(|seat| seat.occupied() && !seat.ready())
    );
}

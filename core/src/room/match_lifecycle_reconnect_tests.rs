use super::*;
use crate::room::reconnect::ReservationState;
use crate::room::series::MatchResult;

fn occupied() -> MatchLifecycle {
    let mut lifecycle = MatchLifecycle::new();
    lifecycle.occupy(SeatId::One).unwrap();
    lifecycle.occupy(SeatId::Two).unwrap();
    assert_invariants(&lifecycle);
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
    assert_eq!(lifecycle.phase(), MatchPhase::Running);
}

fn result_after_one_win() -> MatchLifecycle {
    let mut lifecycle = occupied();
    start(&mut lifecycle);
    lifecycle
        .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    lifecycle
}

fn assert_invariants(lifecycle: &MatchLifecycle) {
    for seat in SeatId::DUEL {
        let index = seat.index();
        assert_eq!(
            lifecycle.seats()[index].occupied(),
            lifecycle
                .reconnect_grace()
                .seat(seat)
                .is_some_and(ReservationState::reserved)
        );
        if lifecycle.seats()[index].ready() {
            assert!(
                lifecycle
                    .reconnect_grace()
                    .seat(seat)
                    .is_some_and(ReservationState::connected)
            );
        }
    }
    assert!(lifecycle.reconnect_grace().latest_tick() <= lifecycle.tick());
}

#[test]
fn prematch_losses_release_and_cancel_before_the_start_boundary() {
    for reason in [DepartureReason::Explicit, DepartureReason::TransportLoss] {
        let mut lifecycle = occupied();
        lifecycle.set_ready(SeatId::One, true).unwrap();
        lifecycle.set_ready(SeatId::Two, true).unwrap();
        advance_idle(&mut lifecycle, COUNTDOWN_TICKS - 1);
        assert_eq!(
            lifecycle.depart_with_grace([Some(reason), None]),
            Ok(MatchChange::Changed)
        );
        assert_eq!(lifecycle.phase(), MatchPhase::Waiting);
        assert_eq!(lifecycle.round_attempt(), 2);
        assert_eq!(lifecycle.series().actual_round_number(), 0);
        assert_eq!(
            lifecycle.reconnect_grace().seat(SeatId::One),
            Some(ReservationState::Vacant)
        );
        assert_eq!(lifecycle.advance_idle_tick(), Ok(MatchChange::Changed));
        assert_eq!(lifecycle.phase(), MatchPhase::Waiting);
        assert_invariants(&lifecycle);
    }

    let mut waiting = occupied();
    waiting.set_ready(SeatId::One, true).unwrap();
    waiting
        .depart_with_grace([None, Some(DepartureReason::TransportLoss)])
        .unwrap();
    assert!(waiting.seats().iter().all(|seat| !seat.ready()));
    assert_eq!(
        waiting.resume_connection(SeatId::Two),
        Err(MatchError::InvalidPhase)
    );
}

#[test]
fn held_result_waits_for_connection_then_starts_a_fresh_full_countdown() {
    let mut lifecycle = result_after_one_win();
    let completed = lifecycle.series().latest_completed_round();
    let original_hold = lifecycle.series().hold_deadline().unwrap();
    let loss_at = lifecycle.tick();
    lifecycle
        .depart_with_grace([None, Some(DepartureReason::TransportLoss)])
        .unwrap();
    assert_eq!(
        lifecycle.reconnect_grace().seat(SeatId::Two),
        Some(ReservationState::Disconnected {
            expires_at: Tick(loss_at.0 + 600)
        })
    );
    assert_eq!(
        lifecycle.depart_with_grace([None, Some(DepartureReason::TransportLoss)]),
        Ok(MatchChange::Unchanged)
    );
    assert_eq!(
        lifecycle.reconnect_grace().seat(SeatId::Two),
        Some(ReservationState::Disconnected {
            expires_at: Tick(loss_at.0 + 600)
        })
    );
    let mut early_return = lifecycle.clone();
    advance_idle(&mut early_return, 100);
    early_return.resume_connection(SeatId::Two).unwrap();
    advance_idle(&mut early_return, 79);
    assert_eq!(
        early_return.phase(),
        MatchPhase::Result {
            next_countdown_at: original_hold
        }
    );
    assert_eq!(
        early_return.advance_idle_tick(),
        Ok(MatchChange::CountdownStarted)
    );
    advance_idle(&mut lifecycle, 599);
    assert_eq!(
        lifecycle.phase(),
        MatchPhase::Result {
            next_countdown_at: original_hold
        }
    );
    assert_eq!(
        lifecycle.resume_connection(SeatId::Two),
        Ok(MatchChange::Changed)
    );
    let transition_tick = Tick(lifecycle.tick().0 + 1);
    assert_eq!(
        lifecycle.advance_idle_tick(),
        Ok(MatchChange::CountdownStarted)
    );
    assert_eq!(
        lifecycle.phase(),
        MatchPhase::Countdown {
            kind: CountdownKind::InterRound,
            ends_at: Tick(transition_tick.0 + COUNTDOWN_TICKS),
        }
    );
    assert_eq!(lifecycle.series().latest_completed_round(), completed);
    assert_invariants(&lifecycle);
}

#[test]
fn single_expiry_forfeits_to_the_connected_opponent_in_both_orientations() {
    for expired in SeatId::DUEL {
        let winner = SeatId::DUEL[1 - expired.index()];
        let mut lifecycle = occupied();
        start(&mut lifecycle);
        let mut loss = [None; 2];
        loss[expired.index()] = Some(DepartureReason::TransportLoss);
        lifecycle.depart_with_grace(loss).unwrap();
        advance_running(&mut lifecycle, 600);
        let MatchChange::MatchFinished(outcome) = lifecycle.expire_reconnect_grace().unwrap()
        else {
            panic!("connected opponent wins after grace");
        };
        assert_eq!(outcome.result(), MatchResult::ForfeitWinner(winner));
        assert_eq!(
            lifecycle.reconnect_grace().seat(expired),
            Some(ReservationState::Expired)
        );
    }
}

#[test]
fn exact_grace_boundary_rejects_resume_atomically() {
    let mut lifecycle = result_after_one_win();
    lifecycle
        .depart_with_grace([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    advance_idle(&mut lifecycle, 599);
    assert_eq!(
        lifecycle.resume_connection(SeatId::One),
        Ok(MatchChange::Changed)
    );

    let mut boundary = result_after_one_win();
    boundary
        .depart_with_grace([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    advance_idle(&mut boundary, 600);
    let before = boundary.clone();
    assert!(matches!(
        boundary.resume_connection(SeatId::One),
        Err(MatchError::Reconnect(ReconnectError::GraceExpired { .. }))
    ));
    assert_eq!(boundary, before);
    let MatchChange::MatchFinished(outcome) = boundary.expire_reconnect_grace().unwrap() else {
        panic!("connected opponent wins at expiry");
    };
    assert_eq!(outcome.result(), MatchResult::ForfeitWinner(SeatId::Two));
}

#[test]
fn interround_loss_at_final_boundary_cancels_once_without_an_actual_start() {
    for losses in [
        [Some(DepartureReason::TransportLoss), None],
        [
            Some(DepartureReason::TransportLoss),
            Some(DepartureReason::TransportLoss),
        ],
    ] {
        let mut lifecycle = result_after_one_win();
        let completed = lifecycle.series().latest_completed_round();
        advance_idle(&mut lifecycle, COUNTDOWN_TICKS);
        advance_idle(&mut lifecycle, COUNTDOWN_TICKS - 1);
        let epoch = lifecycle.round_attempt();
        assert_eq!(
            lifecycle.depart_with_grace(losses),
            Ok(MatchChange::Changed)
        );
        assert!(matches!(lifecycle.phase(), MatchPhase::Result { .. }));
        assert_eq!(lifecycle.round_attempt(), epoch + 1);
        assert_eq!(lifecycle.series().actual_round_number(), 1);
        assert_eq!(lifecycle.series().latest_completed_round(), completed);
        assert_eq!(lifecycle.advance_idle_tick(), Ok(MatchChange::Changed));
        assert!(matches!(lifecycle.phase(), MatchPhase::Result { .. }));
        assert_eq!(
            lifecycle.depart_with_grace(losses),
            Ok(MatchChange::Unchanged)
        );
        assert_eq!(lifecycle.round_attempt(), epoch + 1);
    }
}

#[test]
fn staggered_expiry_waits_then_abandons_and_return_can_win() {
    let mut lifecycle = occupied();
    start(&mut lifecycle);
    advance_running(&mut lifecycle, 10);
    lifecycle
        .depart_with_grace([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    advance_running(&mut lifecycle, 10);
    lifecycle
        .depart_with_grace([None, Some(DepartureReason::TransportLoss)])
        .unwrap();
    advance_running(&mut lifecycle, 590);
    assert_eq!(lifecycle.expire_reconnect_grace(), Ok(MatchChange::Changed));
    assert_eq!(lifecycle.series().outcome(), None);
    assert_eq!(
        lifecycle.reconnect_grace().seats(),
        &[
            ReservationState::Expired,
            ReservationState::Disconnected {
                expires_at: Tick(800)
            }
        ]
    );
    let mut returning = lifecycle.clone();
    let MatchChange::MatchFinished(outcome) = returning.resume_connection(SeatId::Two).unwrap()
    else {
        panic!("eligible return against expired opponent wins");
    };
    assert_eq!(outcome.result(), MatchResult::ForfeitWinner(SeatId::Two));

    advance_running(&mut lifecycle, 10);
    let MatchChange::MatchFinished(outcome) = lifecycle.expire_reconnect_grace().unwrap() else {
        panic!("both expired must abandon");
    };
    assert_eq!(outcome.result(), MatchResult::Abandoned);
    assert_eq!(
        lifecycle.reconnect_grace().seats(),
        &[ReservationState::Expired; 2]
    );
    assert!(matches!(
        lifecycle.resume_connection(SeatId::One),
        Err(MatchError::Reconnect(ReconnectError::SeatExpired(
            SeatId::One
        )))
    ));
    assert!(matches!(
        lifecycle.occupy(SeatId::One),
        Err(MatchError::InvalidPhase)
    ));
}

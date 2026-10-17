use crate::game::{RoundResult, SeatId};
use crate::room::{COUNTDOWN_TICKS, DepartureReason, MatchPhase};

use super::projection_test_support::{
    advance_idle, advance_running, occupied, project, result_after_wins, running_with_wins, start,
};
use super::{ConnectionStatus, MatchResult, MatchWinnerReason, RoomPhase};

#[test]
fn held_and_canceled_results_project_old_completed_epoch_and_past_hold() {
    let mut held = result_after_wins(1);
    let completed = held.series().latest_completed_round().unwrap();
    held.depart_with_grace([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    advance_idle(&mut held, COUNTDOWN_TICKS + 5);
    let held_view = project(&held);
    assert_eq!(
        held_view.last_round.unwrap().round_id.get(),
        completed.epoch() as u32
    );
    let RoomPhase::Result { hold_until } = held_view.phase else {
        panic!("disconnected participant holds the result");
    };
    assert!(held_view.tick > hold_until);
    assert!(matches!(
        held_view.seats[0].connection,
        ConnectionStatus::Disconnected { .. }
    ));

    held.resume_connection(SeatId::One).unwrap();
    held.advance_idle_tick().unwrap();
    assert!(matches!(project(&held).phase, RoomPhase::Countdown { .. }));

    let mut canceled = result_after_wins(1);
    advance_idle(&mut canceled, COUNTDOWN_TICKS * 2 - 1);
    let countdown_epoch = canceled.round_attempt();
    canceled
        .depart_with_grace([None, Some(DepartureReason::TransportLoss)])
        .unwrap();
    let canceled_view = project(&canceled);
    assert!(matches!(canceled_view.phase, RoomPhase::Result { .. }));
    assert_eq!(canceled_view.round_id.get(), (countdown_epoch + 1) as u32);
    assert!(canceled_view.last_round.unwrap().round_id < canceled_view.round_id);
}

#[test]
fn second_and_third_win_expiry_preserve_real_score_priority() {
    let mut second = running_with_wins(1);
    second
        .depart_with_grace([None, Some(DepartureReason::TransportLoss)])
        .unwrap();
    advance_running(&mut second, 599);
    second
        .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    second.expire_reconnect_grace().unwrap();
    let second_view = project(&second);
    assert_eq!(second_view.scores.one.get(), 2);
    assert_eq!(second_view.last_round.unwrap().round_number.get(), 2);
    assert_eq!(second_view.seats[1].connection, ConnectionStatus::Expired);
    assert!(matches!(
        second_view.phase,
        RoomPhase::MatchResult {
            result: MatchResult::Winner {
                reason: MatchWinnerReason::Forfeit,
                ..
            },
            ..
        }
    ));

    let mut third = running_with_wins(2);
    third
        .depart_with_grace([None, Some(DepartureReason::TransportLoss)])
        .unwrap();
    advance_running(&mut third, 599);
    third
        .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    third.expire_reconnect_grace().unwrap();
    let third_view = project(&third);
    assert_eq!(third_view.scores.one.get(), 3);
    assert!(matches!(
        third_view.phase,
        RoomPhase::MatchResult {
            result: MatchResult::Winner {
                reason: MatchWinnerReason::RoundWins,
                ..
            },
            ..
        }
    ));
}

#[test]
fn mixed_and_postfinal_departures_keep_historical_outcome() {
    let mut lifecycle = occupied();
    start(&mut lifecycle);
    lifecycle
        .depart_with_grace([
            Some(DepartureReason::Explicit),
            Some(DepartureReason::TransportLoss),
        ])
        .unwrap();
    let outcome = lifecycle.series().outcome();
    let mixed = project(&lifecycle);
    assert_eq!(mixed.seats[0].connection, ConnectionStatus::Vacant);
    assert!(matches!(
        mixed.seats[1].connection,
        ConnectionStatus::Disconnected { .. }
    ));

    lifecycle.resume_connection(SeatId::Two).unwrap();
    lifecycle
        .depart_with_grace([None, Some(DepartureReason::TransportLoss)])
        .unwrap();
    assert_eq!(lifecycle.series().outcome(), outcome);
    assert_eq!(
        project(&lifecycle).seats[1].connection,
        ConnectionStatus::Vacant
    );
}

#[test]
fn terminal_resume_expiry_and_rematch_project_settled_facts() {
    let mut terminal = running_with_wins(2);
    terminal
        .depart_with_grace([None, Some(DepartureReason::TransportLoss)])
        .unwrap();
    terminal
        .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    terminal.resume_connection(SeatId::Two).unwrap();
    assert_eq!(
        project(&terminal).seats[1].connection,
        ConnectionStatus::Connected
    );
    advance_idle(&mut terminal, COUNTDOWN_TICKS);
    terminal.set_rematch_vote(SeatId::One, true).unwrap();
    assert!(project(&terminal).seats[0].rematch_ready);
    terminal.set_rematch_vote(SeatId::Two, true).unwrap();
    let reset = project(&terminal);
    assert_eq!(reset.phase, RoomPhase::Waiting);
    assert_eq!(reset.round_number.get(), 0);
    assert!(reset.last_round.is_none());
    assert!(matches!(terminal.phase(), MatchPhase::Waiting));
}

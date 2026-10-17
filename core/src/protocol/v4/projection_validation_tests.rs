use crate::game::{RoundResult, SeatId};
use crate::protocol::{Counter, Seat, Tick};
use crate::room::{COUNTDOWN_TICKS, DepartureReason};

use super::projection_test_support::{
    advance_idle, advance_running, occupied, project, result_after_wins, running_with_wins, start,
};
use super::{
    ConnectionStatus, MatchResult, MatchWinnerReason, RoomPhase, Wins, validate_room_view,
};

fn rejected(mutator: impl FnOnce(&mut super::RoomView), mut room: super::RoomView) {
    mutator(&mut room);
    assert!(validate_room_view(&room).is_err(), "mutation was accepted");
}

#[test]
fn rejects_occupancy_deadline_and_connection_flag_mismatches() {
    let mut lifecycle = result_after_wins(1);
    lifecycle
        .depart_with_grace([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    let base = project(&lifecycle);
    assert!(validate_room_view(&base).is_ok());

    rejected(|room| room.seats[0].occupied = false, base.clone());
    rejected(|room| room.seats[0].ready = true, base.clone());
    rejected(
        |room| {
            room.seats[0].connection = ConnectionStatus::Disconnected {
                expires_at: room.tick,
            };
        },
        base.clone(),
    );
    rejected(
        |room| {
            room.seats[0].connection = ConnectionStatus::Disconnected {
                expires_at: Tick::new(room.tick.get() + 601).unwrap(),
            };
        },
        base,
    );
}

#[test]
fn countdowns_require_connected_seats_and_elapsed_history() {
    let mut initial = occupied();
    initial.set_ready(SeatId::One, true).unwrap();
    initial.set_ready(SeatId::Two, true).unwrap();
    let initial_view = project(&initial);
    rejected(
        |room| {
            room.seats[0].connection = ConnectionStatus::Disconnected {
                expires_at: Tick::new(room.tick.get() + 600).unwrap(),
            };
        },
        initial_view,
    );

    let mut inter = result_after_wins(1);
    advance_idle(&mut inter, COUNTDOWN_TICKS);
    let inter_view = project(&inter);
    rejected(
        |room| room.last_round.as_mut().unwrap().round_id = room.round_id,
        inter_view.clone(),
    );
    rejected(
        |room| room.tick = Tick::new(room.last_round.unwrap().ended_at.get() + 179).unwrap(),
        inter_view,
    );
}

#[test]
fn nonfinal_expiry_waits_only_for_a_disconnected_reservation() {
    let mut lifecycle = occupied();
    start(&mut lifecycle);
    lifecycle
        .depart_with_grace([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    advance_running(&mut lifecycle, 10);
    lifecycle
        .depart_with_grace([None, Some(DepartureReason::TransportLoss)])
        .unwrap();
    advance_running(&mut lifecycle, 590);
    lifecycle.expire_reconnect_grace().unwrap();
    let base = project(&lifecycle);
    assert_eq!(base.seats[0].connection, ConnectionStatus::Expired);
    assert!(matches!(
        base.seats[1].connection,
        ConnectionStatus::Disconnected { .. }
    ));
    rejected(
        |room| room.seats[1].connection = ConnectionStatus::Connected,
        base.clone(),
    );
    rejected(
        |room| room.seats[1].connection = ConnectionStatus::Expired,
        base,
    );
}

#[test]
fn final_outcomes_reject_forged_winners_losers_and_history() {
    let mut forfeit = running_with_wins(1);
    forfeit
        .depart_with_grace([None, Some(DepartureReason::Explicit)])
        .unwrap();
    let forfeit_view = project(&forfeit);
    rejected(
        |room| room.seats[1].connection = ConnectionStatus::Connected,
        forfeit_view.clone(),
    );
    rejected(
        |room| {
            let RoomPhase::MatchResult { result, .. } = &mut room.phase else {
                unreachable!()
            };
            *result = MatchResult::Winner {
                seat: Seat::Two,
                reason: MatchWinnerReason::Forfeit,
            };
        },
        forfeit_view,
    );

    let mut score = running_with_wins(2);
    score
        .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    let score_view = project(&score);
    rejected(
        |room| room.scores.one = Wins::new(2).unwrap(),
        score_view.clone(),
    );
    rejected(
        |room| {
            room.last_round.as_mut().unwrap().round_id =
                Counter::new(room.round_id.get() - 1).unwrap()
        },
        score_view,
    );
}

#[test]
fn terminal_votes_and_disconnected_deadlines_use_authoritative_time() {
    let mut terminal = running_with_wins(2);
    terminal
        .depart_with_grace([None, Some(DepartureReason::TransportLoss)])
        .unwrap();
    terminal
        .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    let base = project(&terminal);
    rejected(|room| room.seats[0].rematch_ready = true, base.clone());
    rejected(
        |room| {
            room.seats[1].connection = ConnectionStatus::Disconnected {
                expires_at: Tick::new(match room.phase {
                    RoomPhase::MatchResult { ended_at, .. } => ended_at.get() + 601,
                    _ => unreachable!(),
                })
                .unwrap(),
            };
        },
        base,
    );
}

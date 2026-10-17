use crate::game::{RoundResult, SeatId};
use crate::protocol::{Counter, Seat, Tick};

use super::super::{RoomPhase, RoundCount, Wins};
use super::test_support::{
    first_running_view, initial_countdown_view, inter_countdown_view, later_running_view, occupied,
    project, result_view, score_terminal_lifecycle, start_initial, start_next, waiting_view,
};
use super::{RoomProjectionError, project_room_view, validate_room_view};

fn assert_error(room: super::super::RoomView, expected: RoomProjectionError) {
    assert_eq!(validate_room_view(&room), Err(expected));
}

#[test]
fn rejects_noncanonical_or_phase_incompatible_seat_flags() {
    let mut room = waiting_view();
    room.seats[0].seat = Seat::Two;
    assert_error(room, RoomProjectionError::InvalidSeats);

    let mut room = waiting_view();
    room.seats[0].occupied = false;
    room.seats[0].ready = true;
    assert_error(room, RoomProjectionError::InvalidSeats);

    let mut room = first_running_view();
    room.seats[0].ready = true;
    assert_error(room, RoomProjectionError::InvalidSeats);

    let mut room = first_running_view();
    room.seats[0].rematch_ready = true;
    assert_error(room, RoomProjectionError::InvalidSeats);

    let lifecycle = score_terminal_lifecycle();
    let mut room = project_room_view(super::test_support::metadata(), &lifecycle).unwrap();
    room.seats[0].occupied = false;
    room.seats[0].rematch_ready = true;
    assert_error(room, RoomProjectionError::InvalidSeats);
}

#[test]
fn waiting_requires_fresh_match_facts() {
    let mut room = waiting_view();
    room.round_number = RoundCount::new(1).unwrap();
    assert_error(room, RoomProjectionError::InvalidPhase);

    let mut room = waiting_view();
    room.scores.one = Wins::new(1).unwrap();
    assert_error(room, RoomProjectionError::InvalidScores);

    let mut room = waiting_view();
    room.last_round = result_view().last_round;
    assert_error(room, RoomProjectionError::InvalidCompletedRound);

    let mut lifecycle = occupied();
    lifecycle.set_ready(SeatId::One, true).unwrap();
    let mut room = project(&lifecycle);
    assert!(validate_room_view(&room).is_ok());
    room.seats[1].ready = true;
    assert_error(room, RoomProjectionError::InvalidPhase);
}

#[test]
fn initial_countdown_requires_two_ready_players_and_a_near_future_deadline() {
    let base = initial_countdown_view();
    let now = base.tick.get();

    let mut room = base.clone();
    room.seats[0].ready = false;
    assert_error(room, RoomProjectionError::InvalidPhase);

    let mut room = base.clone();
    room.scores.one = Wins::new(1).unwrap();
    assert_error(room, RoomProjectionError::InvalidScores);

    let mut room = base.clone();
    room.seats[1].occupied = false;
    room.seats[1].ready = false;
    assert_error(room, RoomProjectionError::InvalidPhase);

    let mut room = base.clone();
    room.phase = RoomPhase::Countdown {
        kind: super::super::CountdownKind::Initial,
        ends_at: Tick::new(now).unwrap(),
    };
    assert_error(room, RoomProjectionError::InvalidPhase);

    let mut room = base;
    room.phase = RoomPhase::Countdown {
        kind: super::super::CountdownKind::Initial,
        ends_at: Tick::new(now + 181).unwrap(),
    };
    assert_error(room, RoomProjectionError::InvalidPhase);
}

#[test]
fn interround_countdown_requires_current_record_older_epoch_and_bounded_time() {
    let base = inter_countdown_view();
    let now = base.tick.get();

    let mut room = base.clone();
    room.last_round.as_mut().unwrap().round_number = Counter::new(2).unwrap();
    assert_error(room, RoomProjectionError::InvalidCompletedRound);

    let mut room = base.clone();
    room.last_round.as_mut().unwrap().round_id = room.round_id;
    assert_error(room, RoomProjectionError::InvalidPhase);

    let mut room = base.clone();
    room.seats[0].ready = true;
    assert_error(room, RoomProjectionError::InvalidSeats);

    let mut room = base.clone();
    room.seats[0].occupied = false;
    assert_error(room, RoomProjectionError::InvalidPhase);

    let mut room = base.clone();
    room.scores.one = Wins::new(3).unwrap();
    assert_error(room, RoomProjectionError::InvalidCompletedRound);

    let mut room = base;
    room.phase = RoomPhase::Countdown {
        kind: super::super::CountdownKind::InterRound,
        ends_at: Tick::new(now + 181).unwrap(),
    };
    assert_error(room, RoomProjectionError::InvalidPhase);
}

#[test]
fn running_requires_complete_prior_record_or_fresh_first_round() {
    let mut room = first_running_view();
    room.round_number = RoundCount::new(0).unwrap();
    assert_error(room, RoomProjectionError::InvalidPhase);

    let mut room = first_running_view();
    room.last_round = result_view().last_round;
    assert_error(room, RoomProjectionError::InvalidCompletedRound);

    let mut room = first_running_view();
    room.scores.one = Wins::new(3).unwrap();
    assert_error(room, RoomProjectionError::InvalidScores);

    let mut room = later_running_view();
    room.last_round = None;
    assert_error(room, RoomProjectionError::InvalidScores);

    let mut room = later_running_view();
    room.last_round.as_mut().unwrap().round_number = Counter::new(2).unwrap();
    assert_error(room, RoomProjectionError::InvalidCompletedRound);

    let mut room = later_running_view();
    room.last_round.as_mut().unwrap().round_id = room.round_id;
    assert_error(room, RoomProjectionError::InvalidCompletedRound);

    let mut room = later_running_view();
    room.seats[1].occupied = false;
    assert_error(room, RoomProjectionError::InvalidPhase);
}

#[test]
fn result_requires_current_record_exact_deadline_and_nonterminal_scores() {
    let base = result_view();
    let last = base.last_round.unwrap();

    let mut room = base.clone();
    room.last_round = None;
    assert_error(room, RoomProjectionError::InvalidScores);

    let mut room = base.clone();
    room.last_round.as_mut().unwrap().round_id = Counter::new(room.round_id.get() + 1).unwrap();
    assert_error(room, RoomProjectionError::InvalidCompletedRound);

    let mut room = base.clone();
    room.last_round.as_mut().unwrap().round_number = Counter::new(2).unwrap();
    assert_error(room, RoomProjectionError::InvalidCompletedRound);

    let mut room = base.clone();
    room.phase = RoomPhase::Result {
        next_countdown_at: Tick::new(last.ended_at.get() + 179).unwrap(),
    };
    assert_error(room, RoomProjectionError::InvalidPhase);

    let mut room = base.clone();
    room.tick = Tick::new(last.ended_at.get() + 180).unwrap();
    assert_error(room, RoomProjectionError::InvalidPhase);

    let mut room = base;
    room.scores.one = Wins::new(3).unwrap();
    assert_error(room, RoomProjectionError::InvalidCompletedRound);
}

#[test]
fn global_completed_round_bounds_reject_future_gap_and_impossible_score_history() {
    let base = result_view();

    let mut room = base.clone();
    room.last_round.as_mut().unwrap().ended_at = Tick::new(room.tick.get() + 1).unwrap();
    assert_error(room, RoomProjectionError::InvalidCompletedRound);

    let mut room = base.clone();
    room.last_round.as_mut().unwrap().round_number = Counter::new(2).unwrap();
    assert_error(room, RoomProjectionError::InvalidCompletedRound);

    let mut room = base.clone();
    room.last_round.as_mut().unwrap().round_id = Counter::new(room.round_id.get() + 1).unwrap();
    assert_error(room, RoomProjectionError::InvalidCompletedRound);

    let mut room = base.clone();
    room.scores.one = Wins::new(0).unwrap();
    assert_error(room, RoomProjectionError::InvalidScores);

    let mut room = base;
    room.scores.two = Wins::new(1).unwrap();
    assert_error(room, RoomProjectionError::InvalidCompletedRound);
}

#[test]
fn global_round_ids_cannot_trail_their_actual_round_numbers() {
    let mut room = later_running_view();
    room.round_id = Counter::new(1).unwrap();
    assert_error(room, RoomProjectionError::InvalidCounter);

    let mut lifecycle = occupied();
    for _ in 0..2 {
        lifecycle.set_ready(SeatId::One, true).unwrap();
        lifecycle.set_ready(SeatId::Two, true).unwrap();
        lifecycle.set_ready(SeatId::One, false).unwrap();
    }
    start_initial(&mut lifecycle);
    lifecycle
        .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    start_next(&mut lifecycle);
    lifecycle
        .complete_running_tick(Some(RoundResult::Winner(SeatId::Two)))
        .unwrap();
    start_next(&mut lifecycle);
    let mut room = project(&lifecycle);
    assert!(validate_room_view(&room).is_ok());
    room.last_round.as_mut().unwrap().round_id = Counter::new(1).unwrap();
    assert_error(room, RoomProjectionError::InvalidCompletedRound);
}

#[test]
fn latest_draw_cannot_account_for_a_point_in_its_round() {
    let mut room = inter_countdown_view();
    assert!(validate_room_view(&room).is_ok());
    room.scores.one = Wins::new(1).unwrap();
    assert_error(room, RoomProjectionError::InvalidScores);
}

use crate::game::{DrawReason, RoundResult, SeatId};
use crate::protocol::{Counter, MAX_COUNTER, MAX_SAFE_TICK, Seat, Tick, WireRoundResult};
use crate::room::DepartureReason;

use super::super::{MatchResult, MatchWinnerReason, RoomPhase, RoundCount, Wins};
use super::test_support::{
    abandoned_terminal_view, advance, forfeit_terminal_view, match_interrupted_view, metadata,
    occupied, prematch_interrupted_view, project, score_terminal_lifecycle, start_initial,
    start_next, waiting_view,
};
use super::{RoomProjectionError, project_room_view, validate_room_view};

fn assert_error(room: super::super::RoomView, expected: RoomProjectionError) {
    assert_eq!(validate_room_view(&room), Err(expected));
}

fn phase_parts(room: &super::super::RoomView) -> (Tick, MatchResult, Tick) {
    let RoomPhase::MatchResult {
        ended_at,
        result,
        rematch_at,
    } = room.phase
    else {
        panic!("expected terminal match result");
    };
    (ended_at, result, rematch_at)
}

fn later_running_forfeit_view() -> super::super::RoomView {
    let mut lifecycle = occupied();
    start_initial(&mut lifecycle);
    lifecycle
        .complete_running_tick(Some(RoundResult::Winner(SeatId::One)))
        .unwrap();
    start_next(&mut lifecycle);
    lifecycle
        .depart_batch([None, Some(DepartureReason::Explicit)])
        .unwrap();
    project(&lifecycle)
}

#[test]
fn score_terminal_requires_exact_third_win_record_and_timing() {
    let lifecycle = score_terminal_lifecycle();
    let base = project_room_view(metadata(), &lifecycle).unwrap();
    let (ended_at, result, rematch_at) = phase_parts(&base);
    assert_eq!(
        result,
        MatchResult::Winner {
            seat: Seat::One,
            reason: MatchWinnerReason::RoundWins
        }
    );

    let mut room = base.clone();
    room.scores.one = Wins::new(2).unwrap();
    assert_error(room, RoomProjectionError::InvalidMatchResult);

    let mut room = base.clone();
    room.scores.two = Wins::new(3).unwrap();
    assert_error(room, RoomProjectionError::InvalidCompletedRound);

    let mut room = base.clone();
    room.last_round = None;
    assert_error(room, RoomProjectionError::InvalidScores);

    let mut room = base.clone();
    room.last_round.as_mut().unwrap().result = WireRoundResult::Draw {
        reason: crate::protocol::DrawReason::Timeout,
    };
    assert_error(room, RoomProjectionError::InvalidScores);

    let mut room = base.clone();
    room.last_round.as_mut().unwrap().round_id = Counter::new(room.round_id.get() - 1).unwrap();
    assert_error(room, RoomProjectionError::InvalidCompletedRound);

    let mut room = base.clone();
    room.phase = RoomPhase::MatchResult {
        ended_at,
        result,
        rematch_at: Tick::new(rematch_at.get() - 1).unwrap(),
    };
    assert_error(room, RoomProjectionError::InvalidPhase);

    let mut room = base.clone();
    room.tick = Tick::new(ended_at.get() - 1).unwrap();
    assert_error(room, RoomProjectionError::InvalidCompletedRound);

    let mut room = base;
    room.seats[0].rematch_ready = true;
    assert_error(room, RoomProjectionError::InvalidPhase);
}

#[test]
fn non_score_terminal_allows_only_current_or_immediately_prior_record() {
    assert!(validate_room_view(&forfeit_terminal_view()).is_ok());
    assert!(validate_room_view(&match_interrupted_view()).is_ok());
    assert!(validate_room_view(&later_running_forfeit_view()).is_ok());

    let mut room = later_running_forfeit_view();
    room.round_number = RoundCount::new(3).unwrap();
    assert_error(room, RoomProjectionError::InvalidCounter);

    let mut room = later_running_forfeit_view();
    room.last_round.as_mut().unwrap().round_id = room.round_id;
    assert_error(room, RoomProjectionError::InvalidCompletedRound);

    let mut room = later_running_forfeit_view();
    room.scores.one = Wins::new(3).unwrap();
    assert_error(room, RoomProjectionError::InvalidCompletedRound);

    let mut lifecycle = occupied();
    start_initial(&mut lifecycle);
    lifecycle
        .complete_running_tick(Some(RoundResult::Draw(DrawReason::Timeout)))
        .unwrap();
    lifecycle
        .depart_batch([Some(DepartureReason::Explicit), None])
        .unwrap();
    let mut room = project(&lifecycle);
    let (ended_at, result, rematch_at) = phase_parts(&room);
    room.tick = Tick::new(ended_at.get() + 1).unwrap();
    room.last_round.as_mut().unwrap().ended_at = room.tick;
    room.phase = RoomPhase::MatchResult {
        ended_at,
        result,
        rematch_at,
    };
    assert_error(room, RoomProjectionError::InvalidPhase);
}

#[test]
fn abandonment_requires_both_seats_vacant() {
    let mut room = abandoned_terminal_view();
    assert!(validate_room_view(&room).is_ok());
    room.seats[0].occupied = true;
    assert_error(room, RoomProjectionError::InvalidMatchResult);
}

#[test]
fn forfeit_requires_the_named_loser_vacant_but_allows_later_winner_departure() {
    let mut invalid = forfeit_terminal_view();
    invalid.seats[0].occupied = true;
    assert_error(invalid, RoomProjectionError::InvalidMatchResult);

    let mut lifecycle = occupied();
    start_initial(&mut lifecycle);
    lifecycle
        .depart_batch([Some(DepartureReason::Explicit), None])
        .unwrap();
    lifecycle
        .depart_batch([None, Some(DepartureReason::Explicit)])
        .unwrap();
    let room = project(&lifecycle);
    assert!(room.seats.iter().all(|seat| !seat.occupied));
    assert!(validate_room_view(&room).is_ok());
}

#[test]
fn terminal_rematch_flags_require_elapsed_hold_and_occupied_seats() {
    let mut lifecycle = score_terminal_lifecycle();
    let mut early = project(&lifecycle);
    early.seats[0].rematch_ready = true;
    assert_error(early, RoomProjectionError::InvalidPhase);

    advance(&mut lifecycle, 180);
    lifecycle.set_rematch_vote(SeatId::One, true).unwrap();
    let voted = project(&lifecycle);
    assert!(voted.seats[0].rematch_ready);
    assert!(validate_room_view(&voted).is_ok());

    let mut invalid = voted;
    invalid.seats[0].occupied = false;
    assert_error(invalid, RoomProjectionError::InvalidSeats);

    let mut impossible_consensus = project(&lifecycle);
    impossible_consensus.seats[1].rematch_ready = true;
    assert_error(impossible_consensus, RoomProjectionError::InvalidPhase);
}

#[test]
fn prematch_interruption_requires_frozen_zeroed_match_facts() {
    let base = prematch_interrupted_view();
    assert!(validate_room_view(&base).is_ok());

    let mut room = base.clone();
    room.round_number = RoundCount::new(1).unwrap();
    assert_error(room, RoomProjectionError::InvalidPhase);

    let mut room = base.clone();
    room.scores.one = Wins::new(1).unwrap();
    assert_error(room, RoomProjectionError::InvalidScores);

    let mut room = base.clone();
    room.last_round = later_running_forfeit_view().last_round;
    assert_error(room, RoomProjectionError::InvalidCompletedRound);

    let mut room = base.clone();
    room.seats[0].ready = true;
    assert_error(room, RoomProjectionError::InvalidSeats);

    let mut room = base;
    let RoomPhase::Interrupted { ended_at, reason } = room.phase else {
        unreachable!();
    };
    room.tick = Tick::new(ended_at.get() + 1).unwrap();
    room.phase = RoomPhase::Interrupted { ended_at, reason };
    assert_error(room, RoomProjectionError::InvalidPhase);
}

#[test]
fn validator_accepts_typed_wire_maxima_without_overflowing_deadline_checks() {
    let mut waiting = waiting_view();
    waiting.round_id = Counter::new(MAX_COUNTER).unwrap();
    waiting.tick = Tick::new(MAX_SAFE_TICK).unwrap();
    assert!(validate_room_view(&waiting).is_ok());

    let mut interrupted = prematch_interrupted_view();
    interrupted.round_id = Counter::new(MAX_COUNTER).unwrap();
    interrupted.tick = Tick::new(MAX_SAFE_TICK).unwrap();
    let RoomPhase::Interrupted { reason, .. } = interrupted.phase else {
        unreachable!();
    };
    interrupted.phase = RoomPhase::Interrupted {
        ended_at: Tick::new(MAX_SAFE_TICK).unwrap(),
        reason,
    };
    assert!(validate_room_view(&interrupted).is_ok());

    let mut terminal = project(&score_terminal_lifecycle());
    let (_, result, _) = phase_parts(&terminal);
    terminal.tick = Tick::new(MAX_SAFE_TICK).unwrap();
    terminal.last_round.as_mut().unwrap().ended_at = terminal.tick;
    terminal.phase = RoomPhase::MatchResult {
        ended_at: terminal.tick,
        result,
        rematch_at: terminal.tick,
    };
    assert_error(terminal, RoomProjectionError::InvalidPhase);
}

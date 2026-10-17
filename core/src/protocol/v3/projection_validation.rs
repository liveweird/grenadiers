use super::super::{Seat, Tick, WireRoundResult};
use super::{
    CountdownKind, MatchResult, MatchWinnerReason, RoomPhase, RoomProjectionError, RoomView,
    SeatStatus,
};

pub fn validate_room_view(room: &RoomView) -> Result<(), RoomProjectionError> {
    if room.rules_version == super::super::RulesVersion::ClassicV3 {
        return Err(RoomProjectionError::UnsupportedRulesVersion);
    }
    if room.round_id.get() < room.round_number.get() {
        return Err(RoomProjectionError::InvalidCounter);
    }
    validate_seats(room)?;
    validate_last_round(room)?;
    match &room.phase {
        RoomPhase::Waiting => validate_waiting(room),
        RoomPhase::Countdown {
            kind: CountdownKind::Initial,
            ends_at,
        } => validate_initial_countdown(room, *ends_at),
        RoomPhase::Countdown {
            kind: CountdownKind::InterRound,
            ends_at,
        } => validate_inter_countdown(room, *ends_at),
        RoomPhase::Running => validate_running(room),
        RoomPhase::Result { next_countdown_at } => validate_result(room, *next_countdown_at),
        RoomPhase::MatchResult {
            ended_at,
            result,
            rematch_at,
        } => validate_match_result(room, *ended_at, *result, *rematch_at),
        RoomPhase::Interrupted { ended_at, .. } => validate_interrupted(room, *ended_at),
    }
}

fn validate_seats(room: &RoomView) -> Result<(), RoomProjectionError> {
    if room.seats[0].seat != Seat::One || room.seats[1].seat != Seat::Two {
        return Err(RoomProjectionError::InvalidSeats);
    }
    let rematch_phase = matches!(room.phase, RoomPhase::MatchResult { .. });
    let readiness_phase = matches!(
        room.phase,
        RoomPhase::Waiting
            | RoomPhase::Countdown {
                kind: CountdownKind::Initial,
                ..
            }
    );
    if room.seats.iter().any(|seat| {
        (!seat.occupied && (seat.ready || seat.rematch_ready))
            || (seat.ready && !readiness_phase)
            || (seat.rematch_ready && !rematch_phase)
    }) {
        return Err(RoomProjectionError::InvalidSeats);
    }
    Ok(())
}

fn validate_last_round(room: &RoomView) -> Result<(), RoomProjectionError> {
    let round_number = room.round_number.get();
    let score_sum = u32::from(room.scores.one.get()) + u32::from(room.scores.two.get());
    let Some(last) = room.last_round else {
        return (score_sum == 0)
            .then_some(())
            .ok_or(RoomProjectionError::InvalidScores);
    };
    if last.ended_at > room.tick
        || last.round_number.get() > round_number
        || last.round_id > room.round_id
        || last.round_id.get() < last.round_number.get()
        || score_sum > last.round_number.get()
    {
        return Err(RoomProjectionError::InvalidCompletedRound);
    }
    match last.result {
        WireRoundResult::Winner { seat } if score(room, seat) == 0 => {
            return Err(RoomProjectionError::InvalidScores);
        }
        WireRoundResult::Draw { .. } if score_sum >= last.round_number.get() => {
            return Err(RoomProjectionError::InvalidScores);
        }
        _ => {}
    }
    Ok(())
}

fn validate_waiting(room: &RoomView) -> Result<(), RoomProjectionError> {
    if room.round_number.get() != 0
        || !scores_are(room, 0, 0)
        || room.last_round.is_some()
        || (both_occupied(&room.seats) && room.seats.iter().all(|seat| seat.ready))
    {
        return Err(RoomProjectionError::InvalidPhase);
    }
    no_rematch_votes(&room.seats)
}

fn validate_initial_countdown(room: &RoomView, ends_at: Tick) -> Result<(), RoomProjectionError> {
    if room.round_number.get() != 0
        || !scores_are(room, 0, 0)
        || room.last_round.is_some()
        || !both_occupied(&room.seats)
        || room.seats.iter().any(|seat| !seat.ready)
        || !future_within(room.tick, ends_at, 180)
    {
        return Err(RoomProjectionError::InvalidPhase);
    }
    no_rematch_votes(&room.seats)
}

fn validate_inter_countdown(room: &RoomView, ends_at: Tick) -> Result<(), RoomProjectionError> {
    let Some(last) = room.last_round else {
        return Err(RoomProjectionError::InvalidCompletedRound);
    };
    if room.round_number.get() == 0
        || !both_occupied(&room.seats)
        || any_ready(&room.seats)
        || last.round_number.get() != room.round_number.get()
        || last.round_id >= room.round_id
        || !future_within(room.tick, ends_at, 180)
        || !scores_below_three(room)
    {
        return Err(RoomProjectionError::InvalidPhase);
    }
    Ok(())
}

fn validate_running(room: &RoomView) -> Result<(), RoomProjectionError> {
    let actual = room.round_number.get();
    if actual == 0
        || !both_occupied(&room.seats)
        || any_ready(&room.seats)
        || !scores_below_three(room)
    {
        return Err(RoomProjectionError::InvalidPhase);
    }
    match room.last_round {
        None if actual == 1 && scores_are(room, 0, 0) => Ok(()),
        Some(last)
            if last.round_number.get().checked_add(1) == Some(actual)
                && last.round_id < room.round_id =>
        {
            Ok(())
        }
        _ => Err(RoomProjectionError::InvalidCompletedRound),
    }
}

fn validate_result(room: &RoomView, next: Tick) -> Result<(), RoomProjectionError> {
    let Some(last) = room.last_round else {
        return Err(RoomProjectionError::InvalidCompletedRound);
    };
    if room.round_number.get() == 0
        || !both_occupied(&room.seats)
        || any_ready(&room.seats)
        || !scores_below_three(room)
        || last.round_number.get() != room.round_number.get()
        || last.round_id != room.round_id
        || last.ended_at > room.tick
        || room.tick >= next
        || deadline(last.ended_at, 180) != Some(next)
    {
        return Err(RoomProjectionError::InvalidPhase);
    }
    Ok(())
}

fn validate_match_result(
    room: &RoomView,
    ended_at: Tick,
    result: MatchResult,
    rematch_at: Tick,
) -> Result<(), RoomProjectionError> {
    if room.round_number.get() == 0
        || ended_at > room.tick
        || deadline(ended_at, 180) != Some(rematch_at)
        || room.seats.iter().any(|seat| seat.ready)
        || (room.tick < rematch_at && room.seats.iter().any(|seat| seat.rematch_ready))
        || (both_occupied(&room.seats) && room.seats.iter().all(|seat| seat.rematch_ready))
        || room.last_round.is_some_and(|last| last.ended_at > ended_at)
    {
        return Err(RoomProjectionError::InvalidPhase);
    }
    match result {
        MatchResult::Winner {
            seat,
            reason: MatchWinnerReason::RoundWins,
        } => validate_score_winner(room, seat, ended_at),
        MatchResult::Winner {
            seat,
            reason: MatchWinnerReason::Forfeit,
        } => validate_forfeit(room, seat),
        MatchResult::Interrupted { .. } => validate_non_score_terminal(room, false),
        MatchResult::Abandoned => validate_non_score_terminal(room, true),
    }
}

fn validate_forfeit(room: &RoomView, winner: Seat) -> Result<(), RoomProjectionError> {
    let loser = match winner {
        Seat::One => &room.seats[1],
        Seat::Two => &room.seats[0],
    };
    if loser.occupied {
        return Err(RoomProjectionError::InvalidMatchResult);
    }
    validate_non_score_terminal(room, false)
}

fn validate_score_winner(
    room: &RoomView,
    winner: Seat,
    ended_at: Tick,
) -> Result<(), RoomProjectionError> {
    let Some(last) = room.last_round else {
        return Err(RoomProjectionError::InvalidCompletedRound);
    };
    let other = match winner {
        Seat::One => Seat::Two,
        Seat::Two => Seat::One,
    };
    if score(room, winner) != 3
        || score(room, other) >= 3
        || last.round_number.get() != room.round_number.get()
        || last.round_id != room.round_id
        || last.ended_at != ended_at
        || last.result != (WireRoundResult::Winner { seat: winner })
    {
        return Err(RoomProjectionError::InvalidMatchResult);
    }
    Ok(())
}

fn validate_non_score_terminal(
    room: &RoomView,
    abandoned: bool,
) -> Result<(), RoomProjectionError> {
    if !scores_below_three(room) || (abandoned && !both_vacant(&room.seats)) {
        return Err(RoomProjectionError::InvalidMatchResult);
    }
    match room.last_round {
        None if room.round_number.get() == 1 => Ok(()),
        Some(last) if last.round_number.get() == room.round_number.get() => Ok(()),
        Some(last)
            if last.round_number.get().checked_add(1) == Some(room.round_number.get())
                && last.round_id < room.round_id =>
        {
            Ok(())
        }
        _ => Err(RoomProjectionError::InvalidCompletedRound),
    }
}

fn validate_interrupted(room: &RoomView, ended_at: Tick) -> Result<(), RoomProjectionError> {
    if ended_at != room.tick
        || room.round_number.get() != 0
        || !scores_are(room, 0, 0)
        || room.last_round.is_some()
        || any_ready(&room.seats)
    {
        return Err(RoomProjectionError::InvalidPhase);
    }
    Ok(())
}

fn score(room: &RoomView, seat: Seat) -> u8 {
    match seat {
        Seat::One => room.scores.one.get(),
        Seat::Two => room.scores.two.get(),
    }
}

fn scores_are(room: &RoomView, one: u8, two: u8) -> bool {
    room.scores.one.get() == one && room.scores.two.get() == two
}

fn scores_below_three(room: &RoomView) -> bool {
    room.scores.one.get() < 3 && room.scores.two.get() < 3
}

fn any_ready(seats: &[SeatStatus; 2]) -> bool {
    seats.iter().any(|seat| seat.ready || seat.rematch_ready)
}

fn no_rematch_votes(seats: &[SeatStatus; 2]) -> Result<(), RoomProjectionError> {
    (!seats.iter().any(|seat| seat.rematch_ready))
        .then_some(())
        .ok_or(RoomProjectionError::InvalidSeats)
}

fn both_occupied(seats: &[SeatStatus; 2]) -> bool {
    seats.iter().all(|seat| seat.occupied)
}

fn both_vacant(seats: &[SeatStatus; 2]) -> bool {
    seats.iter().all(|seat| !seat.occupied)
}

fn future_within(now: Tick, future: Tick, maximum: u64) -> bool {
    future.get() > now.get() && future.get() - now.get() <= maximum
}

fn deadline(start: Tick, duration: u64) -> Option<Tick> {
    start
        .get()
        .checked_add(duration)
        .and_then(|value| Tick::new(value).ok())
}

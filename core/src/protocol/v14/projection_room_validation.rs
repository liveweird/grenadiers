use super::{
    MatchResult, MatchWinnerReason, ProjectionError, Seat,
    projection_lifecycle_facts::{LifecycleConnection, LifecyclePhase, LifecycleRoom, connected},
};
use crate::protocol::Tick;

pub(crate) fn validate_v14(room: &super::RoomView) -> Result<(), ProjectionError> {
    validate(&LifecycleRoom::v14(room))
}

pub(crate) fn validate(room: &LifecycleRoom<'_>) -> Result<(), ProjectionError> {
    if room.round_id < room.round_number {
        return invalid();
    }
    validate_seats(room)?;
    validate_last_round(room)?;
    match room.phase {
        LifecyclePhase::Waiting => waiting(room),
        LifecyclePhase::CountdownInitial { ends_at } => initial(room, ends_at),
        LifecyclePhase::CountdownInterRound { ends_at } => inter(room, ends_at),
        LifecyclePhase::Running => running(room),
        LifecyclePhase::Result { hold_until } => round_result(room, hold_until),
        LifecyclePhase::MatchResult {
            ended_at,
            result,
            rematch_at,
        } => match_result(room, ended_at, result, rematch_at),
        LifecyclePhase::Interrupted { ended_at } => interrupted(room, ended_at),
    }
}

fn validate_seats(room: &LifecycleRoom<'_>) -> Result<(), ProjectionError> {
    let readiness = matches!(
        room.phase,
        LifecyclePhase::Waiting | LifecyclePhase::CountdownInitial { .. }
    );
    let rematch_at = match room.phase {
        LifecyclePhase::MatchResult { rematch_at, .. } => Some(rematch_at),
        _ => None,
    };
    if room.seats.iter().any(|seat| {
        let cpu = seat.is_cpu;
        let expected_cpu_ready = cpu
            && matches!(
                room.phase,
                LifecyclePhase::Waiting
                    | LifecyclePhase::CountdownInitial { .. }
                    | LifecyclePhase::CountdownInterRound { .. }
            );
        let expected_cpu_rematch = cpu && rematch_at.is_some_and(|at| room.tick >= at);
        seat.occupied != !matches!(seat.connection, LifecycleConnection::Vacant)
            || if cpu {
                seat.ready != expected_cpu_ready || seat.rematch_ready != expected_cpu_rematch
            } else {
                seat.ready && (!connected(seat.connection) || !readiness)
                    || seat.rematch_ready
                        && (!connected(seat.connection)
                            || rematch_at.is_none()
                            || rematch_at.is_some_and(|at| room.tick < at))
            }
            || !valid_disconnect(room, seat.connection)
    }) || rematch_consensus(room)
    {
        return invalid();
    }
    Ok(())
}

fn validate_last_round(room: &LifecycleRoom<'_>) -> Result<(), ProjectionError> {
    let sum: u32 = room
        .scores
        .iter()
        .map(|score| u32::from(score.wins.get()))
        .sum();
    let Some(last) = room.last_round else {
        return (sum == 0)
            .then_some(())
            .ok_or(ProjectionError::InvalidExpandedBinding);
    };
    if last.ended_at > room.tick
        || last.round_number.get() > room.round_number
        || last.round_id.get() > room.round_id
        || last.round_id.get() < last.round_number.get()
        || sum > last.round_number.get()
    {
        return invalid();
    }
    match last.result {
        super::RoundResult::Winner { seat } if score(room, seat) == 0 => invalid(),
        super::RoundResult::Draw { .. } if sum >= last.round_number.get() => invalid(),
        _ => Ok(()),
    }
}

fn waiting(room: &LifecycleRoom<'_>) -> Result<(), ProjectionError> {
    if room.round_number != 0
        || room.scores.iter().any(|score| score.wins.get() != 0)
        || room.last_round.is_some()
        || readiness_consensus(room)
        || any_rematch(room)
    {
        invalid()
    } else {
        Ok(())
    }
}
fn initial(room: &LifecycleRoom<'_>, ends_at: Tick) -> Result<(), ProjectionError> {
    if room.round_number != 0
        || room.scores.iter().any(|score| score.wins.get() != 0)
        || room.last_round.is_some()
        || !readiness_consensus(room)
        || !future_within(room.tick, ends_at, 180)
        || any_rematch(room)
    {
        invalid()
    } else {
        Ok(())
    }
}
fn inter(room: &LifecycleRoom<'_>, ends_at: Tick) -> Result<(), ProjectionError> {
    let Some(last) = room.last_round else {
        return invalid();
    };
    if room.round_number == 0
        || !countdown_participants(room)
        || any_human_ready(room)
        || last.round_number.get() != room.round_number
        || last.round_id.get() >= room.round_id
        || deadline(last.ended_at, 180).is_none_or(|hold| room.tick < hold)
        || !future_within(room.tick, ends_at, 180)
        || !scores_below(room)
    {
        invalid()
    } else {
        Ok(())
    }
}
fn running(room: &LifecycleRoom<'_>) -> Result<(), ProjectionError> {
    let round = room.round_number;
    if round == 0 || !nonfinal_reservations(room) || any_ready(room) || !scores_below(room) {
        return invalid();
    }
    match room.last_round {
        None if round == 1 && room.scores.iter().all(|score| score.wins.get() == 0) => Ok(()),
        Some(last)
            if last.round_number.get().checked_add(1) == Some(round)
                && last.round_id.get() < room.round_id =>
        {
            Ok(())
        }
        _ => invalid(),
    }
}
fn round_result(room: &LifecycleRoom<'_>, hold: Tick) -> Result<(), ProjectionError> {
    let Some(last) = room.last_round else {
        return invalid();
    };
    if room.round_number == 0
        || !nonfinal_reservations(room)
        || any_ready(room)
        || !scores_below(room)
        || last.round_number.get() != room.round_number
        || last.round_id.get() > room.round_id
        || deadline(last.ended_at, 180) != Some(hold)
    {
        invalid()
    } else {
        Ok(())
    }
}
fn match_result(
    room: &LifecycleRoom<'_>,
    ended: Tick,
    result: MatchResult,
    rematch: Tick,
) -> Result<(), ProjectionError> {
    if room.round_number == 0
        || ended > room.tick
        || deadline(ended, 180) != Some(rematch)
        || room.seats.iter().any(|seat| seat.ready)
        || room.tick < rematch && any_rematch(room)
        || all_occupied(room) && room.seats.iter().all(|seat| seat.rematch_ready)
        || room.last_round.is_some_and(|last| last.ended_at > ended)
    {
        return invalid();
    }
    match result {
        MatchResult::Winner {
            seat,
            reason: MatchWinnerReason::RoundWins,
        } => score_winner(room, seat, ended),
        MatchResult::Winner {
            seat,
            reason: MatchWinnerReason::Forfeit,
        } => {
            if room
                .seats
                .iter()
                .filter(|status| status.seat != seat)
                .any(|status| {
                    !matches!(
                        status.connection,
                        LifecycleConnection::Vacant | LifecycleConnection::Expired
                    )
                })
            {
                return invalid();
            }
            non_score_terminal(room, false)
        }
        MatchResult::Interrupted { .. } => non_score_terminal(room, false),
        MatchResult::Abandoned => non_score_terminal(room, true),
    }
}
fn score_winner(
    room: &LifecycleRoom<'_>,
    winner: Seat,
    ended: Tick,
) -> Result<(), ProjectionError> {
    let Some(last) = room.last_round else {
        return invalid();
    };
    if score(room, winner) != room.wins_to_finish
        || room
            .scores
            .iter()
            .any(|value| value.seat != winner && value.wins.get() >= room.wins_to_finish)
        || last.round_number.get() != room.round_number
        || last.round_id.get() != room.round_id
        || last.ended_at != ended
        || last.result != (super::RoundResult::Winner { seat: winner })
    {
        invalid()
    } else {
        Ok(())
    }
}
fn non_score_terminal(room: &LifecycleRoom<'_>, abandoned: bool) -> Result<(), ProjectionError> {
    if !scores_below(room)
        || abandoned
            && room.seats.iter().any(|seat| {
                !seat.is_cpu
                    && !matches!(
                        seat.connection,
                        LifecycleConnection::Vacant | LifecycleConnection::Expired
                    )
            })
    {
        return invalid();
    }
    match room.last_round {
        None if room.round_number == 1 => Ok(()),
        Some(last) if last.round_number.get() == room.round_number => Ok(()),
        Some(last)
            if last.round_number.get().checked_add(1) == Some(room.round_number)
                && last.round_id.get() < room.round_id =>
        {
            Ok(())
        }
        _ => invalid(),
    }
}
fn interrupted(room: &LifecycleRoom<'_>, ended: Tick) -> Result<(), ProjectionError> {
    if ended != room.tick
        || room.round_number != 0
        || room.scores.iter().any(|score| score.wins.get() != 0)
        || room.last_round.is_some()
        || any_ready(room)
    {
        invalid()
    } else {
        Ok(())
    }
}

fn score(room: &LifecycleRoom<'_>, seat: Seat) -> u8 {
    room.scores[seat.index()].wins.get()
}
fn scores_below(room: &LifecycleRoom<'_>) -> bool {
    room.scores
        .iter()
        .all(|score| score.wins.get() < room.wins_to_finish)
}
fn any_ready(room: &LifecycleRoom<'_>) -> bool {
    room.seats
        .iter()
        .any(|seat| seat.ready || seat.rematch_ready)
}
fn any_rematch(room: &LifecycleRoom<'_>) -> bool {
    room.seats.iter().any(|seat| seat.rematch_ready)
}
fn all_occupied(room: &LifecycleRoom<'_>) -> bool {
    room.seats.iter().all(|seat| seat.occupied)
}
fn nonfinal_reservations(room: &LifecycleRoom<'_>) -> bool {
    all_occupied(room)
        && room
            .seats
            .iter()
            .filter(|seat| !matches!(seat.connection, LifecycleConnection::Expired))
            .count()
            >= 2
}
fn readiness_consensus(room: &LifecycleRoom<'_>) -> bool {
    let mut eligible = 0;
    let mut connected_human = false;
    let complete = room.seats.iter().all(|seat| match seat.connection {
        LifecycleConnection::Connected | LifecycleConnection::ServerControlled => {
            eligible += 1;
            connected_human |= !seat.is_cpu;
            seat.ready
        }
        LifecycleConnection::Expired => true,
        LifecycleConnection::Vacant | LifecycleConnection::Disconnected { .. } => false,
    });
    complete && eligible >= 2 && connected_human
}
fn countdown_participants(room: &LifecycleRoom<'_>) -> bool {
    let eligible = room
        .seats
        .iter()
        .filter(|seat| connected(seat.connection))
        .count();
    let connected_human = room
        .seats
        .iter()
        .any(|seat| !seat.is_cpu && connected(seat.connection));
    eligible >= 2
        && connected_human
        && room.seats.iter().all(|seat| {
            connected(seat.connection) || matches!(seat.connection, LifecycleConnection::Expired)
        })
}
fn any_human_ready(room: &LifecycleRoom<'_>) -> bool {
    room.seats.iter().any(|seat| !seat.is_cpu && seat.ready)
}
fn rematch_consensus(room: &LifecycleRoom<'_>) -> bool {
    if !matches!(room.phase, LifecyclePhase::MatchResult { .. }) {
        return false;
    }
    let mut eligible = 0;
    let mut connected_human = false;
    let complete = room.seats.iter().all(|seat| match seat.connection {
        LifecycleConnection::Connected | LifecycleConnection::ServerControlled => {
            eligible += 1;
            connected_human |= !seat.is_cpu;
            seat.rematch_ready
        }
        LifecycleConnection::Expired => true,
        LifecycleConnection::Vacant | LifecycleConnection::Disconnected { .. } => false,
    });
    complete && eligible >= 2 && connected_human
}
fn valid_disconnect(room: &LifecycleRoom<'_>, value: LifecycleConnection) -> bool {
    let LifecycleConnection::Disconnected { expires_at } = value else {
        return true;
    };
    future_within(room.tick, expires_at, 600)
        && match room.phase {
            LifecyclePhase::MatchResult { ended_at, .. } => expires_at
                .get()
                .checked_sub(ended_at.get())
                .is_some_and(|duration| duration <= 600),
            _ => true,
        }
}
fn future_within(now: Tick, future: Tick, maximum: u64) -> bool {
    future > now && future.get() - now.get() <= maximum
}
fn deadline(start: Tick, duration: u64) -> Option<Tick> {
    start
        .get()
        .checked_add(duration)
        .and_then(|value| Tick::new(value).ok())
}
fn invalid<T>() -> Result<T, ProjectionError> {
    Err(ProjectionError::InvalidExpandedBinding)
}

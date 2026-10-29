use super::{ProjectionError, RoomPhase, RoomView, Seat};

pub fn validate_room_view(room: &RoomView) -> Result<(), ProjectionError> {
    let base = room.configuration.domain().base();
    let count = base.controllers().len();
    if room.rules_version.get() != 7
        || room.map_id != super::MapId::GeneratedV2
        || !(2..=4).contains(&count)
        || room.seats.len() != count
        || room.scores.len() != count
        || base.controllers().first() != Some(&crate::room::SeatControl::Human)
        || room.configuration.domain().storm_cloud() != room.settings.environment.domain()
        || room.available_items
            != super::ItemSet::from_mask(
                super::ItemMask::new(crate::game::EnabledItems::ALL.bits())
                    .map_err(|_| ProjectionError::InvalidItems)?,
            )
        || !room
            .settings
            .base
            .enabled_items()
            .is_subset_of(room.available_items)
        || room
            .seats
            .iter()
            .enumerate()
            .any(|(index, status)| status.seat.index() != index || !valid_controller(room, index))
        || room
            .scores
            .iter()
            .enumerate()
            .any(|(index, score)| score.seat.index() != index)
        || room.settings_editor
            != matches!(
                room.seats[0].connection,
                super::ConnectionStatus::Connected | super::ConnectionStatus::Disconnected { .. }
            )
            .then_some(Seat::One)
        || room
            .scores
            .iter()
            .any(|score| score.wins.get() > room.settings.base.wins_to_finish())
        || room
            .last_round
            .is_some_and(|round| !valid_result(round.result, count))
        || matches!(
            room.phase,
            RoomPhase::MatchResult {
                result: super::MatchResult::Winner { seat, .. },
                ..
            } if seat.index() >= count
        )
    {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    let phase = match room.phase {
        RoomPhase::Waiting => {
            crate::protocol::v14::projection_lifecycle_facts::LifecyclePhase::Waiting
        }
        RoomPhase::Countdown {
            kind: super::CountdownKind::Initial,
            ends_at,
        } => crate::protocol::v14::projection_lifecycle_facts::LifecyclePhase::CountdownInitial {
            ends_at,
        },
        RoomPhase::Countdown {
            kind: super::CountdownKind::InterRound,
            ends_at,
        } => {
            crate::protocol::v14::projection_lifecycle_facts::LifecyclePhase::CountdownInterRound {
                ends_at,
            }
        }
        RoomPhase::Running => {
            crate::protocol::v14::projection_lifecycle_facts::LifecyclePhase::Running
        }
        RoomPhase::Result { hold_until } => {
            crate::protocol::v14::projection_lifecycle_facts::LifecyclePhase::Result { hold_until }
        }
        RoomPhase::MatchResult {
            ended_at,
            result,
            rematch_at,
        } => crate::protocol::v14::projection_lifecycle_facts::LifecyclePhase::MatchResult {
            ended_at,
            result,
            rematch_at,
        },
        RoomPhase::Interrupted { ended_at, .. } => {
            crate::protocol::v14::projection_lifecycle_facts::LifecyclePhase::Interrupted {
                ended_at,
            }
        }
    };
    crate::protocol::v14::projection_room_validation::validate(
        &crate::protocol::v14::projection_lifecycle_facts::LifecycleRoom {
            tick: room.tick,
            round_id: room.round_id.get(),
            round_number: room.round_number.get(),
            wins_to_finish: room.settings.base.wins_to_finish(),
            scores: &room.scores,
            last_round: room.last_round,
            seats: &room.seats,
            phase,
        },
    )
}

pub(crate) use validate_room_view as validate_room;

fn valid_result(result: super::RoundResult, count: usize) -> bool {
    match result {
        super::RoundResult::Winner { seat } => seat.index() < count,
        super::RoundResult::Draw { .. } => true,
    }
}

fn valid_controller(room: &RoomView, index: usize) -> bool {
    use super::{ConnectionStatus, Controller, Difficulty};
    let status = &room.seats[index];
    let reserved = !matches!(status.connection, ConnectionStatus::Vacant);
    if status.occupied != reserved || (!status.occupied && (status.ready || status.rematch_ready)) {
        return false;
    }
    match (
        room.configuration.domain().base().controllers()[index],
        status.controller,
        status.connection,
    ) {
        (
            crate::room::SeatControl::Human,
            Controller::Human,
            ConnectionStatus::ServerControlled,
        ) => false,
        (crate::room::SeatControl::Human, Controller::Human, _) => true,
        (
            crate::room::SeatControl::Cpu,
            Controller::Cpu { difficulty },
            ConnectionStatus::ServerControlled,
        ) => {
            let ready = matches!(
                &room.phase,
                RoomPhase::Waiting | RoomPhase::Countdown { .. }
            );
            let rematch_ready = matches!(
                &room.phase,
                RoomPhase::MatchResult { rematch_at, .. } if room.tick >= *rematch_at
            );
            status.occupied
                && status.ready == ready
                && status.rematch_ready == rematch_ready
                && room
                    .configuration
                    .domain()
                    .base()
                    .cpu_difficulty()
                    .is_some_and(|expected| {
                        matches!(
                            (expected, difficulty),
                            (crate::room::CpuDifficulty::Easy, Difficulty::Easy)
                                | (crate::room::CpuDifficulty::Standard, Difficulty::Standard)
                        )
                    })
        }
        _ => false,
    }
}

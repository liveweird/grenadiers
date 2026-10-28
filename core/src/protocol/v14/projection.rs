use super::{ConnectionStatus, Controller, MatchResult, RoomPhase, RoomView, RoundResult, Seat};
use crate::room::SeatControl;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionError {
    InvalidRules,
    InvalidClock,
    InvalidArena,
    InvalidActors,
    InvalidBombs,
    InvalidFlames,
    InvalidPickups,
    InvalidResult,
    InvalidEffects,
    InvalidItems,
    InvalidReports,
    InvalidEvent,
    InvalidExpandedBinding,
    EncodingFailed,
}

impl std::fmt::Display for ProjectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ProjectionError {}

impl Seat {
    pub(crate) const fn index(self) -> usize {
        match self {
            Self::One => 0,
            Self::Two => 1,
            Self::Three => 2,
            Self::Four => 3,
        }
    }
}

fn valid_result(result: RoundResult, count: usize) -> bool {
    match result {
        RoundResult::Winner { seat } => seat.index() < count,
        RoundResult::Draw { .. } => true,
    }
}

pub fn validate_room_view(room: &RoomView) -> Result<(), ProjectionError> {
    let configured = room.configuration.domain().controllers();
    let count = configured.len();
    if room.seats.len() != count || room.scores.len() != count || !(2..=4).contains(&count) {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    if room.rules_version.get() < 6
        && (count != 2
            || room.configuration.domain().board() != crate::room::BoardSize::FifteenByThirteen)
    {
        return Err(ProjectionError::InvalidRules);
    }
    validate_identity_and_settings(room)?;
    if let Some(editor) = room.settings_editor {
        let creator = &room.seats[Seat::One.index()];
        if editor != Seat::One
            || creator.controller != Controller::Human
            || matches!(
                creator.connection,
                ConnectionStatus::Vacant
                    | ConnectionStatus::Expired
                    | ConnectionStatus::ServerControlled
            )
        {
            return Err(ProjectionError::InvalidExpandedBinding);
        }
    }
    if room
        .seats
        .iter()
        .enumerate()
        .any(|(index, status)| status.seat.index() != index)
        || room
            .scores
            .iter()
            .enumerate()
            .any(|(index, score)| score.seat.index() != index)
    {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    for status in &room.seats {
        let expected_control = configured[status.seat.index()];
        let valid = match (expected_control, status.controller, status.connection) {
            (SeatControl::Human, Controller::Human, ConnectionStatus::ServerControlled) => false,
            (SeatControl::Human, Controller::Human, _) => true,
            (
                SeatControl::Cpu,
                Controller::Cpu { difficulty },
                ConnectionStatus::ServerControlled,
            ) => {
                status.occupied
                    && room
                        .configuration
                        .domain()
                        .cpu_difficulty()
                        .is_some_and(|value| {
                            matches!(
                                (value, difficulty),
                                (crate::room::CpuDifficulty::Easy, super::Difficulty::Easy)
                                    | (
                                        crate::room::CpuDifficulty::Standard,
                                        super::Difficulty::Standard
                                    )
                            )
                        })
            }
            _ => false,
        };
        if !valid {
            return Err(ProjectionError::InvalidExpandedBinding);
        }
        let reserved = !matches!(status.connection, ConnectionStatus::Vacant);
        if status.occupied != reserved
            || (!status.occupied && (status.ready || status.rematch_ready))
        {
            return Err(ProjectionError::InvalidExpandedBinding);
        }
    }
    if room
        .scores
        .iter()
        .any(|score| score.wins.get() > room.settings.wins_to_finish())
    {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    if room
        .last_round
        .is_some_and(|round| !valid_result(round.result, count))
    {
        return Err(ProjectionError::InvalidResult);
    }
    if let RoomPhase::MatchResult {
        result: MatchResult::Winner { seat, .. },
        ..
    } = room.phase
        && seat.index() >= count
    {
        return Err(ProjectionError::InvalidResult);
    }
    super::projection_room_validation::validate(room)
}

fn validate_identity_and_settings(room: &RoomView) -> Result<(), ProjectionError> {
    let expected_map = match room.rules_version.get() {
        1 => super::MapId::FixedV1,
        2..=4 => super::MapId::GeneratedV1,
        5..=6 => super::MapId::GeneratedV2,
        _ => return Err(ProjectionError::InvalidRules),
    };
    if room.map_id != expected_map
        || room.available_items
            != super::projection_supported_items::available_items_for(room.rules_version)
        || !room
            .settings
            .enabled_items()
            .is_subset_of(room.available_items)
    {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    let legacy = match room.rules_version.get() {
        1 => Some(crate::protocol::v8::RulesVersion::ClassicV1),
        2 => Some(crate::protocol::v8::RulesVersion::ClassicV2),
        3 => Some(crate::protocol::v8::RulesVersion::ClassicV3),
        _ => None,
    };
    if legacy.is_some_and(|version| room.settings != super::GameSettings::for_rules(version)) {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    Ok(())
}

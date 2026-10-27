use crate::game::{GenerationRecord, SeatId};
use crate::protocol::{Counter, Seat};
use crate::room::{ControllerKind, CpuDifficulty, CpuPolicy, MatchLifecycle};

use super::{
    ConnectionStatus, Controller, Difficulty, ExpandedCapture, ProjectionError, RoomMetadata,
    RoomMode, RoomView, SeatStatus,
};

#[derive(Clone)]
pub struct BoundPublication {
    pub(crate) room: RoomView,
    pub(crate) physics: crate::protocol::v8::BoundPublication,
}

impl std::fmt::Debug for BoundPublication {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BoundPublication")
            .field("room", &self.room)
            .field("physics", &"<validated v11 physics>")
            .finish()
    }
}

pub fn project_expanded_room(
    metadata: RoomMetadata,
    lifecycle: &MatchLifecycle,
) -> Result<RoomView, ProjectionError> {
    let RoomMetadata {
        settings,
        settings_revision,
        settings_editor,
        room_id,
        room_code,
        revision,
        rules_version,
        map_id,
        mode,
    } = metadata;
    let common = crate::protocol::v11::project_expanded_room(
        crate::protocol::v8::RoomMetadata {
            settings,
            settings_revision,
            settings_editor,
            room_id,
            room_code,
            revision,
            rules_version,
            map_id,
        },
        lifecycle,
    )?;
    let room = room_from_common(common, mode)?;
    validate_room_view(&room)?;
    Ok(room)
}

pub fn bind_expanded(
    room: &RoomView,
    state_epoch: Counter,
    capture: &ExpandedCapture,
    generation: Option<GenerationRecord>,
) -> Result<BoundPublication, ProjectionError> {
    let common = checked_common_room(room)?;
    let physics = crate::protocol::v11::bind_expanded(&common, state_epoch, capture, generation)?;
    Ok(BoundPublication {
        room: room.clone(),
        physics,
    })
}

pub fn validate_room_view(room: &RoomView) -> Result<(), ProjectionError> {
    checked_common_room(room).map(|_| ())
}

pub(crate) fn checked_common_room(
    room: &RoomView,
) -> Result<crate::protocol::v8::RoomView, ProjectionError> {
    validate_metadata(room)?;
    let common = crate::protocol::v8::RoomView {
        settings: room.settings,
        settings_revision: room.settings_revision,
        settings_editor: room.settings_editor,
        available_items: room.available_items,
        room_id: room.room_id.clone(),
        room_code: room.room_code.clone(),
        round_id: room.round_id,
        revision: room.revision,
        tick: room.tick,
        rules_version: room.rules_version,
        map_id: room.map_id,
        round_number: room.round_number,
        scores: room.scores,
        last_round: room.last_round,
        seats: room.seats.map(common_seat),
        phase: room.phase.clone(),
    };
    crate::protocol::v11::validate_expanded_room(&common)?;
    Ok(common)
}

fn validate_metadata(room: &RoomView) -> Result<(), ProjectionError> {
    if room.seats[0].seat != Seat::One || room.seats[1].seat != Seat::Two {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    let valid = match room.mode {
        RoomMode::HumanDuel => room.seats.iter().all(|seat| {
            seat.controller == Controller::Human
                && seat.connection != ConnectionStatus::ServerControlled
        }),
        RoomMode::Solo { difficulty } => {
            let human_eligible = matches!(
                room.seats[0].connection,
                ConnectionStatus::Connected | ConnectionStatus::Disconnected { .. }
            );
            let valid_editor = room.settings_editor == human_eligible.then_some(Seat::One);
            difficulty == Difficulty::Standard
                && valid_editor
                && room.seats[0].controller == Controller::Human
                && room.seats[0].connection != ConnectionStatus::ServerControlled
                && room.seats[1].controller == Controller::Cpu { difficulty }
                && room.seats[1].occupied
                && room.seats[1].connection == ConnectionStatus::ServerControlled
        }
    };
    valid
        .then_some(())
        .ok_or(ProjectionError::InvalidExpandedBinding)
}

fn common_seat(seat: SeatStatus) -> crate::protocol::v8::SeatStatus {
    let connection = match seat.connection {
        ConnectionStatus::Vacant => crate::protocol::v8::ConnectionStatus::Vacant,
        ConnectionStatus::Connected | ConnectionStatus::ServerControlled => {
            crate::protocol::v8::ConnectionStatus::Connected
        }
        ConnectionStatus::Disconnected { expires_at } => {
            crate::protocol::v8::ConnectionStatus::Disconnected { expires_at }
        }
        ConnectionStatus::Expired => crate::protocol::v8::ConnectionStatus::Expired,
    };
    crate::protocol::v8::SeatStatus {
        seat: seat.seat,
        occupied: seat.occupied,
        ready: seat.ready,
        rematch_ready: seat.rematch_ready,
        connection,
    }
}

fn room_from_common(
    common: crate::protocol::v8::RoomView,
    mode: crate::room::RoomMode,
) -> Result<RoomView, ProjectionError> {
    let wire_mode = match mode {
        crate::room::RoomMode::HumanDuel => RoomMode::HumanDuel,
        crate::room::RoomMode::Solo(profile)
            if matches!(
                profile.policy,
                CpuPolicy::UtilityV1
                    | CpuPolicy::UtilityV2
                    | CpuPolicy::UtilityV3
                    | CpuPolicy::UtilityV4
                    | CpuPolicy::UtilityV5
                    | CpuPolicy::UtilityV6
                    | CpuPolicy::UtilityV7
                    | CpuPolicy::UtilityV8
                    | CpuPolicy::UtilityV9
            ) && profile.difficulty == CpuDifficulty::Standard =>
        {
            RoomMode::Solo {
                difficulty: Difficulty::Standard,
            }
        }
        crate::room::RoomMode::Solo(_) => return Err(ProjectionError::InvalidExpandedBinding),
    };
    let mut index = 0;
    let seats = common.seats.map(|seat| {
        let domain_seat = SeatId::ALL[index];
        index += 1;
        let controller = match mode.controller(domain_seat) {
            ControllerKind::Human => Controller::Human,
            ControllerKind::Cpu(_) => Controller::Cpu {
                difficulty: Difficulty::Standard,
            },
        };
        let connection = match controller {
            Controller::Cpu { .. } => ConnectionStatus::ServerControlled,
            Controller::Human => human_connection(seat.connection),
        };
        SeatStatus {
            seat: seat.seat,
            occupied: seat.occupied,
            ready: seat.ready,
            rematch_ready: seat.rematch_ready,
            controller,
            connection,
        }
    });
    Ok(RoomView {
        settings: common.settings,
        settings_revision: common.settings_revision,
        settings_editor: common.settings_editor,
        available_items: common.available_items,
        mode: wire_mode,
        room_id: common.room_id,
        room_code: common.room_code,
        round_id: common.round_id,
        revision: common.revision,
        tick: common.tick,
        rules_version: common.rules_version,
        map_id: common.map_id,
        round_number: common.round_number,
        scores: common.scores,
        last_round: common.last_round,
        seats,
        phase: common.phase,
    })
}

const fn human_connection(value: crate::protocol::v8::ConnectionStatus) -> ConnectionStatus {
    match value {
        crate::protocol::v8::ConnectionStatus::Vacant => ConnectionStatus::Vacant,
        crate::protocol::v8::ConnectionStatus::Connected => ConnectionStatus::Connected,
        crate::protocol::v8::ConnectionStatus::Disconnected { expires_at } => {
            ConnectionStatus::Disconnected { expires_at }
        }
        crate::protocol::v8::ConnectionStatus::Expired => ConnectionStatus::Expired,
    }
}

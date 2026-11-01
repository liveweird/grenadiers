use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use grenadiers_core::protocol::v19::{ErrorCode, GameServerMessage, ManualActionKind};
use grenadiers_core::protocol::{Counter, RoomId};
use tokio::time::Instant;

use super::{
    command::{Command, SequencedCommand},
    connection::{ConnectionState, Membership},
    rooms::{
        BindingKey, GameRooms, LairSettingsAction, LiveAdmission as Admission, RoomError,
        RosterAction as RoomAction,
    },
    socket::{send_error, send_game, send_raw},
};
use crate::admission::{DenialOperation, GameplayOperation};

mod environment_settings;

pub(super) async fn dispatch(
    socket: &mut WebSocket,
    command: SequencedCommand,
    rooms: &Arc<GameRooms>,
    state: &mut ConnectionState,
) -> bool {
    let (sequence, body) = command.into_parts();
    match body {
        Command::CreateRoom { configuration } => {
            if state.membership.is_some() {
                return reply_error(socket, ErrorCode::AlreadyInRoom, sequence).await;
            }
            let Some(next_generation) = state.generation.checked_add(1) else {
                return reply_error(socket, ErrorCode::CapacityReached, sequence).await;
            };
            let binding = BindingKey {
                connection_id: state.connection_id,
                generation: next_generation,
            };
            // Retain the handler permit through the correlated response attempt.
            let _room_permit = match super::peer_policy::acquire(
                socket,
                state,
                GameplayOperation::Create,
                sequence,
            )
            .await
            {
                Ok(permit) => permit,
                Err(alive) => return alive,
            };
            let result = rooms
                .create_live(binding, sequence, std::time::Instant::now(), configuration)
                .await;
            admit(
                socket,
                result,
                binding,
                state,
                sequence,
                DenialOperation::Create,
            )
            .await
        }
        Command::JoinRoom { room_code } => {
            if state.membership.is_some() {
                return reply_error(socket, ErrorCode::AlreadyInRoom, sequence).await;
            }
            let Some(next_generation) = state.generation.checked_add(1) else {
                return reply_error(socket, ErrorCode::CapacityReached, sequence).await;
            };
            let binding = BindingKey {
                connection_id: state.connection_id,
                generation: next_generation,
            };
            let _room_permit =
                match super::peer_policy::acquire(socket, state, GameplayOperation::Join, sequence)
                    .await
                {
                    Ok(permit) => permit,
                    Err(alive) => return alive,
                };
            let result = rooms
                .join_live(&room_code, binding, sequence, std::time::Instant::now())
                .await;
            admit(
                socket,
                result,
                binding,
                state,
                sequence,
                DenialOperation::Join,
            )
            .await
        }
        Command::ResumeRoom {
            room_id,
            resume_token,
        } => {
            if state.membership.is_some() {
                return reply_error(socket, ErrorCode::AlreadyInRoom, sequence).await;
            }
            let Some(next_generation) = state.generation.checked_add(1) else {
                return reply_error(socket, ErrorCode::CapacityReached, sequence).await;
            };
            let binding = BindingKey {
                connection_id: state.connection_id,
                generation: next_generation,
            };
            let _room_permit = match super::peer_policy::acquire(
                socket,
                state,
                GameplayOperation::Resume,
                sequence,
            )
            .await
            {
                Ok(permit) => permit,
                Err(alive) => return alive,
            };
            let result = rooms
                .resume_live(
                    room_id,
                    binding,
                    sequence,
                    resume_token,
                    std::time::Instant::now(),
                )
                .await;
            admit(
                socket,
                result,
                binding,
                state,
                sequence,
                DenialOperation::Resume,
            )
            .await
        }
        Command::LeaveRoom { room_id, round_id } => {
            leave(
                socket,
                rooms,
                room_id,
                u64::from(round_id.get()),
                state,
                sequence,
            )
            .await
        }
        Command::Ready {
            room_id,
            round_id,
            ready,
            settings_revision,
        } => {
            action(
                socket,
                rooms,
                room_id,
                u64::from(round_id.get()),
                RoomAction::Ready {
                    ready,
                    settings_revision: settings_revision.get(),
                },
                state,
                sequence,
            )
            .await
        }
        Command::UpdateSettings {
            room_id,
            round_id,
            settings_revision,
            settings,
        } => {
            environment_settings::dispatch(
                socket,
                rooms,
                room_id,
                u64::from(round_id.get()),
                LairSettingsAction {
                    settings_revision: settings_revision.get(),
                    settings: match grenadiers_core::protocol::v13::expanded_settings(settings.base)
                    {
                        Ok(settings) => settings,
                        Err(_) => {
                            return reply_error(socket, ErrorCode::InvalidMessage, sequence).await;
                        }
                    },
                    cloud: if settings.environment.storm_cloud {
                        grenadiers_core::game::StormCloudConfig::Single
                    } else {
                        grenadiers_core::game::StormCloudConfig::Disabled
                    },
                    tuning: grenadiers_core::room::RespawnLairTuning::new(
                        settings.environment.lairs.enabled,
                        grenadiers_core::room::RespawnLairSettings::new(
                            settings.environment.lairs.spawn_seconds,
                        )
                        .expect("v19 parser validates lair settings"),
                    ),
                    spike_pits: grenadiers_core::room::SpikePitTuning::new(
                        settings.environment.spike_pits.enabled,
                        grenadiers_core::room::SpikePitSettings::new(
                            settings.environment.spike_pits.warning_ticks.into(),
                            settings.environment.spike_pits.raised_ticks.into(),
                        )
                        .expect("v19 parser validates spike-pit settings"),
                    ),
                },
                state,
                sequence,
            )
            .await
        }
        Command::Movement {
            room_id,
            round_id,
            direction,
        } => {
            action(
                socket,
                rooms,
                room_id,
                u64::from(round_id.get()),
                RoomAction::Movement(direction.map(environment_settings::domain_direction)),
                state,
                sequence,
            )
            .await
        }
        Command::ManualAction {
            room_id,
            round_id,
            action_id,
            action: manual,
        } => match manual_room_action(manual, action_id) {
            Some(manual) => {
                action(
                    socket,
                    rooms,
                    room_id,
                    u64::from(round_id.get()),
                    manual,
                    state,
                    sequence,
                )
                .await
            }
            None => reply_error(socket, ErrorCode::InvalidMessage, sequence).await,
        },
        Command::Rematch {
            room_id,
            round_id,
            ready,
        } => {
            action(
                socket,
                rooms,
                room_id,
                u64::from(round_id.get()),
                RoomAction::Rematch(ready),
                state,
                sequence,
            )
            .await
        }
    }
}

const fn manual_room_action(kind: ManualActionKind, id: Counter) -> Option<RoomAction> {
    match kind {
        ManualActionKind::Bomb => Some(RoomAction::Bomb(id.get())),
        ManualActionKind::Equipment => Some(RoomAction::Equipment(id.get())),
    }
}

async fn admit(
    socket: &mut WebSocket,
    result: Result<Admission, RoomError>,
    binding: BindingKey,
    state: &mut ConnectionState,
    sequence: Counter,
    operation: DenialOperation,
) -> bool {
    super::peer_policy::record_room_result(state, operation, &result);
    match result {
        Ok(admission) => {
            if send_raw(
                socket,
                Message::Text(admission.personal_frame().to_string().into()),
            )
            .await
            .is_err()
            {
                return false;
            }
            state.generation = binding.generation;
            state.membership = Some(Membership {
                admission,
                binding,
                replacement_closed: false,
            });
            true
        }
        Err(error) => room_error(socket, error, sequence).await,
    }
}

async fn leave(
    socket: &mut WebSocket,
    rooms: &GameRooms,
    room_id: RoomId,
    round_id: u64,
    state: &mut ConnectionState,
    sequence: Counter,
) -> bool {
    let Some(member) = state.membership.as_ref() else {
        return reply_error(socket, ErrorCode::NotInRoom, sequence).await;
    };
    if *member.admission.room_id() != room_id {
        return reply_error(socket, ErrorCode::StaleRoom, sequence).await;
    }
    let Some(next_generation) = state.generation.checked_add(1) else {
        return reply_error(socket, ErrorCode::CapacityReached, sequence).await;
    };
    match rooms
        .leave_live(&member.admission, member.binding, room_id.clone(), round_id)
        .await
    {
        Ok(()) => {
            let mut member = state.membership.take().expect("membership exists");
            member.admission.disarm();
            state.generation = next_generation;
            state.deadlines.restart_unseated(Instant::now());
            send_game(socket, &GameServerMessage::LeftRoom { room_id, sequence })
                .await
                .is_ok()
        }
        Err(error) => room_error(socket, error, sequence).await,
    }
}

async fn action(
    socket: &mut WebSocket,
    rooms: &GameRooms,
    room_id: RoomId,
    round_id: u64,
    action: RoomAction,
    state: &ConnectionState,
    sequence: Counter,
) -> bool {
    let Some(member) = state.membership.as_ref() else {
        return reply_error(socket, ErrorCode::NotInRoom, sequence).await;
    };
    match rooms
        .action_live(&member.admission, member.binding, room_id, round_id, action)
        .await
    {
        Ok(()) => true,
        Err(error) => room_error(socket, error, sequence).await,
    }
}

pub(super) async fn room_error(
    socket: &mut WebSocket,
    error: RoomError,
    sequence: Counter,
) -> bool {
    let code = match error {
        RoomError::RateLimited => ErrorCode::RateLimited,
        RoomError::CapacityReached => ErrorCode::CapacityReached,
        RoomError::RoomNotFound => ErrorCode::RoomNotFound,
        RoomError::RoomFull => ErrorCode::RoomFull,
        RoomError::StaleRoom => ErrorCode::StaleRoom,
        RoomError::StaleSettings => ErrorCode::StaleSettings,
        RoomError::SettingsLocked => ErrorCode::SettingsLocked,
        RoomError::InvalidPhase => ErrorCode::InvalidPhase,
        RoomError::DuplicateAction => ErrorCode::DuplicateAction,
        RoomError::ResumeRejected => ErrorCode::ResumeRejected,
        RoomError::UnsupportedConfiguration => ErrorCode::UnsupportedConfiguration,
    };
    reply_error(socket, code, sequence).await
}

pub(super) async fn reply_error(
    socket: &mut WebSocket,
    code: ErrorCode,
    sequence: Counter,
) -> bool {
    send_error(socket, code, Some(sequence)).await.is_ok()
}

#[cfg(test)]
mod tests;

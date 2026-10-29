use grenadiers_core::protocol::v16::{
    GameClientMessage, ManualActionKind, ResumeToken, RoomSettings,
};
use grenadiers_core::protocol::{
    Counter, Direction, ProtocolVersionInput, RoomCode, RoomId, SessionToken,
};

pub(super) enum InboundMessage {
    Authenticate {
        protocol_version: ProtocolVersionInput,
        token: SessionToken,
    },
    Command(SequencedCommand),
}

pub(super) struct SequencedCommand {
    sequence: Counter,
    body: Command,
}

impl SequencedCommand {
    pub(super) const fn sequence(&self) -> Counter {
        self.sequence
    }

    pub(super) fn into_parts(self) -> (Counter, Command) {
        (self.sequence, self.body)
    }
}

pub(super) enum Command {
    CreateRoom {
        configuration: grenadiers_core::room::LairRoomConfiguration,
    },
    JoinRoom {
        room_code: RoomCode,
    },
    ResumeRoom {
        room_id: RoomId,
        resume_token: ResumeToken,
    },
    Ready {
        room_id: RoomId,
        round_id: Counter,
        ready: bool,
        settings_revision: Counter,
    },
    UpdateSettings {
        room_id: RoomId,
        round_id: Counter,
        settings_revision: Counter,
        settings: RoomSettings,
    },
    Movement {
        room_id: RoomId,
        round_id: Counter,
        direction: Option<Direction>,
    },
    ManualAction {
        room_id: RoomId,
        round_id: Counter,
        action_id: Counter,
        action: ManualActionKind,
    },
    LeaveRoom {
        room_id: RoomId,
        round_id: Counter,
    },
    Rematch {
        room_id: RoomId,
        round_id: Counter,
        ready: bool,
    },
}

impl From<GameClientMessage> for InboundMessage {
    fn from(message: GameClientMessage) -> Self {
        let (sequence, body) = match message {
            GameClientMessage::Authenticate {
                protocol_version,
                token,
            } => {
                return Self::Authenticate {
                    protocol_version,
                    token,
                };
            }
            GameClientMessage::CreateRoom {
                sequence,
                configuration,
            } => (
                sequence,
                Command::CreateRoom {
                    configuration: configuration.domain().clone(),
                },
            ),
            GameClientMessage::JoinRoom {
                sequence,
                room_code,
            } => (sequence, Command::JoinRoom { room_code }),
            GameClientMessage::ResumeRoom {
                sequence,
                room_id,
                resume_token,
            } => (
                sequence,
                Command::ResumeRoom {
                    room_id,
                    resume_token,
                },
            ),
            GameClientMessage::Ready {
                sequence,
                room_id,
                round_id,
                ready,
                settings_revision,
            } => (
                sequence,
                Command::Ready {
                    room_id,
                    round_id,
                    ready,
                    settings_revision,
                },
            ),
            GameClientMessage::UpdateSettings {
                sequence,
                room_id,
                round_id,
                settings_revision,
                settings,
            } => (
                sequence,
                Command::UpdateSettings {
                    room_id,
                    round_id,
                    settings_revision,
                    settings,
                },
            ),
            GameClientMessage::Movement {
                sequence,
                room_id,
                round_id,
                direction,
            } => (
                sequence,
                Command::Movement {
                    room_id,
                    round_id,
                    direction,
                },
            ),
            GameClientMessage::ManualAction {
                sequence,
                room_id,
                round_id,
                action_id,
                action,
            } => (
                sequence,
                Command::ManualAction {
                    room_id,
                    round_id,
                    action_id,
                    action,
                },
            ),
            GameClientMessage::LeaveRoom {
                sequence,
                room_id,
                round_id,
            } => (sequence, Command::LeaveRoom { room_id, round_id }),
            GameClientMessage::Rematch {
                sequence,
                room_id,
                round_id,
                ready,
            } => (
                sequence,
                Command::Rematch {
                    room_id,
                    round_id,
                    ready,
                },
            ),
        };
        Self::Command(SequencedCommand { sequence, body })
    }
}

#[cfg(test)]
mod tests {
    use grenadiers_core::protocol::v16::parse_client_message;

    use super::{Command, InboundMessage};

    #[test]
    fn configured_creation_preserves_its_checked_controller_difficulty_and_board() {
        for (difficulty, expected) in [
            ("easy", grenadiers_core::room::CpuDifficulty::Easy),
            ("standard", grenadiers_core::room::CpuDifficulty::Standard),
        ] {
            let message = parse_client_message(&format!(
                r#"{{"type":"createRoom","sequence":1,"configuration":{{"controllers":["human","cpu"],"cpuDifficulty":"{difficulty}","board":{{"width":15,"height":13}},"environment":{{"stormCloud":true,"lairs":{{"enabled":false,"spawnSeconds":10,"maxLivingMobs":3}}}}}}}}"#
            ))
            .unwrap();
            let InboundMessage::Command(command) = InboundMessage::from(message) else {
                panic!("create is sequenced");
            };
            let (_, Command::CreateRoom { configuration }) = command.into_parts() else {
                panic!("create body is retained");
            };
            assert_eq!(
                configuration,
                grenadiers_core::room::LairRoomConfiguration::new(
                    grenadiers_core::room::RoomConfiguration::solo(expected),
                    grenadiers_core::game::StormCloudConfig::Single,
                    grenadiers_core::room::LairTuning::new(
                        false,
                        grenadiers_core::room::LairSettings::DEFAULT,
                    ),
                )
            );
        }
    }

    #[test]
    fn conversion_keeps_sequence_round_settings_revision_and_action_id_distinct() {
        let ready = parse_client_message(
            r#"{"type":"ready","sequence":11,"roomId":"00000000000000000000000000000001","roundId":22,"ready":true,"settingsRevision":33}"#,
        )
        .unwrap();
        let InboundMessage::Command(ready) = InboundMessage::from(ready) else {
            panic!("ready is sequenced");
        };
        let (
            sequence,
            Command::Ready {
                round_id,
                settings_revision,
                ..
            },
        ) = ready.into_parts()
        else {
            panic!("ready body is retained");
        };
        assert_eq!(sequence.get(), 11);
        assert_eq!(round_id.get(), 22);
        assert_eq!(settings_revision.get(), 33);

        let manual = parse_client_message(
            r#"{"type":"manualAction","sequence":44,"roomId":"00000000000000000000000000000001","roundId":55,"actionId":66,"action":"equipment"}"#,
        )
        .unwrap();
        let InboundMessage::Command(manual) = InboundMessage::from(manual) else {
            panic!("manual action is sequenced");
        };
        let (
            sequence,
            Command::ManualAction {
                round_id,
                action_id,
                ..
            },
        ) = manual.into_parts()
        else {
            panic!("manual-action body is retained");
        };
        assert_eq!(sequence.get(), 44);
        assert_eq!(round_id.get(), 55);
        assert_eq!(action_id.get(), 66);
    }
}

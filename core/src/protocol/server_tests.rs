use jsonschema::{Validator, validator_for};
use serde_json::{Value, json};

use super::{
    Cell, Counter, DrawReason, EncodeServerMessageError, EntityId, ErrorCode, GameServerMessage,
    GameplayVersion, InterruptReason, MapId, Position, ProtocolVersionInput, RoomCode, RoomId,
    RoomPhase, RoomView, RulesVersion, Seat, SeatStatus, Tick, WireRoundResult,
    encode_server_message, enforce_server_message_size,
};

fn server_schema() -> Validator {
    let schema: Value =
        serde_json::from_str(include_str!("../../../api/gameplay.schema.json")).unwrap();
    validator_for(&schema).unwrap()
}

fn definition_schema(name: &str) -> Validator {
    let mut schema: Value =
        serde_json::from_str(include_str!("../../../api/gameplay.schema.json")).unwrap();
    schema["$ref"] = json!(format!("#/$defs/{name}"));
    validator_for(&schema).unwrap()
}

fn room(phase: RoomPhase) -> RoomView {
    RoomView {
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        round_id: Counter::new(1).unwrap(),
        revision: Counter::new(2).unwrap(),
        tick: Tick::new(90).unwrap(),
        rules_version: RulesVersion::ClassicV1,
        map_id: MapId::FixedV1,
        seats: [
            SeatStatus {
                seat: Seat::One,
                occupied: true,
                ready: true,
            },
            SeatStatus {
                seat: Seat::Two,
                occupied: true,
                ready: true,
            },
        ],
        phase,
    }
}

fn assert_server_valid(message: &GameServerMessage) -> Value {
    let encoded = encode_server_message(message).unwrap();
    let value: Value = serde_json::from_str(&encoded).unwrap();
    assert!(
        server_schema().is_valid(&value),
        "invalid server JSON: {value}"
    );
    value
}

#[test]
fn every_server_lifecycle_variant_conforms_to_the_schema() {
    assert_server_valid(&GameServerMessage::Authenticated {
        protocol_version: GameplayVersion,
    });
    let incompatible = assert_server_valid(&GameServerMessage::IncompatibleProtocol {
        supported_version: ProtocolVersionInput::new(super::GAMEPLAY_PROTOCOL_VERSION),
    });
    assert_eq!(incompatible["supportedVersion"], 2);
    assert_server_valid(&GameServerMessage::RoomState {
        room: room(RoomPhase::Waiting),
        you_seat: Seat::One,
    });
    assert_server_valid(&GameServerMessage::LeftRoom {
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        sequence: Counter::new(2_147_483_647).unwrap(),
    });
    let error = assert_server_valid(&GameServerMessage::Error {
        code: ErrorCode::InvalidMessage,
        sequence: None,
    });
    assert!(error.get("sequence").is_some());
    assert!(error["sequence"].is_null());
}

#[test]
fn every_room_phase_and_round_outcome_conforms() {
    let phases = [
        RoomPhase::Waiting,
        RoomPhase::Countdown {
            ends_at: Tick::new(180).unwrap(),
        },
        RoomPhase::Running,
        RoomPhase::Result {
            ended_at: Tick::new(200).unwrap(),
            result: WireRoundResult::Winner { seat: Seat::One },
        },
        RoomPhase::Result {
            ended_at: Tick::new(201).unwrap(),
            result: WireRoundResult::Draw {
                reason: DrawReason::MutualElimination,
            },
        },
        RoomPhase::Result {
            ended_at: Tick::new(202).unwrap(),
            result: WireRoundResult::Draw {
                reason: DrawReason::Timeout,
            },
        },
    ];
    for phase in phases {
        assert_server_valid(&GameServerMessage::RoomState {
            room: room(phase),
            you_seat: Seat::Two,
        });
    }
}

#[test]
fn every_interruption_reason_conforms_without_inventing_a_winner() {
    for reason in [
        InterruptReason::ParticipantLeft,
        InterruptReason::ConnectionLost,
        InterruptReason::ServerOverloaded,
        InterruptReason::ServerStopped,
        InterruptReason::InternalError,
    ] {
        let value = assert_server_valid(&GameServerMessage::RoomState {
            room: room(RoomPhase::Interrupted {
                ended_at: Tick::new(300).unwrap(),
                reason,
            }),
            you_seat: Seat::One,
        });
        let phase = &value["room"]["phase"];
        assert!(phase.get("result").is_none());
        assert!(phase.get("seat").is_none());
    }

    let mut invalid = serde_json::to_value(GameServerMessage::RoomState {
        room: room(RoomPhase::Interrupted {
            ended_at: Tick::new(300).unwrap(),
            reason: InterruptReason::ParticipantLeft,
        }),
        you_seat: Seat::One,
    })
    .unwrap();
    invalid["room"]["phase"]["result"] = json!({"type":"winner", "seat":"one"});
    assert!(!server_schema().is_valid(&invalid));
}

#[test]
fn every_error_code_conforms_with_present_nullable_sequence() {
    let codes = [
        ErrorCode::InvalidMessage,
        ErrorCode::AuthenticationFailed,
        ErrorCode::ConnectionInUse,
        ErrorCode::NotAuthenticated,
        ErrorCode::AlreadyAuthenticated,
        ErrorCode::AlreadyInRoom,
        ErrorCode::NotInRoom,
        ErrorCode::RoomNotFound,
        ErrorCode::RoomFull,
        ErrorCode::StaleRoom,
        ErrorCode::InvalidPhase,
        ErrorCode::StaleSequence,
        ErrorCode::DuplicateAction,
        ErrorCode::RateLimited,
        ErrorCode::CapacityReached,
    ];
    for (index, code) in codes.into_iter().enumerate() {
        let sequence = (index % 2 == 0).then(|| Counter::new(1).unwrap());
        assert_server_valid(&GameServerMessage::Error { code, sequence });
    }
}

#[test]
fn scalar_constructors_enforce_wire_bounds() {
    assert!(Tick::new(9_007_199_254_740_991).is_ok());
    assert!(Tick::new(9_007_199_254_740_992).is_err());
    assert!(Counter::new(1).is_ok());
    assert!(Counter::new(2_147_483_647).is_ok());
    assert!(Counter::new(0).is_err());
    assert!(Counter::new(2_147_483_648).is_err());

    let cell = Cell::new(14, 12).unwrap();
    assert!(definition_schema("Cell").is_valid(&serde_json::to_value(cell).unwrap()));
    assert!(Cell::new(15, 12).is_err());
    assert!(Cell::new(14, 13).is_err());

    let position = Position::new(15_359, 13_311).unwrap();
    assert!(definition_schema("Position").is_valid(&serde_json::to_value(position).unwrap()));
    assert!(Position::new(15_360, 13_311).is_err());
    assert!(Position::new(15_359, 13_312).is_err());

    assert!(
        definition_schema("EntityId")
            .is_valid(&serde_json::to_value(EntityId::new(u32::MAX)).unwrap())
    );
}

#[test]
fn outbound_size_guard_uses_utf8_bytes() {
    assert!(enforce_server_message_size("a".repeat(65_536)).is_ok());
    assert_eq!(
        enforce_server_message_size("é".repeat(32_769)),
        Err(EncodeServerMessageError::MessageTooLarge)
    );
}

#[path = "v2_contract_tests.rs"]
mod v2;

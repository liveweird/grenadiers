use jsonschema::{Validator, validator_for};
use serde_json::{Value, json};

use super::super::{
    ARENA_TERRAIN_CELLS, ActorView, ArenaView, Counter, Direction, EntityId, GameState,
    MAX_COUNTER, MapId, Position, ProtocolVersionInput, RoomCode, RoomId, RulesVersion, Seat,
    Terrain, Tick, WireRoundResult,
};
use super::{
    CompletedRound, ConnectionStatus, CountdownKind, GameServerMessage, GameplayVersion,
    MatchInterruptReason, MatchResult, MatchWinnerReason, ResumeToken, RoomPhase, RoomView,
    RoundCount, Scores, SeatStatus, V5ErrorCode, Wins, encode_server_message,
};

fn server_schema() -> Validator {
    let schema: Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v5.schema.json")).unwrap();
    validator_for(&schema).unwrap()
}

fn empty_state() -> GameState {
    GameState {
        running_updates: 0,
        remaining_ticks: 0,
        arena: ArenaView {
            width: 15,
            height: 13,
            terrain: vec![Terrain::Floor; ARENA_TERRAIN_CELLS],
            blocks: vec![],
        },
        actors: [Seat::One, Seat::Two].map(|seat| ActorView {
            id: EntityId::new(0),
            seat,
            position: Position::new(0, 0).unwrap(),
            heading: Direction::North,
            speed_level: 1,
            alive: true,
            bomb_capacity: 1,
            bomb_range: 1,
            live_bomb_count: 0,
        }),
        bombs: vec![],
        flames: vec![],
        pickups: vec![],
    }
}

fn room(phase: RoomPhase) -> RoomView {
    RoomView {
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        round_id: Counter::new(7).unwrap(),
        revision: Counter::new(9).unwrap(),
        tick: Tick::new(540).unwrap(),
        rules_version: RulesVersion::ClassicV2,
        map_id: MapId::GeneratedV1,
        round_number: RoundCount::new(2).unwrap(),
        scores: Scores {
            one: Wins::new(1).unwrap(),
            two: Wins::new(1).unwrap(),
        },
        last_round: Some(CompletedRound {
            round_id: Counter::new(6).unwrap(),
            round_number: Counter::new(2).unwrap(),
            ended_at: Tick::new(360).unwrap(),
            result: WireRoundResult::Winner { seat: Seat::Two },
        }),
        seats: [
            SeatStatus {
                seat: Seat::One,
                occupied: true,
                ready: false,
                rematch_ready: false,
                connection: ConnectionStatus::Connected,
            },
            SeatStatus {
                seat: Seat::Two,
                occupied: true,
                ready: false,
                rematch_ready: true,
                connection: ConnectionStatus::Disconnected {
                    expires_at: Tick::new(600).unwrap(),
                },
            },
        ],
        phase,
    }
}

fn assert_valid(message: &GameServerMessage) -> Value {
    let encoded = encode_server_message(message).unwrap();
    let value = serde_json::from_str(&encoded).unwrap();
    assert!(server_schema().is_valid(&value), "schema rejected {value}");
    value
}

#[test]
fn server_envelope_variants_and_fixed_v5_authentication_conform() {
    let authenticated = assert_valid(&GameServerMessage::Authenticated {
        protocol_version: GameplayVersion,
    });
    assert_eq!(authenticated["protocolVersion"], 5);
    for version in [0, 3, 4, u16::MAX] {
        assert_valid(&GameServerMessage::IncompatibleProtocol {
            supported_version: ProtocolVersionInput::new(version),
        });
    }
    assert_valid(&GameServerMessage::RoomState {
        room: room(RoomPhase::Waiting),
    });
    assert_valid(&GameServerMessage::Admitted {
        sequence: Counter::new(1).unwrap(),
        room: room(RoomPhase::Waiting),
        you_seat: Seat::One,
        resume_token: ResumeToken::new("a".repeat(64)).unwrap(),
        last_bomb_action_id: None,
        state: None,
    });
    assert_valid(&GameServerMessage::Snapshot {
        room: room(RoomPhase::Running),
        state: empty_state(),
        events: vec![],
    });
    assert_valid(&GameServerMessage::LeftRoom {
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        sequence: Counter::new(1).unwrap(),
    });
    assert_valid(&GameServerMessage::Error {
        code: V5ErrorCode::InvalidMessage,
        sequence: None,
    });
    assert_valid(&GameServerMessage::SeatReplaced {
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
    });
    assert!(!server_schema().is_valid(&json!({"type":"authenticated","protocolVersion":3})));
    assert!(
        !server_schema().is_valid(&json!({"type":"incompatibleProtocol","supportedVersion":65536}))
    );
}

#[test]
fn personal_admission_requires_nullable_fields_and_shared_room_has_no_identity() {
    let admitted = assert_valid(&GameServerMessage::Admitted {
        sequence: Counter::new(MAX_COUNTER).unwrap(),
        room: room(RoomPhase::Running),
        you_seat: Seat::Two,
        resume_token: ResumeToken::new("b".repeat(64)).unwrap(),
        last_bomb_action_id: Some(Counter::new(MAX_COUNTER).unwrap()),
        state: Some(empty_state()),
    });
    assert!(admitted.get("events").is_none());
    for field in ["lastBombActionId", "state"] {
        let mut missing = admitted.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(!server_schema().is_valid(&missing));
    }
    let mut nulls = admitted.clone();
    nulls["lastBombActionId"] = Value::Null;
    nulls["state"] = Value::Null;
    assert!(server_schema().is_valid(&nulls));
    let shared = assert_valid(&GameServerMessage::RoomState {
        room: room(RoomPhase::Waiting),
    });
    let mut leaked = shared;
    leaked["youSeat"] = json!("one");
    assert!(!server_schema().is_valid(&leaked));
}

#[test]
fn every_connection_and_v5_error_code_conforms() {
    for connection in [
        ConnectionStatus::Vacant,
        ConnectionStatus::Connected,
        ConnectionStatus::Disconnected {
            expires_at: Tick::new(600).unwrap(),
        },
        ConnectionStatus::Expired,
    ] {
        let mut view = room(RoomPhase::Waiting);
        view.seats[0].connection = connection;
        assert_valid(&GameServerMessage::RoomState { room: view });
    }
    for code in [
        V5ErrorCode::InvalidMessage,
        V5ErrorCode::AuthenticationFailed,
        V5ErrorCode::ConnectionInUse,
        V5ErrorCode::NotAuthenticated,
        V5ErrorCode::AlreadyAuthenticated,
        V5ErrorCode::AlreadyInRoom,
        V5ErrorCode::NotInRoom,
        V5ErrorCode::RoomNotFound,
        V5ErrorCode::RoomFull,
        V5ErrorCode::StaleRoom,
        V5ErrorCode::InvalidPhase,
        V5ErrorCode::StaleSequence,
        V5ErrorCode::DuplicateAction,
        V5ErrorCode::RateLimited,
        V5ErrorCode::CapacityReached,
        V5ErrorCode::ResumeRejected,
    ] {
        assert_valid(&GameServerMessage::Error {
            code,
            sequence: None,
        });
    }
}

#[test]
fn every_phase_and_match_result_variant_conforms() {
    let phases = [
        RoomPhase::Waiting,
        RoomPhase::Countdown {
            kind: CountdownKind::Initial,
            ends_at: Tick::new(180).unwrap(),
        },
        RoomPhase::Countdown {
            kind: CountdownKind::InterRound,
            ends_at: Tick::new(360).unwrap(),
        },
        RoomPhase::Running,
        RoomPhase::Result {
            hold_until: Tick::new(540).unwrap(),
        },
        RoomPhase::MatchResult {
            ended_at: Tick::new(540).unwrap(),
            result: MatchResult::Winner {
                seat: Seat::One,
                reason: MatchWinnerReason::RoundWins,
            },
            rematch_at: Tick::new(720).unwrap(),
        },
        RoomPhase::MatchResult {
            ended_at: Tick::new(540).unwrap(),
            result: MatchResult::Winner {
                seat: Seat::Two,
                reason: MatchWinnerReason::Forfeit,
            },
            rematch_at: Tick::new(720).unwrap(),
        },
        RoomPhase::MatchResult {
            ended_at: Tick::new(540).unwrap(),
            result: MatchResult::Abandoned,
            rematch_at: Tick::new(720).unwrap(),
        },
        RoomPhase::MatchResult {
            ended_at: Tick::new(540).unwrap(),
            result: MatchResult::Interrupted {
                reason: MatchInterruptReason::ConnectionLost,
            },
            rematch_at: Tick::new(720).unwrap(),
        },
        RoomPhase::Interrupted {
            ended_at: Tick::new(20).unwrap(),
            reason: MatchInterruptReason::ServerStopped,
        },
    ];
    for phase in phases {
        assert_valid(&GameServerMessage::RoomState { room: room(phase) });
    }
    for reason in [
        MatchInterruptReason::ConnectionLost,
        MatchInterruptReason::ServerOverloaded,
        MatchInterruptReason::ServerStopped,
        MatchInterruptReason::InternalError,
    ] {
        assert_valid(&GameServerMessage::RoomState {
            room: room(RoomPhase::Interrupted {
                ended_at: Tick::new(20).unwrap(),
                reason,
            }),
        });
    }
}

#[test]
fn room_metadata_is_closed_required_nullable_and_bounded() {
    let value = assert_valid(&GameServerMessage::RoomState {
        room: room(RoomPhase::Waiting),
    });
    for field in ["roundNumber", "scores", "lastRound", "seats", "phase"] {
        let mut missing = value.clone();
        missing["room"].as_object_mut().unwrap().remove(field);
        assert!(
            !server_schema().is_valid(&missing),
            "accepted missing {field}"
        );
    }
    let mut null_last_round = value.clone();
    null_last_round["room"]["lastRound"] = Value::Null;
    assert!(server_schema().is_valid(&null_last_round));
    for (field, invalid) in [
        ("roundNumber", json!(2_147_483_648_u64)),
        ("scores", json!({"one":4,"two":0})),
        (
            "lastRound",
            json!({"roundId":6,"roundNumber":0,"endedAt":360,"result":{"type":"winner","seat":"one"}}),
        ),
    ] {
        let mut changed = value.clone();
        changed["room"][field] = invalid;
        assert!(
            !server_schema().is_valid(&changed),
            "accepted invalid {field}"
        );
    }
    let mut extra = value;
    extra["room"]["matchSeed"] = json!(1);
    assert!(!server_schema().is_valid(&extra));
}

#[test]
fn match_counter_scalars_enforce_their_wire_bounds() {
    assert_eq!(RoundCount::new(0).unwrap().get(), 0);
    assert_eq!(RoundCount::new(MAX_COUNTER).unwrap().get(), MAX_COUNTER);
    assert!(RoundCount::new(MAX_COUNTER + 1).is_err());
    assert_eq!(Wins::new(0).unwrap().get(), 0);
    assert_eq!(Wins::new(3).unwrap().get(), 3);
    assert!(Wins::new(4).is_err());
}

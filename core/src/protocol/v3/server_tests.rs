use jsonschema::{Validator, validator_for};
use serde_json::{Value, json};

use super::super::{
    ARENA_TERRAIN_CELLS, ActorView, ArenaView, BombView, Cell, Counter, Direction, EntityId,
    ErrorCode, EventKind, FlameView, GameState, MAX_COUNTER, MapId, PickupKind, PickupView,
    Position, ProtocolVersionInput, RoomCode, RoomId, RulesVersion, Seat, Terrain, Tick, WireEvent,
    WireRoundResult,
};
use super::{
    CompletedRound, CountdownKind, GameServerMessage, GameplayVersion, MatchInterruptReason,
    MatchResult, MatchWinnerReason, RoomPhase, RoomView, RoundCount, Scores, SeatStatus, Wins,
    encode_server_message,
};

fn server_schema() -> Validator {
    let schema: Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v3.schema.json")).unwrap();
    validator_for(&schema).unwrap()
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
            },
            SeatStatus {
                seat: Seat::Two,
                occupied: true,
                ready: false,
                rematch_ready: true,
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
fn server_envelope_variants_and_fixed_v3_authentication_conform() {
    let authenticated = assert_valid(&GameServerMessage::Authenticated {
        protocol_version: GameplayVersion,
    });
    assert_eq!(authenticated["protocolVersion"], 3);
    for version in [0, 2, 3, u16::MAX] {
        assert_valid(&GameServerMessage::IncompatibleProtocol {
            supported_version: ProtocolVersionInput::new(version),
        });
    }
    assert_valid(&GameServerMessage::RoomState {
        room: room(RoomPhase::Waiting),
        you_seat: Seat::One,
    });
    assert_valid(&GameServerMessage::LeftRoom {
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        sequence: Counter::new(1).unwrap(),
    });
    assert_valid(&GameServerMessage::Error {
        code: ErrorCode::InvalidMessage,
        sequence: None,
    });
    assert!(!server_schema().is_valid(&json!({"type":"authenticated","protocolVersion":2})));
    assert!(
        !server_schema().is_valid(&json!({"type":"incompatibleProtocol","supportedVersion":65536}))
    );
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
            next_countdown_at: Tick::new(540).unwrap(),
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
        assert_valid(&GameServerMessage::RoomState {
            room: room(phase),
            you_seat: Seat::Two,
        });
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
            you_seat: Seat::One,
        });
    }
}

#[test]
fn room_metadata_is_closed_required_nullable_and_bounded() {
    let value = assert_valid(&GameServerMessage::RoomState {
        room: room(RoomPhase::Waiting),
        you_seat: Seat::One,
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

fn cells() -> Vec<Cell> {
    (0..13)
        .flat_map(|row| (0..15).map(move |column| Cell::new(column, row).unwrap()))
        .collect()
}

fn maximal_snapshot() -> GameServerMessage {
    let tick = Tick::new(9_007_199_254_740_991).unwrap();
    let bomb = BombView {
        id: EntityId::new(u32::MAX),
        cell: Cell::new(14, 12).unwrap(),
        owner: EntityId::new(u32::MAX),
        range: 8,
        placed_at: tick,
        detonates_at: tick,
    };
    let all_cells = cells();
    let state = GameState {
        running_updates: 10_800,
        remaining_ticks: 10_800,
        arena: ArenaView {
            width: 15,
            height: 13,
            terrain: vec![Terrain::Floor; ARENA_TERRAIN_CELLS],
            blocks: all_cells.clone(),
        },
        actors: [Seat::One, Seat::Two].map(|seat| ActorView {
            id: EntityId::new(u32::MAX),
            seat,
            position: Position::new(15_359, 13_311).unwrap(),
            heading: Direction::North,
            speed_level: 4,
            alive: false,
            bomb_capacity: 8,
            bomb_range: 8,
            live_bomb_count: 8,
        }),
        bombs: vec![bomb; 16],
        flames: all_cells
            .iter()
            .copied()
            .map(|cell| FlameView {
                cell,
                expires_at: tick,
            })
            .collect(),
        pickups: all_cells
            .into_iter()
            .map(|cell| PickupView {
                cell,
                kind: PickupKind::MovementSpeed,
            })
            .collect(),
    };
    let mut room = room(RoomPhase::MatchResult {
        ended_at: tick,
        result: MatchResult::Winner {
            seat: Seat::Two,
            reason: MatchWinnerReason::RoundWins,
        },
        rematch_at: tick,
    });
    room.round_id = Counter::new(MAX_COUNTER).unwrap();
    room.revision = Counter::new(MAX_COUNTER).unwrap();
    room.round_number = RoundCount::new(MAX_COUNTER).unwrap();
    room.scores = Scores {
        one: Wins::new(3).unwrap(),
        two: Wins::new(3).unwrap(),
    };
    GameServerMessage::Snapshot {
        room,
        state,
        events: vec![
            WireEvent {
                tick,
                index: 63,
                kind: EventKind::BombDetonated { bomb },
            };
            64
        ],
    }
}

#[test]
fn maximal_v3_structure_with_pickups_and_events_fits_the_frame() {
    let message = maximal_snapshot();
    let encoded = encode_server_message(&message).unwrap();
    let value: Value = serde_json::from_str(&encoded).unwrap();
    assert!(server_schema().is_valid(&value));
    assert_eq!(value["state"]["pickups"].as_array().unwrap().len(), 195);
    assert_eq!(value["events"].as_array().unwrap().len(), 64);
    assert!(encoded.len() <= 65_536, "encoded {} bytes", encoded.len());
}

use jsonschema::{Validator, validator_for};
use serde_json::Value;

use super::super::{
    ARENA_TERRAIN_CELLS, ActorView, ArenaView, BombView, Cell, Counter, Direction, DrawReason,
    EncodeServerMessageError, EntityId, EventKind, FlameView, GameState, MAX_COUNTER, MapId,
    PickupKind, PickupView, Position, RoomCode, RoomId, RulesVersion, Seat, Terrain, Tick,
    WireEvent, WireRoundResult,
};
use super::{
    CompletedRound, ConnectionStatus, GameServerMessage, MatchInterruptReason, MatchResult,
    MatchWinnerReason, ResumeToken, RoomPhase, RoomView, RoundCount, Scores, SeatStatus, Wins,
    encode_server_message,
};

fn server_schema() -> Validator {
    let schema: Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v6.schema.json")).unwrap();
    validator_for(&schema).unwrap()
}

fn cells() -> Vec<Cell> {
    (0..13)
        .flat_map(|row| (0..15).map(move |column| Cell::new(column, row).unwrap()))
        .collect()
}

fn maximal_body() -> (RoomView, GameState, BombView) {
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
    let room = RoomView {
        room_id: RoomId::new("ffffffffffffffffffffffffffffffff").unwrap(),
        room_code: RoomCode::new("ZZZZZZZZ").unwrap(),
        round_id: Counter::new(MAX_COUNTER).unwrap(),
        revision: Counter::new(MAX_COUNTER).unwrap(),
        tick,
        rules_version: RulesVersion::ClassicV2,
        map_id: MapId::GeneratedV1,
        settings: super::GameSettings::for_rules(RulesVersion::ClassicV2),
        settings_revision: Counter::new(1).unwrap(),
        settings_editor: None,
        round_number: RoundCount::new(MAX_COUNTER).unwrap(),
        scores: Scores {
            one: Wins::new(3).unwrap(),
            two: Wins::new(3).unwrap(),
        },
        last_round: Some(CompletedRound {
            round_id: Counter::new(MAX_COUNTER).unwrap(),
            round_number: Counter::new(MAX_COUNTER).unwrap(),
            ended_at: tick,
            result: WireRoundResult::Draw {
                reason: DrawReason::MutualElimination,
            },
        }),
        seats: [Seat::One, Seat::Two].map(|seat| SeatStatus {
            seat,
            occupied: false,
            ready: false,
            rematch_ready: false,
            connection: ConnectionStatus::Disconnected { expires_at: tick },
        }),
        phase: RoomPhase::MatchResult {
            ended_at: tick,
            result: MatchResult::Winner {
                seat: Seat::Two,
                reason: MatchWinnerReason::RoundWins,
            },
            rematch_at: tick,
        },
    };
    (room, state, bomb)
}

fn assert_frame(message: &GameServerMessage, label: &str) -> usize {
    let encoded = encode_server_message(message).unwrap();
    let value: Value = serde_json::from_str(&encoded).unwrap();
    assert!(
        server_schema().is_valid(&value),
        "schema rejected {label}: {value}"
    );
    assert!(
        encoded.len() <= 65_536,
        "{label} encoded {} bytes",
        encoded.len()
    );
    eprintln!(
        "Maximal tested v6 structural {label}: {} UTF-8 bytes",
        encoded.len()
    );
    encoded.len()
}

#[test]
fn maximal_complete_snapshot_and_admitted_structures_fit_frames() {
    let winner = MatchResult::Winner {
        seat: Seat::Two,
        reason: MatchWinnerReason::RoundWins,
    };
    let interrupted = MatchResult::Interrupted {
        reason: MatchInterruptReason::ServerOverloaded,
    };
    assert!(
        serde_json::to_string(&winner).unwrap().len()
            > serde_json::to_string(&interrupted).unwrap().len()
    );
    let (room, state, bomb) = maximal_body();
    let snapshot = GameServerMessage::Snapshot {
        room: room.clone(),
        state: state.clone(),
        events: vec![
            WireEvent {
                tick: room.tick,
                index: 63,
                kind: EventKind::BombDetonated { bomb }
            };
            64
        ],
    };
    let admitted = GameServerMessage::Admitted {
        sequence: Counter::new(MAX_COUNTER).unwrap(),
        room,
        you_seat: Seat::Two,
        resume_token: ResumeToken::new("f".repeat(64)).unwrap(),
        last_bomb_action_id: Some(Counter::new(MAX_COUNTER).unwrap()),
        state: Some(state),
    };
    assert_frame(&snapshot, "snapshot");
    assert_frame(&admitted, "admitted");
}

#[test]
fn encoder_rejects_an_oversized_structural_message() {
    let (room, state, bomb) = maximal_body();
    let oversized = GameServerMessage::Snapshot {
        room,
        state,
        events: vec![
            WireEvent {
                tick: Tick::new(9_007_199_254_740_991).unwrap(),
                index: 63,
                kind: EventKind::BombDetonated { bomb }
            };
            256
        ],
    };
    assert_eq!(
        encode_server_message(&oversized),
        Err(EncodeServerMessageError::MessageTooLarge)
    );
}

use jsonschema::{Validator, validator_for};
use serde_json::Value;

use super::{
    ARENA_TERRAIN_CELLS, ActorView, ArenaView, BombView, Cell, Counter, Direction, DrawReason,
    EncodeServerMessageError, EntityId, EventKind, FlameView, GameServerMessage, GameState,
    MAX_COUNTER, MapId, PickupKind, PickupView, Position, RoomCode, RoomId, RoomPhase, RoomView,
    RulesVersion, Seat, SeatStatus, Terrain, Tick, WireEvent, WireRoundResult,
    encode_server_message,
};

fn server_schema() -> Validator {
    let schema: Value =
        serde_json::from_str(include_str!("../../../api/gameplay.schema.json")).unwrap();
    validator_for(&schema).unwrap()
}

fn room(phase: RoomPhase, tick: Tick) -> RoomView {
    RoomView {
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        round_id: Counter::new(1).unwrap(),
        revision: Counter::new(2).unwrap(),
        tick,
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

fn actor(id: u32, seat: Seat) -> ActorView {
    ActorView {
        id: EntityId::new(id),
        seat,
        position: Position::new(1_536, 1_536).unwrap(),
        heading: Direction::East,
        speed_level: 0,
        alive: true,
        bomb_capacity: 1,
        bomb_range: 2,
        live_bomb_count: 0,
    }
}

fn bomb(id: u32, cell: Cell, tick: Tick) -> BombView {
    BombView {
        id: EntityId::new(id),
        cell,
        owner: EntityId::new(10),
        range: 2,
        placed_at: tick,
        detonates_at: tick,
    }
}

fn state() -> GameState {
    GameState {
        running_updates: 1,
        remaining_ticks: 10_799,
        arena: ArenaView {
            width: 15,
            height: 13,
            terrain: vec![Terrain::Floor; ARENA_TERRAIN_CELLS],
            blocks: vec![Cell::new(4, 5).unwrap()],
        },
        actors: [actor(10, Seat::One), actor(20, Seat::Two)],
        bombs: vec![bomb(30, Cell::new(5, 5).unwrap(), Tick::new(100).unwrap())],
        flames: vec![FlameView {
            cell: Cell::new(6, 5).unwrap(),
            expires_at: Tick::new(130).unwrap(),
        }],
        pickups: Vec::new(),
    }
}

fn assert_valid(message: &GameServerMessage) -> (Value, usize) {
    let encoded = encode_server_message(message).unwrap();
    let value: Value = serde_json::from_str(&encoded).unwrap();
    assert!(server_schema().is_valid(&value), "schema rejected {value}");
    (value, encoded.len())
}

#[test]
fn full_snapshot_and_every_event_kind_conform_to_the_schema() {
    let tick = Tick::new(100).unwrap();
    let event_bomb = bomb(30, Cell::new(5, 5).unwrap(), tick);
    let message = GameServerMessage::Snapshot {
        room: room(RoomPhase::Running, tick),
        state: state(),
        events: vec![
            WireEvent {
                tick,
                index: 0,
                kind: EventKind::BombPlaced { bomb: event_bomb },
            },
            WireEvent {
                tick,
                index: 1,
                kind: EventKind::BombDetonated { bomb: event_bomb },
            },
            WireEvent {
                tick,
                index: 2,
                kind: EventKind::ActorEliminated {
                    id: EntityId::new(10),
                    seat: Seat::One,
                },
            },
            WireEvent {
                tick,
                index: 3,
                kind: EventKind::RoundEnded {
                    result: WireRoundResult::Winner { seat: Seat::Two },
                },
            },
        ],
    };

    let (value, _) = assert_valid(&message);
    assert_eq!(value["type"], "snapshot");
    assert_eq!(value["state"]["runningUpdates"], 1);
    assert_eq!(
        value["state"]["arena"]["terrain"].as_array().unwrap().len(),
        195
    );
    assert_eq!(value["events"][0]["kind"]["type"], "bombPlaced");
    assert_eq!(value["events"][3]["kind"]["type"], "roundEnded");
}

#[test]
fn result_snapshots_cover_both_wire_outcome_shapes() {
    let tick = Tick::new(10_800).unwrap();
    for result in [
        WireRoundResult::Winner { seat: Seat::One },
        WireRoundResult::Draw {
            reason: DrawReason::MutualElimination,
        },
        WireRoundResult::Draw {
            reason: DrawReason::Timeout,
        },
    ] {
        assert_valid(&GameServerMessage::Snapshot {
            room: room(
                RoomPhase::Result {
                    ended_at: tick,
                    result,
                },
                tick,
            ),
            state: state(),
            events: vec![WireEvent {
                tick,
                index: 0,
                kind: EventKind::RoundEnded { result },
            }],
        });
    }
}

fn all_cells() -> Vec<Cell> {
    (0..13)
        .flat_map(|row| (0..15).map(move |column| Cell::new(column, row).unwrap()))
        .collect()
}

fn maximal_structural_snapshot() -> GameServerMessage {
    let tick = Tick::new(9_007_199_254_740_991).unwrap();
    let cells = all_cells();
    let maximal_bomb = || BombView {
        id: EntityId::new(u32::MAX),
        cell: Cell::new(14, 12).unwrap(),
        owner: EntityId::new(u32::MAX),
        range: 8,
        placed_at: tick,
        detonates_at: tick,
    };
    let state = GameState {
        running_updates: 10_800,
        remaining_ticks: 10_800,
        arena: ArenaView {
            width: 15,
            height: 13,
            terrain: vec![Terrain::Floor; ARENA_TERRAIN_CELLS],
            blocks: cells.clone(),
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
        bombs: (0..16).map(|_| maximal_bomb()).collect(),
        pickups: cells
            .iter()
            .copied()
            .map(|cell| PickupView {
                cell,
                kind: PickupKind::MovementSpeed,
            })
            .collect(),
        flames: cells
            .into_iter()
            .map(|cell| FlameView {
                cell,
                expires_at: tick,
            })
            .collect(),
    };
    let mut room = room(
        RoomPhase::Result {
            ended_at: tick,
            result: WireRoundResult::Draw {
                reason: DrawReason::MutualElimination,
            },
        },
        tick,
    );
    room.rules_version = RulesVersion::ClassicV2;
    room.map_id = MapId::GeneratedV1;
    for seat in &mut room.seats {
        seat.occupied = false;
        seat.ready = false;
    }
    room.round_id = Counter::new(MAX_COUNTER).unwrap();
    room.revision = Counter::new(MAX_COUNTER).unwrap();
    GameServerMessage::Snapshot {
        room,
        state,
        events: (0..64)
            .map(|_| WireEvent {
                tick,
                index: 63,
                kind: EventKind::BombDetonated {
                    bomb: maximal_bomb(),
                },
            })
            .collect(),
    }
}

#[test]
fn maximal_bounded_structure_conforms_and_fits_the_frame_budget() {
    let message = maximal_structural_snapshot();
    let (_, encoded_bytes) = assert_valid(&message);

    eprintln!("Maximal v2 structural snapshot: {encoded_bytes} UTF-8 bytes");
    assert!(encoded_bytes <= 65_536, "encoded {encoded_bytes} bytes");
}

#[test]
fn encoder_rejects_an_independently_oversized_snapshot() {
    let mut message = maximal_structural_snapshot();
    let GameServerMessage::Snapshot { state, .. } = &mut message else {
        unreachable!();
    };
    state.arena.blocks = vec![Cell::new(14, 12).unwrap(); 10_000];

    assert_eq!(
        encode_server_message(&message),
        Err(EncodeServerMessageError::MessageTooLarge)
    );
}

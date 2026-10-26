use serde::Serialize;

use super::super::{
    Cell, Counter, Direction, DrawReason, EntityId, MAX_COUNTER, Position, RoomCode, RoomId, Seat,
    Tick, WireRoundResult,
};
use super::{
    ARENA_TERRAIN_CELLS, ActorView, ArenaView, BlastKind, BombTrigger, BombView, CompletedRound,
    ConnectionStatus, DamageGraceTicks, EffectKind, EffectView, EffectViews, EventIndex, EventKind,
    GameServerMessage, GameSettings, GameState, InventoryView, ItemMask, ItemSet, MapId,
    MatchInterruptReason, MatchResult, MatchWinnerReason, OilBarrelState, OilBarrelView,
    PickupKind, PickupView, ResumeToken, RoomPhase, RoomView, RoundCount, RulesVersion, Scores,
    SeatStatus, SnapshotEvents, Terrain, VisibleActorView, Wins, WireEvent, encode_server_message,
};

const FRAME_BYTES: usize = 65_536;

crate::protocol::expanded_frame_test_support::expanded_frame_test_support!();
fn largest_state(tick: Tick, cell: Cell, bomb: BombView) -> GameState {
    let effect = longest_effect();
    let visible: ActorView = VisibleActorView {
        id: EntityId::new(u32::MAX),
        seat: Seat::One,
        position: Position::new(15_359, 13_311).unwrap(),
        heading: longest([
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]),
        speed_level: 4,
        alive: false,
        bomb_capacity: 8,
        bomb_range: 8,
        live_bomb_count: 8,
        inventory: InventoryView {
            equipment: longest([None, Some(longest_equipment())]),
            weapon: longest([None, Some(longest_weapon())]),
        },
        effects: EffectViews::new(vec![effect; 16]).unwrap(),
        damage_grace_remaining_ticks: DamageGraceTicks::new(60).unwrap(),
        voluntary_actions_frozen: false,
        last_teleport_tick: Some(tick),
    }
    .into();
    let hidden: ActorView = super::HiddenActorView {
        id: EntityId::new(u32::MAX),
        seat: Seat::One,
        alive: false,
    }
    .into();
    let actor = longest([visible, hidden]);
    let terrain = longest([Terrain::Floor, Terrain::Wall]);
    let pickup = PickupView {
        cell,
        kind: longest_pickup(),
    };
    GameState {
        running_updates: 10_800,
        remaining_ticks: 10_800,
        physical_tick: tick,
        arena: ArenaView {
            width: 15,
            height: 13,
            terrain: vec![terrain; ARENA_TERRAIN_CELLS],
            blocks: vec![cell; 195],
        },
        actors: [actor.clone(), actor],
        bombs: vec![bomb; 16],
        flames: vec![
            super::FlameView {
                cell,
                expires_at: tick
            };
            195
        ],
        oil_barrels: vec![
            OilBarrelView {
                cell,
                state: OilBarrelState::Burning
            };
            8
        ],
        pickups: vec![pickup; 195],
    }
}

#[test]
fn synthetic_typed_overapproximation_proves_v11_frames_fit() {
    let tick = Tick::new(9_007_199_254_740_991).unwrap();
    let cell = Cell::new(14, 12).unwrap();
    let bomb = largest_bomb(tick, cell);
    let state = largest_state(tick, cell, bomb);
    let room = largest_room(tick);
    let events = SnapshotEvents::new(vec![longest_event(tick, cell, bomb); 96]).unwrap();
    let snapshot = GameServerMessage::Snapshot {
        room: room.clone(),
        state: state.clone(),
        events,
    };
    let admitted = GameServerMessage::Admitted {
        sequence: Counter::new(MAX_COUNTER).unwrap(),
        room,
        you_seat: Seat::One,
        resume_token: ResumeToken::new("f".repeat(64)).unwrap(),
        last_action_id: longest([None, Some(Counter::new(MAX_COUNTER).unwrap())]),
        state: longest([None, Some(state)]),
    };

    let snapshot_json = serde_json::to_string(&snapshot).unwrap();
    let admitted_json = serde_json::to_string(&admitted).unwrap();
    let GameServerMessage::Snapshot {
        room,
        state,
        events,
    } = &snapshot
    else {
        unreachable!()
    };
    eprintln!(
        "v11 synthetic upper bounds: snapshot={} admitted={} room={} state={} events={}",
        snapshot_json.len(),
        admitted_json.len(),
        encoded_len(room),
        encoded_len(state),
        encoded_len(events)
    );
    assert!(
        snapshot_json.len() < FRAME_BYTES,
        "snapshot bound is {}",
        snapshot_json.len()
    );
    assert!(
        admitted_json.len() < FRAME_BYTES,
        "admitted bound is {}",
        admitted_json.len()
    );
    assert_eq!(encode_server_message(&snapshot).unwrap(), snapshot_json);
    assert_eq!(encode_server_message(&admitted).unwrap(), admitted_json);
}

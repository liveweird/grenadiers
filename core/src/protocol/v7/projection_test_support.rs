use crate::game::{
    ActorInit, Direction as GameDirection, EntityId as GameEntityId, Position as GamePosition,
    RulesConfig, SeatId, SpeedLevel, State, World, fixed_arena,
};
use crate::protocol::{
    Counter, Direction, EntityId, MapId, Position, RoomCode, RoomId, RulesVersion, Seat, Tick,
};

use super::projection_facts::WorldFacts;
use super::*;

fn actor(id: u32, seat: Seat, position: (u16, u16)) -> VisibleActorView {
    VisibleActorView {
        id: EntityId::new(id),
        seat,
        position: Position::new(position.0, position.1).unwrap(),
        heading: Direction::East,
        speed_level: 0,
        alive: true,
        bomb_capacity: 2,
        bomb_range: 2,
        live_bomb_count: 0,
        inventory: InventoryView {
            equipment: None,
            weapon: None,
        },
        effects: EffectViews::default(),
        damage_grace_remaining_ticks: DamageGraceTicks::new(0).unwrap(),
    }
}

pub(super) fn base_world(tick: u64) -> WorldFacts {
    let running = u16::try_from(tick).unwrap();
    WorldFacts {
        rules_version: RulesVersion::ClassicV1,
        physical_tick: Tick::new(tick).unwrap(),
        running_updates: running,
        remaining_ticks: MAX_RUNNING_UPDATES - running,
        result: None,
        arena: ArenaView {
            width: 15,
            height: 13,
            terrain: vec![Terrain::Floor; ARENA_TERRAIN_CELLS],
            blocks: vec![],
        },
        actors: [
            actor(1, Seat::One, (512, 512)),
            actor(2, Seat::Two, (14_848, 12_800)),
        ],
        bombs: vec![],
        flames: vec![],
        pickups: vec![],
        available_items: ItemSet::empty(),
        enabled_items: ItemSet::empty(),
    }
}

pub(super) fn room(tick: u64, settings: GameSettings, available_items: ItemSet) -> RoomView {
    RoomView {
        settings,
        settings_revision: Counter::new(1).unwrap(),
        settings_editor: None,
        available_items,
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        round_id: Counter::new(1).unwrap(),
        revision: Counter::new(1).unwrap(),
        tick: Tick::new(tick).unwrap(),
        rules_version: RulesVersion::ClassicV1,
        map_id: MapId::FixedV1,
        round_number: RoundCount::new(1).unwrap(),
        scores: Scores {
            one: Wins::new(0).unwrap(),
            two: Wins::new(0).unwrap(),
        },
        last_round: None,
        seats: [Seat::One, Seat::Two].map(|seat| SeatStatus {
            seat,
            occupied: true,
            ready: false,
            rematch_ready: false,
            connection: ConnectionStatus::Connected,
        }),
        phase: RoomPhase::Running,
    }
}

pub(super) fn classic_state() -> State {
    classic_state_at(0)
}

pub(super) fn classic_state_at(tick: u64) -> State {
    let arena = fixed_arena();
    let spawns = arena.spawns().to_vec();
    State::new_at_tick(
        RulesConfig::classic_v1(),
        World::new(arena),
        [SeatId::One, SeatId::Two].map(|seat| ActorInit {
            id: GameEntityId(seat.index() as u32 + 1),
            seat,
            position: GamePosition::center(spawns[seat.index()]),
            heading: GameDirection::East,
            speed: SpeedLevel::Base,
        }),
        crate::game::Tick(tick),
    )
    .unwrap()
}

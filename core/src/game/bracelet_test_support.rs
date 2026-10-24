use std::collections::BTreeSet;

use super::{ActorInit, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    CellId, Direction, EnabledItems, EntityId, Equipment, InputAction, Position, RulesConfig,
    SeatId, SeatInput, SpeedLevel, Tick, World,
};
use crate::room::RoomSettings;

pub(crate) fn arena(blocks: BTreeSet<CellId>) -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if row == 0 || column == 0 || row == ARENA_HEIGHT - 1 || column == ARENA_WIDTH - 1 {
                terrain[usize::from(row * ARENA_WIDTH + column)] = Terrain::Wall;
            }
        }
    }
    Arena::new(terrain, blocks, [CellId::new(1, 1), CellId::new(13, 11)]).unwrap()
}

pub(crate) fn actor(id: u32, seat: SeatId, position: Position, heading: Direction) -> ActorInit {
    ActorInit {
        id: EntityId(id),
        seat,
        position,
        heading,
        speed: SpeedLevel::Base,
    }
}

pub(crate) fn state(
    enabled: EnabledItems,
    world: World,
    one: Position,
    two: Position,
    tick: Tick,
) -> State {
    let mut state = State::new_at_tick(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled)),
        world,
        [
            actor(1, SeatId::One, one, Direction::East),
            actor(2, SeatId::Two, two, Direction::West),
        ],
        tick,
    )
    .unwrap();
    if enabled.contains_bracelet() {
        state.replace_actor_equipment(SeatId::One, Equipment::Bracelet);
    }
    state
}

pub(crate) fn simple_state(enabled: EnabledItems, one: Position, two: Position) -> State {
    state(
        enabled,
        World::new(arena(BTreeSet::new())),
        one,
        two,
        Tick(0),
    )
}

pub(crate) const fn input(seat: SeatId, action: InputAction) -> SeatInput {
    SeatInput { seat, action }
}

pub(crate) const fn press(seat: SeatId, direction: Direction) -> SeatInput {
    input(seat, InputAction::Press(direction))
}

use std::collections::BTreeSet;

use super::{
    arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain},
    bombs::BombTrigger,
    rules::RulesConfig,
    simulation::{State, StateError},
    types::{CellId, EntityId, Tick},
};

pub(crate) fn open_border_arena(blocks: BTreeSet<CellId>) -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row * ARENA_WIDTH + column)] = Terrain::Wall;
            }
        }
    }
    Arena::new(terrain, blocks, [CellId::new(1, 1), CellId::new(13, 11)]).unwrap()
}

pub(crate) fn add_test_bomb(
    state: &mut State,
    id: EntityId,
    cell: CellId,
    owner: EntityId,
) -> Result<(), StateError> {
    let rules = RulesConfig::classic_v1();
    add_test_bomb_with_deadline(
        state,
        id,
        cell,
        owner,
        rules.bomb_range(),
        Tick(0),
        Tick(rules.bomb_fuse_ticks()),
    )
}

pub(crate) fn add_test_bomb_with_deadline(
    state: &mut State,
    id: EntityId,
    cell: CellId,
    owner: EntityId,
    range: u16,
    placed_at: Tick,
    detonates_at: Tick,
) -> Result<(), StateError> {
    add_test_bomb_with_trigger(
        state,
        id,
        cell,
        owner,
        range,
        placed_at,
        BombTrigger::Timed { detonates_at },
    )
}

pub(crate) fn add_test_bomb_with_trigger(
    state: &mut State,
    id: EntityId,
    cell: CellId,
    owner: EntityId,
    range: u16,
    placed_at: Tick,
    trigger: BombTrigger,
) -> Result<(), StateError> {
    state.insert_test_bomb(id, cell, owner, range, placed_at, trigger)
}

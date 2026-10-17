use super::{
    bombs::BombTrigger,
    rules::RulesConfig,
    simulation::{State, StateError},
    types::{CellId, EntityId, Tick},
};

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

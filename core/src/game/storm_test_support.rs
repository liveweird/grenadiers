use std::collections::BTreeSet;

use super::{ActorInit, State};
use crate::game::{
    Arena, CellId, Direction, EnabledItems, EntityId, PlayerCount, Position, RulesConfig,
    SimulationSeed, SpeedLevel, StormCloudConfig, Terrain, Tick, World,
};
use crate::room::{BoardSize, RoomSettings};

pub(super) fn arena(board: BoardSize, count: PlayerCount) -> Arena {
    arena_with_blocks(board, count, BTreeSet::new())
}

pub(super) fn arena_with_blocks(
    board: BoardSize,
    count: PlayerCount,
    blocks: BTreeSet<CellId>,
) -> Arena {
    let mut terrain = vec![Terrain::Floor; board.cell_count()];
    for row in 0..board.height() {
        for column in 0..board.width() {
            if row == 0 || column == 0 || row == board.height() - 1 || column == board.width() - 1 {
                terrain[usize::from(row) * usize::from(board.width()) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    let corners = [
        CellId::new(1, 1),
        CellId::new(board.width() - 2, 1),
        CellId::new(1, board.height() - 2),
        CellId::new(board.width() - 2, board.height() - 2),
    ];
    let spawns = if count == PlayerCount::Two {
        vec![corners[0], corners[3]]
    } else {
        corners[..count.get()].to_vec()
    };
    Arena::new_for_board(board, terrain, blocks, spawns).unwrap()
}

pub(super) fn actors(board: BoardSize, count: PlayerCount) -> Vec<ActorInit> {
    let cells = [
        CellId::new(1, 1),
        CellId::new(board.width() - 2, 1),
        CellId::new(1, board.height() - 2),
        CellId::new(board.width() - 2, board.height() - 2),
    ];
    let selected = if count == PlayerCount::Two {
        vec![cells[0], cells[3]]
    } else {
        cells[..count.get()].to_vec()
    };
    count
        .seats()
        .iter()
        .zip(selected)
        .map(|(&seat, position)| ActorInit {
            id: EntityId(10 + seat.index() as u32),
            seat,
            position: Position::center(position),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        })
        .collect()
}

pub(super) fn rules(items: EnabledItems, cloud: StormCloudConfig) -> RulesConfig {
    RulesConfig::environment_v7(RoomSettings::DEFAULT.with_enabled_items(items), cloud)
}

pub(super) fn state_with(
    board: BoardSize,
    count: PlayerCount,
    items: EnabledItems,
    seed: [u8; 32],
) -> State {
    State::new_at_tick_with_assignment_and_seed(
        rules(items, StormCloudConfig::Single),
        World::new(arena(board, count)),
        actors(board, count),
        Tick(0),
        crate::game::RoundAssignment::first(),
        SimulationSeed::new(seed),
    )
    .unwrap()
}

pub(super) fn state(seed: [u8; 32]) -> State {
    state_with(
        BoardSize::FifteenByThirteen,
        PlayerCount::Two,
        EnabledItems::EMPTY,
        seed,
    )
}

use std::collections::{BTreeSet, VecDeque};
use std::num::NonZeroU32;

use crate::game::{
    CellId, EnabledItems, PlayerCount, RoundAssignment, generate_lair_clearance_multiplayer_arena,
    lair_layout_v1,
};
use crate::room::{BoardSize, BombCapacityDropMultiplier};

use super::{
    ArenaGenerationConfig, WEIGHTED_LAIR_CLEARANCE_GENERATOR_VERSION,
    generate_weighted_lair_clearance_multiplayer_arena, generate_weighted_multiplayer_arena,
};

fn config(board: BoardSize, count: PlayerCount, round: u32) -> ArenaGenerationConfig {
    ArenaGenerationConfig {
        player_count: count,
        board,
        assignment: RoundAssignment::new(NonZeroU32::new(round).unwrap()),
    }
}

#[test]
fn v7_matrix_preserves_weighted_layers_without_lairs_and_v5_at_multiplier_one() {
    for board in boards() {
        for count in [PlayerCount::Two, PlayerCount::Three, PlayerCount::Four] {
            for round in 1..=4 {
                for enabled in masks() {
                    for value in [1, 5, 10] {
                        let multiplier = BombCapacityDropMultiplier::new(value).unwrap();
                        let seed = [board.width() as u8 + count.get() as u8 + round as u8; 32];
                        let config = config(board, count, round);
                        let v6 =
                            generate_weighted_multiplayer_arena(seed, enabled, config, multiplier)
                                .unwrap();
                        let disabled = generate_weighted_lair_clearance_multiplayer_arena(
                            seed, enabled, config, multiplier, false,
                        )
                        .unwrap();
                        assert_physical_parity(&disabled, &v6);
                        if value == 1 {
                            let v5 = generate_lair_clearance_multiplayer_arena(
                                seed, enabled, config, true,
                            )
                            .unwrap();
                            let v7 = generate_weighted_lair_clearance_multiplayer_arena(
                                seed, enabled, config, multiplier, true,
                            )
                            .unwrap();
                            assert_physical_parity(&v7, &v5);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn v7_matrix_clears_only_blocks_and_loot_needed_for_connected_reflected_lairs() {
    for board in boards() {
        for count in [PlayerCount::Two, PlayerCount::Three, PlayerCount::Four] {
            for enabled in masks() {
                for value in [1, 5, 10] {
                    let multiplier = BombCapacityDropMultiplier::new(value).unwrap();
                    let config = config(board, count, 3);
                    let seed = [17 + count.get() as u8 + value as u8; 32];
                    let base =
                        generate_weighted_multiplayer_arena(seed, enabled, config, multiplier)
                            .unwrap();
                    let generated = generate_weighted_lair_clearance_multiplayer_arena(
                        seed, enabled, config, multiplier, true,
                    )
                    .unwrap();
                    assert_eq!(
                        generated.record().version(),
                        WEIGHTED_LAIR_CLEARANCE_GENERATOR_VERSION
                    );
                    assert_eq!(generated.record().seed(), base.record().seed());
                    assert_eq!(
                        generated.record().fallback_used(),
                        base.record().fallback_used()
                    );
                    assert_eq!(generated.arena().spawns(), base.arena().spawns());
                    assert!(generated.arena().blocks().is_subset(base.arena().blocks()));
                    for (cell, kind) in base.concealed_pickups() {
                        if generated.arena().blocks().contains(cell) {
                            assert_eq!(generated.concealed_pickups().get(cell), Some(kind));
                        } else {
                            assert!(!generated.concealed_pickups().contains_key(cell));
                        }
                    }
                    assert_reflected_and_connected(&generated);
                }
            }
        }
    }
}

fn assert_physical_parity(left: &super::GeneratedArena, right: &super::GeneratedArena) {
    assert_eq!(left.arena(), right.arena());
    assert_eq!(left.concealed_pickups(), right.concealed_pickups());
    assert_eq!(left.record().seed(), right.record().seed());
    assert_eq!(
        left.record().fallback_used(),
        right.record().fallback_used()
    );
}

fn assert_reflected_and_connected(generated: &super::GeneratedArena) {
    let arena = generated.arena();
    for row in 0..arena.height() {
        for column in 0..arena.width() {
            let cell = CellId::new(column, row);
            for reflected in reflections(arena, cell) {
                assert_eq!(
                    arena.blocks().contains(&cell),
                    arena.blocks().contains(&reflected)
                );
                assert_eq!(
                    generated.concealed_pickups().get(&cell),
                    generated.concealed_pickups().get(&reflected)
                );
            }
        }
    }
    let lairs = lair_layout_v1(arena.board());
    let required = corner_openings(arena)
        .into_iter()
        .chain(lairs.iter().copied().flat_map(|cell| cardinal(arena, cell)))
        .collect::<Vec<_>>();
    let mut reached = BTreeSet::from([required[0]]);
    let mut queue = VecDeque::from([required[0]]);
    while let Some(cell) = queue.pop_front() {
        for neighbor in cardinal(arena, cell) {
            if !arena.blocks().contains(&neighbor) && reached.insert(neighbor) {
                queue.push_back(neighbor);
            }
        }
    }
    assert!(required.iter().all(|cell| reached.contains(cell)));
    let ring = crate::game::layout_clearance::lair_floor_ring(arena, lairs);
    assert!(ring.iter().all(|cell| !arena.blocks().contains(cell)));
    assert!(
        generated
            .concealed_pickups()
            .keys()
            .all(|cell| arena.blocks().contains(cell))
    );
}

fn cardinal(arena: &crate::game::Arena, cell: CellId) -> Vec<CellId> {
    crate::game::layout_clearance::floor_neighbors(arena, cell)
}

fn corner_openings(arena: &crate::game::Arena) -> Vec<CellId> {
    let right = arena.width() - 2;
    let bottom = arena.height() - 2;
    vec![
        CellId::new(2, 1),
        CellId::new(1, 2),
        CellId::new(right - 1, 1),
        CellId::new(right, 2),
        CellId::new(2, bottom),
        CellId::new(1, bottom - 1),
        CellId::new(right - 1, bottom),
        CellId::new(right, bottom - 1),
    ]
}

fn reflections(arena: &crate::game::Arena, cell: CellId) -> BTreeSet<CellId> {
    BTreeSet::from([
        cell,
        CellId::new(arena.width() - 1 - cell.column, cell.row),
        CellId::new(cell.column, arena.height() - 1 - cell.row),
        CellId::new(
            arena.width() - 1 - cell.column,
            arena.height() - 1 - cell.row,
        ),
    ])
}

fn boards() -> [BoardSize; 3] {
    [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ]
}

fn masks() -> [EnabledItems; 3] {
    [
        EnabledItems::EMPTY,
        EnabledItems::OIL_BARREL,
        EnabledItems::ALL,
    ]
}

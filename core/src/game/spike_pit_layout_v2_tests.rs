use std::collections::{BTreeSet, VecDeque};
use std::num::NonZeroU32;

use crate::game::{
    Arena, ArenaConstructionProfile, ArenaGenerationConfig, CellId, EnabledItems, PlayerCount,
    RoundAssignment, Terrain, generate_profiled_multiplayer_arena, lair_layout_v1,
};
use crate::room::BoardSize;

use super::{spike_pit_layout_v1, spike_pit_layout_v2};

fn config(board: BoardSize, round: u32) -> ArenaGenerationConfig {
    ArenaGenerationConfig {
        player_count: PlayerCount::Four,
        board,
        assignment: RoundAssignment::new(NonZeroU32::new(round).unwrap()),
    }
}

#[test]
fn layout_v2_preserves_reflections_rings_and_the_complete_lair_network() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        for round in 1..=4 {
            for seed_byte in [0_u8, 7, 31, 97] {
                let generated = generate_profiled_multiplayer_arena(
                    [seed_byte; 32],
                    EnabledItems::ALL,
                    config(board, round),
                    ArenaConstructionProfile::LairClearanceV5,
                    true,
                )
                .unwrap();
                let arena = generated.arena();
                let lairs = lair_layout_v1(board);
                let sites = spike_pit_layout_v2(arena, [], lairs);
                assert!(matches!(sites.len(), 0 | 2 | 4));
                assert_eq!(
                    sites.iter().copied().collect::<BTreeSet<_>>(),
                    reflect(arena, &sites)
                );
                let ring = lair_ring(arena, &lairs);
                assert!(sites.iter().all(|site| !ring.contains(site)));
                assert_network(arena, &lairs, &sites);
            }
        }
    }
}

#[test]
fn unavailable_orbits_fall_back_to_zero_without_changing_legacy_layout() {
    let generated = generate_profiled_multiplayer_arena(
        [41; 32],
        EnabledItems::EMPTY,
        config(BoardSize::FifteenByThirteen, 1),
        ArenaConstructionProfile::LairClearanceV5,
        true,
    )
    .unwrap();
    let arena = generated.arena();
    let lairs = lair_layout_v1(arena.board());
    let legacy = spike_pit_layout_v1(arena, lairs);
    let excluded = floor_cells(arena);

    assert!(!legacy.is_empty());
    assert!(spike_pit_layout_v2(arena, excluded.clone(), lairs).is_empty());
    assert!(spike_pit_layout_v2(arena, excluded.into_iter().rev(), lairs).is_empty());
}

#[test]
fn no_active_lairs_preserve_the_legacy_ranked_layout() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        let generated = generate_profiled_multiplayer_arena(
            [17; 32],
            EnabledItems::EMPTY,
            config(board, 2),
            ArenaConstructionProfile::LairClearanceV5,
            false,
        )
        .unwrap();
        assert_eq!(
            spike_pit_layout_v2(generated.arena(), [], []),
            spike_pit_layout_v1(generated.arena(), [])
        );
    }
}

#[test]
fn selected_orbit_keeps_every_lair_exit_on_the_corner_network() {
    let board = BoardSize::ElevenByNine;
    let generated = generate_profiled_multiplayer_arena(
        [0; 32],
        EnabledItems::EMPTY,
        config(board, 1),
        ArenaConstructionProfile::LairClearanceV5,
        true,
    )
    .unwrap();
    let arena = generated.arena();
    let lairs = lair_layout_v1(board);
    let sites = spike_pit_layout_v2(arena, [], lairs);

    assert_network(arena, &lairs, &sites);
    for site in sites {
        assert!(!lair_ring(arena, &lairs).contains(&site));
    }
}

fn assert_network(arena: &Arena, lairs: &[CellId], sites: &[CellId]) {
    let blocked = arena
        .blocks()
        .iter()
        .copied()
        .chain(sites.iter().copied())
        .collect::<BTreeSet<_>>();
    let required = corner_openings(arena)
        .into_iter()
        .chain(lairs.iter().copied().flat_map(|cell| cardinal(arena, cell)))
        .collect::<Vec<_>>();
    let reached = flood(arena, required[0], &blocked);
    assert!(required.iter().all(|cell| reached.contains(cell)));
}

fn flood(arena: &Arena, start: CellId, blocked: &BTreeSet<CellId>) -> BTreeSet<CellId> {
    let mut reached = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(cell) = queue.pop_front() {
        for neighbor in cardinal(arena, cell) {
            if !blocked.contains(&neighbor) && reached.insert(neighbor) {
                queue.push_back(neighbor);
            }
        }
    }
    reached
}

fn cardinal(arena: &Arena, cell: CellId) -> Vec<CellId> {
    [(0_i32, -1_i32), (-1, 0), (1, 0), (0, 1)]
        .into_iter()
        .filter_map(|(column_delta, row_delta)| {
            let column = u16::try_from(i32::from(cell.column) + column_delta).ok()?;
            let row = u16::try_from(i32::from(cell.row) + row_delta).ok()?;
            let candidate = CellId::new(column, row);
            (arena.terrain(candidate) == Some(Terrain::Floor)).then_some(candidate)
        })
        .collect()
}

fn corner_openings(arena: &Arena) -> Vec<CellId> {
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

fn lair_ring(arena: &Arena, lairs: &[CellId]) -> BTreeSet<CellId> {
    lairs
        .iter()
        .flat_map(|lair| {
            (-1_i32..=1).flat_map(move |row_delta| {
                (-1_i32..=1).filter_map(move |column_delta| {
                    let column = u16::try_from(i32::from(lair.column) + column_delta).ok()?;
                    let row = u16::try_from(i32::from(lair.row) + row_delta).ok()?;
                    let cell = CellId::new(column, row);
                    (arena.terrain(cell) == Some(Terrain::Floor)).then_some(cell)
                })
            })
        })
        .collect()
}

fn reflect(arena: &Arena, sites: &[CellId]) -> BTreeSet<CellId> {
    sites
        .iter()
        .flat_map(|cell| {
            [
                *cell,
                CellId::new(arena.width() - 1 - cell.column, cell.row),
                CellId::new(cell.column, arena.height() - 1 - cell.row),
                CellId::new(
                    arena.width() - 1 - cell.column,
                    arena.height() - 1 - cell.row,
                ),
            ]
        })
        .collect()
}

fn floor_cells(arena: &Arena) -> Vec<CellId> {
    (1..arena.height() - 1)
        .flat_map(|row| (1..arena.width() - 1).map(move |column| CellId::new(column, row)))
        .filter(|cell| arena.terrain(*cell) == Some(Terrain::Floor))
        .collect()
}

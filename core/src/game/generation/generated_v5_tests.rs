use std::collections::{BTreeSet, VecDeque};
use std::num::NonZeroU32;

use crate::game::{
    Arena, ArenaConstructionProfile, CellId, EnabledItems, MAX_OIL_BARRELS, PickupKind,
    PlayerCount, RoundAssignment, Terrain, generate_profiled_multiplayer_arena, lair_layout_v1,
};
use crate::room::BoardSize;

use super::ArenaGenerationConfig;

fn config(board: BoardSize, player_count: PlayerCount, round: u32) -> ArenaGenerationConfig {
    ArenaGenerationConfig {
        player_count,
        board,
        assignment: RoundAssignment::new(NonZeroU32::new(round).unwrap()),
    }
}

fn base(
    seed: [u8; 32],
    enabled: EnabledItems,
    config: ArenaGenerationConfig,
) -> super::GeneratedArena {
    generate_profiled_multiplayer_arena(
        seed,
        enabled,
        config,
        ArenaConstructionProfile::ReflectedV3V4,
        true,
    )
    .unwrap()
}

#[test]
fn seeded_outputs_match_version_five_goldens_for_each_preset() {
    let cases = [
        (
            BoardSize::ElevenByNine,
            [0; 32],
            EnabledItems::EMPTY,
            "..##.##../.X#X#X#X./........./.X.X.X.X./........./.X#X#X#X./..##.##..",
        ),
        (
            BoardSize::ThirteenByEleven,
            [1; 32],
            EnabledItems::ALL,
            ".....#...../.X.X.X.X.X./.#.I...I.#./OX#X.X.X#XO/##I#...#I##/OX#X.X.X#XO/.#.I...I.#./.X.X.X.X.X./.....#.....",
        ),
        (
            BoardSize::FifteenByThirteen,
            [0; 32],
            EnabledItems::EMPTY,
            "..##.###.##../.XCX.X#X.XCX./...#######.../.X.X.X#X.X.X./.#.........#./.XRX.X.X.XRX./.#.........#./.X.X.X#X.X.X./...#######.../.XCX.X#X.XCX./..##.###.##..",
        ),
    ];
    for (board, seed, enabled, expected) in cases {
        let generated = generate_profiled_multiplayer_arena(
            seed,
            enabled,
            config(board, PlayerCount::Four, 1),
            ArenaConstructionProfile::LairClearanceV5,
            true,
        )
        .unwrap();
        assert_eq!(signature(&generated), expected);
    }
}

#[test]
fn profile_versions_and_disabled_lairs_preserve_every_base_layer() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        assert_eq!(
            ArenaConstructionProfile::ReflectedV3V4.generator_version(board),
            if board == BoardSize::FifteenByThirteen {
                3
            } else {
                4
            }
        );
        assert_eq!(
            ArenaConstructionProfile::LairClearanceV5.generator_version(board),
            5
        );
        for byte in [0_u8, 7, 31, 113, 255] {
            let config = config(board, PlayerCount::Four, 3);
            let old = base([byte; 32], EnabledItems::ALL, config);
            let new = generate_profiled_multiplayer_arena(
                [byte; 32],
                EnabledItems::ALL,
                config,
                ArenaConstructionProfile::LairClearanceV5,
                false,
            )
            .unwrap();
            assert_eq!(new.arena(), old.arena());
            assert_eq!(new.concealed_pickups(), old.concealed_pickups());
            assert_eq!(new.record().seed(), old.record().seed());
            assert_eq!(new.record().fallback_used(), old.record().fallback_used());
            assert_eq!(new.record().version(), 5);
        }
    }
}

#[test]
fn seeded_clearance_properties_cover_boards_counts_rotations_and_oil() {
    let masks = [
        EnabledItems::EMPTY,
        EnabledItems::OIL_BARREL,
        EnabledItems::ALL,
    ];
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        for count in [PlayerCount::Two, PlayerCount::Three, PlayerCount::Four] {
            for round in 1..=4 {
                for (seed_byte, enabled) in masks.into_iter().enumerate() {
                    assert_clearance_case(
                        [u8::try_from(seed_byte + round as usize * 11).unwrap(); 32],
                        enabled,
                        config(board, count, round),
                    );
                }
            }
        }
    }
}

fn assert_clearance_case(seed: [u8; 32], enabled: EnabledItems, config: ArenaGenerationConfig) {
    let old = base(seed, enabled, config);
    let generated = generate_profiled_multiplayer_arena(
        seed,
        enabled,
        config,
        ArenaConstructionProfile::LairClearanceV5,
        true,
    )
    .unwrap();
    let repeated = generate_profiled_multiplayer_arena(
        seed,
        enabled,
        config,
        ArenaConstructionProfile::LairClearanceV5,
        true,
    )
    .unwrap();
    assert_eq!(generated, repeated);
    assert_eq!(generated.record().version(), 5);
    assert_eq!(generated.record().seed(), old.record().seed());
    assert_eq!(
        generated.record().fallback_used(),
        old.record().fallback_used()
    );
    assert_eq!(generated.arena().spawns(), old.arena().spawns());
    assert_same_terrain(generated.arena(), old.arena());
    assert!(generated.arena().blocks().is_subset(old.arena().blocks()));
    let removed = old
        .arena()
        .blocks()
        .difference(generated.arena().blocks())
        .copied()
        .collect::<BTreeSet<_>>();
    for (cell, kind) in old.concealed_pickups() {
        if removed.contains(cell) {
            assert!(!generated.concealed_pickups().contains_key(cell));
        } else {
            assert_eq!(generated.concealed_pickups().get(cell), Some(kind));
        }
    }
    assert!(
        generated
            .concealed_pickups()
            .keys()
            .all(|cell| generated.arena().blocks().contains(cell))
    );
    assert!(
        generated
            .concealed_pickups()
            .values()
            .filter(|kind| **kind == PickupKind::OilBarrel)
            .count()
            <= MAX_OIL_BARRELS
    );
    assert_reflections(generated.arena(), generated.concealed_pickups());
    assert_lair_network(generated.arena());
}

fn assert_same_terrain(left: &Arena, right: &Arena) {
    for row in 0..left.height() {
        for column in 0..left.width() {
            let cell = CellId::new(column, row);
            assert_eq!(left.terrain(cell), right.terrain(cell));
        }
    }
}

fn assert_reflections(arena: &Arena, concealed: &std::collections::BTreeMap<CellId, PickupKind>) {
    for row in 0..arena.height() {
        for column in 0..arena.width() {
            let cell = CellId::new(column, row);
            for reflected in reflections(arena, cell) {
                assert_eq!(
                    arena.blocks().contains(&cell),
                    arena.blocks().contains(&reflected)
                );
                assert_eq!(concealed.get(&cell), concealed.get(&reflected));
            }
        }
    }
}

fn assert_lair_network(arena: &Arena) {
    let lairs = lair_layout_v1(arena.board());
    let exits = lairs.into_iter().flat_map(|cell| cardinal(arena, cell));
    let required = corner_openings(arena)
        .into_iter()
        .chain(exits)
        .collect::<Vec<_>>();
    let reached = flood(arena, required[0]);
    assert!(required.iter().all(|cell| reached.contains(cell)));
    for lair in lairs {
        let ring = (-1_i32..=1).flat_map(|row_delta| {
            (-1_i32..=1).filter_map(move |column_delta| {
                let column = u16::try_from(i32::from(lair.column) + column_delta).ok()?;
                let row = u16::try_from(i32::from(lair.row) + row_delta).ok()?;
                Some(CellId::new(column, row))
            })
        });
        assert!(
            ring.filter(|cell| arena.terrain(*cell) == Some(Terrain::Floor))
                .all(|cell| { !arena.blocks().contains(&cell) })
        );
    }
}

fn flood(arena: &Arena, start: CellId) -> BTreeSet<CellId> {
    let mut reached = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(cell) = queue.pop_front() {
        for neighbor in cardinal(arena, cell) {
            if !arena.blocks().contains(&neighbor) && reached.insert(neighbor) {
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
    [
        CellId::new(2, 1),
        CellId::new(1, 2),
        CellId::new(right - 1, 1),
        CellId::new(right, 2),
        CellId::new(2, bottom),
        CellId::new(1, bottom - 1),
        CellId::new(right - 1, bottom),
        CellId::new(right, bottom - 1),
    ]
    .to_vec()
}

fn reflections(arena: &Arena, cell: CellId) -> BTreeSet<CellId> {
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

fn signature(generated: &super::GeneratedArena) -> String {
    let arena = generated.arena();
    let mut rows = Vec::new();
    for row in 1..arena.height() - 1 {
        let mut encoded = String::new();
        for column in 1..arena.width() - 1 {
            let cell = CellId::new(column, row);
            encoded.push(match generated.concealed_pickups().get(&cell) {
                Some(PickupKind::BombCapacity) => 'C',
                Some(PickupKind::BlastRange) => 'R',
                Some(PickupKind::MovementSpeed) => 'S',
                Some(PickupKind::OilBarrel) => 'O',
                Some(_) => 'I',
                None if arena.blocks().contains(&cell) => '#',
                None if arena.terrain(cell) == Some(Terrain::Wall) => 'X',
                None => '.',
            });
        }
        rows.push(encoded);
    }
    rows.join("/")
}

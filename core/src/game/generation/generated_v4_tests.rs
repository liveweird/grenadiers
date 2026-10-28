use std::collections::{BTreeSet, VecDeque};
use std::num::NonZeroU32;

use super::*;
use crate::game::RoundAssignment;

struct Sequence {
    words: VecDeque<u32>,
    default: Option<u32>,
}

impl RandomWords for Sequence {
    fn next_u32(&mut self) -> Option<u32> {
        self.words.pop_front().or(self.default)
    }
}

fn config(board: BoardSize, player_count: PlayerCount, round: u32) -> ArenaGenerationConfig {
    ArenaGenerationConfig {
        player_count,
        board,
        assignment: RoundAssignment::new(NonZeroU32::new(round).unwrap()),
    }
}

#[test]
fn all_presets_and_participant_counts_generate_checked_arenas() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        for count in [PlayerCount::Two, PlayerCount::Three, PlayerCount::Four] {
            let generated = generate_variable_multiplayer_arena(
                [board.width() as u8 + count.get() as u8; 32],
                EnabledItems::ALL,
                config(board, count, 2),
            )
            .unwrap();
            let arena = generated.arena();
            assert_eq!(arena.board(), board);
            assert_eq!(arena.cell_count(), board.cell_count());
            assert_eq!(arena.spawns().len(), count.get());
            assert_eq!(generated.record().version(), VARIABLE_GENERATOR_VERSION);
            assert!(!generated.record().fallback_used());
            assert!(arena.spawns().iter().all(|spawn| {
                interior_corners(board).contains(spawn)
                    && arena.terrain(*spawn) == Some(Terrain::Floor)
                    && !arena.blocks().contains(spawn)
            }));
        }
    }
}

#[test]
fn seeded_generation_is_reproducible_and_records_the_raw_seed() {
    let seed = [31; 32];
    let generation_config = config(BoardSize::ThirteenByEleven, PlayerCount::Three, 7);
    let first =
        generate_variable_multiplayer_arena(seed, EnabledItems::ALL, generation_config).unwrap();
    let second =
        generate_variable_multiplayer_arena(seed, EnabledItems::ALL, generation_config).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.record().seed(), seed);
    assert_eq!(first.record().version(), VARIABLE_GENERATOR_VERSION);
    assert!(!first.record().fallback_used());
}

#[test]
fn seeded_outputs_match_version_four_goldens_for_each_preset() {
    let eleven = generate_variable_multiplayer_arena(
        [0; 32],
        EnabledItems::EMPTY,
        config(BoardSize::ElevenByNine, PlayerCount::Two, 1),
    )
    .unwrap();
    let thirteen = generate_variable_multiplayer_arena(
        [1; 32],
        EnabledItems::ALL,
        config(BoardSize::ThirteenByEleven, PlayerCount::Three, 2),
    )
    .unwrap();
    let fifteen = generate_variable_multiplayer_arena(
        [2; 32],
        EnabledItems::OIL_BARREL,
        config(BoardSize::FifteenByThirteen, PlayerCount::Four, 3),
    )
    .unwrap();

    assert_eq!(
        signature(&eleven),
        "..##.##../.X#X#X#X./C.#.#.#.C/.X#X#X#X./C.#.#.#.C/.X#X#X#X./..##.##.."
    );
    assert_eq!(
        signature(&thirteen),
        "..#..#..#../.X.X#X#X.X./.#.II#II.#./OX#X.X.X#XO/##I##I##I##/OX#X.X.X#XO/.#.II#II.#./.X.X#X#X.X./..#..#..#.."
    );
    assert_eq!(
        signature(&fifteen),
        "..#R#####R#../.X#X#X.X#X#X./####S...S####/.X#X.X#X.X#X./O###.#.#.###O/#X#X.XSX.X#X#/O###.#.#.###O/.X#X.X#X.X#X./####S...S####/.X#X#X.X#X#X./..#R#####R#.."
    );
}

#[test]
fn three_players_omit_each_corner_in_order_on_every_preset() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        let corners = interior_corners(board);
        for round in 1..=4 {
            let generated = generate_variable_multiplayer_arena(
                [round as u8; 32],
                EnabledItems::EMPTY,
                config(board, PlayerCount::Three, round),
            )
            .unwrap();
            let omitted = ((round - 1) as usize + 3) % 4;
            let expected = corners
                .into_iter()
                .enumerate()
                .filter_map(|(index, corner)| (index != omitted).then_some(corner))
                .collect::<Vec<_>>();
            assert_eq!(generated.arena().spawns(), expected);
        }
    }
}

#[test]
fn four_player_assignment_rotates_seats_across_every_corner() {
    let board = BoardSize::ElevenByNine;
    let corners = interior_corners(board);
    for round in 1..=4 {
        let generation_config = config(board, PlayerCount::Four, round);
        let generated = generate_variable_multiplayer_arena(
            [round as u8; 32],
            EnabledItems::EMPTY,
            generation_config,
        )
        .unwrap();
        for seat in PlayerCount::Four.seats() {
            let expected = corners[(seat.index() + (round - 1) as usize) % 4];
            assert_eq!(
                generation_config
                    .assignment
                    .spawn_for(generated.arena(), *seat),
                Some(expected)
            );
        }
    }
}

#[test]
fn generated_board_has_borders_pillars_reserved_floors_and_reflections() {
    let board = BoardSize::ElevenByNine;
    let generated = generate_variable_multiplayer_arena(
        [17; 32],
        EnabledItems::ALL,
        config(board, PlayerCount::Four, 1),
    )
    .unwrap();
    let arena = generated.arena();

    for row in 0..board.height() {
        for column in 0..board.width() {
            let cell = CellId::new(column, row);
            let border =
                row == 0 || column == 0 || row == board.height() - 1 || column == board.width() - 1;
            if border || (row % 2 == 0 && column % 2 == 0) {
                assert_eq!(arena.terrain(cell), Some(Terrain::Wall));
            }
            for reflected in reflection_orbit(cell, board) {
                assert_eq!(arena.terrain(cell), arena.terrain(reflected));
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
    for cell in reserved_cells(board) {
        assert_eq!(arena.terrain(cell), Some(Terrain::Floor));
        assert!(!arena.blocks().contains(&cell));
    }
}

#[test]
fn oil_is_added_only_as_whole_orbits_up_to_the_global_cap() {
    let board = BoardSize::ThirteenByEleven;
    let orbit_count = eligible_orbits(board).len();
    let mut words = VecDeque::new();
    for _ in 0..orbit_count {
        words.extend([0, 0, 114]);
    }
    let mut source = Sequence {
        words,
        default: None,
    };
    let generated = generate_with_source(
        [4; 32],
        EnabledItems::OIL_BARREL,
        config(board, PlayerCount::Four, 1),
        &mut source,
    );
    let oil: BTreeSet<_> = generated
        .concealed_pickups()
        .iter()
        .filter_map(|(&cell, &kind)| (kind == PickupKind::OilBarrel).then_some(cell))
        .collect();
    assert_eq!(oil.len(), MAX_OIL_BARRELS);
    for orbit in eligible_orbits(board) {
        let count = orbit.iter().filter(|cell| oil.contains(cell)).count();
        assert!(count == 0 || count == orbit.len());
    }
}

#[test]
fn exhausted_draw_source_uses_dimension_correct_empty_fallback() {
    let board = BoardSize::ElevenByNine;
    let mut source = Sequence {
        words: VecDeque::new(),
        default: None,
    };
    let generated = generate_with_source(
        [9; 32],
        EnabledItems::ALL,
        config(board, PlayerCount::Three, 4),
        &mut source,
    );
    assert!(generated.record().fallback_used());
    assert_eq!(generated.arena().board(), board);
    assert!(generated.arena().blocks().is_empty());
    assert!(generated.concealed_pickups().is_empty());
    assert_eq!(generated.arena().spawns().len(), 3);
    assert!(
        generated
            .arena()
            .spawns()
            .contains(&CellId::new(board.width() - 2, board.height() - 2))
    );
}

fn signature(generated: &GeneratedArena) -> String {
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

use std::collections::VecDeque;
use std::num::NonZeroU32;

use super::*;

struct Sequence {
    words: VecDeque<u32>,
    default: Option<u32>,
    draws: usize,
}

impl Sequence {
    fn new(words: impl IntoIterator<Item = u32>, default: Option<u32>) -> Self {
        Self {
            words: words.into_iter().collect(),
            default,
            draws: 0,
        }
    }
}

impl RandomWords for Sequence {
    fn next_u32(&mut self) -> Option<u32> {
        self.draws += 1;
        self.words.pop_front().or(self.default)
    }
}

fn config(count: PlayerCount, round: u32) -> ArenaGenerationConfig {
    ArenaGenerationConfig {
        player_count: count,
        board: BoardSize::FifteenByThirteen,
        assignment: RoundAssignment::new(NonZeroU32::new(round).unwrap()),
    }
}

#[test]
fn seeded_generation_is_reproducible_and_records_version_three() {
    let seed = [31; 32];
    let first =
        generate_multiplayer_arena(seed, EnabledItems::ALL, config(PlayerCount::Four, 1)).unwrap();
    let second =
        generate_multiplayer_arena(seed, EnabledItems::ALL, config(PlayerCount::Four, 1)).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.record().version(), MULTIPLAYER_GENERATOR_VERSION);
    assert_eq!(first.record().seed(), seed);
    assert!(!first.record().fallback_used());
}

#[test]
fn seeded_outputs_match_version_three_goldens() {
    assert_eq!(
        signature(
            &generate_multiplayer_arena([0; 32], EnabledItems::EMPTY, config(PlayerCount::Four, 1))
                .unwrap()
        ),
        "..##.###.##../.XCX.X#X.XCX./.#.#######.#./.X.X.X#X.X.X./.#.##R#R##.#./.XRX#X#X#XRX./.#.##R#R##.#./.X.X.X#X.X.X./.#.#######.#./.XCX.X#X.XCX./..##.###.##.."
    );
    assert_eq!(
        signature(
            &generate_multiplayer_arena([1; 32], EnabledItems::ALL, config(PlayerCount::Three, 2))
                .unwrap()
        ),
        "..#..#.#..#../.X#X.X#X.X#X./.II#O#.#O#II./#X#XIX#XIX#X#/#I.#.###.#.I#/.X.X#XCX#X.X./#I.#.###.#.I#/#X#XIX#XIX#X#/.II#O#.#O#II./.X#X.X#X.X#X./..#..#.#..#.."
    );
}

#[test]
fn generated_layers_have_both_reflections_and_all_four_escape_routes() {
    let generated =
        generate_multiplayer_arena([7; 32], EnabledItems::ALL, config(PlayerCount::Three, 2))
            .unwrap();
    let arena = generated.arena();

    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            let cell = CellId::new(column, row);
            for reflected in reflection_orbit(cell) {
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
    for cell in RESERVED_CELLS {
        assert_eq!(arena.terrain(cell), Some(Terrain::Floor));
        assert!(!arena.blocks().contains(&cell));
        assert!(!generated.concealed_pickups().contains_key(&cell));
    }
}

#[test]
fn reflection_orbits_are_deduplicated_and_ordered_by_row_major_representative() {
    let orbits = eligible_orbits();
    assert!(orbits.iter().all(|orbit| matches!(orbit.len(), 1 | 2 | 4)));
    assert!(orbits.windows(2).all(|pair| pair[0][0] < pair[1][0]));
    let cells: Vec<_> = orbits.iter().flatten().copied().collect();
    let unique: BTreeSet<_> = cells.iter().copied().collect();
    assert_eq!(cells.len(), unique.len());
}

#[test]
fn oil_accepts_only_whole_orbits_within_the_eight_site_cap() {
    let orbit_count = eligible_orbits().len();
    let mut words = Vec::new();
    for _ in 0..orbit_count {
        words.extend([0, 0, 114]);
    }
    let mut source = Sequence::new(words, None);
    let generated = generate_with_source(
        [4; 32],
        EnabledItems::OIL_BARREL,
        config(PlayerCount::Four, 1),
        &mut source,
    );
    let oil_count = generated
        .concealed_pickups()
        .values()
        .filter(|kind| **kind == PickupKind::OilBarrel)
        .count();
    assert_eq!(oil_count, MAX_OIL_BARRELS);
    for orbit in eligible_orbits() {
        let in_orbit = orbit
            .iter()
            .filter(|cell| generated.concealed_pickups().get(cell) == Some(&PickupKind::OilBarrel))
            .count();
        assert!(in_orbit == 0 || in_orbit == orbit.len());
    }
}

#[test]
fn bounded_source_failure_uses_same_round_empty_fallback() {
    let mut source = Sequence::new([u32::MAX; 16], None);
    let generated = generate_with_source(
        [9; 32],
        EnabledItems::ALL,
        config(PlayerCount::Three, 4),
        &mut source,
    );

    assert!(generated.record().fallback_used());
    assert!(generated.arena().blocks().is_empty());
    assert!(generated.concealed_pickups().is_empty());
    assert_eq!(
        generated.arena().spawns(),
        round_spawns(PlayerCount::Three, config(PlayerCount::Three, 4).assignment)
    );
    assert_eq!(source.draws, 16);
}

#[test]
fn smaller_board_sizes_are_rejected_before_generation() {
    for board in [BoardSize::ElevenByNine, BoardSize::ThirteenByEleven] {
        let mut unsupported = config(PlayerCount::Four, 1);
        unsupported.board = board;
        assert_eq!(
            generate_multiplayer_arena([0; 32], EnabledItems::EMPTY, unsupported),
            Err(GenerationError::UnsupportedBoard(board))
        );
    }
}

#[test]
fn three_players_omit_each_corner_over_four_rounds() {
    for round in 1..=4 {
        let generated = generate_multiplayer_arena(
            [round as u8; 32],
            EnabledItems::EMPTY,
            config(PlayerCount::Three, round),
        )
        .unwrap();
        let omitted = ((round - 1) as usize + 3) % 4;
        assert_eq!(generated.arena().spawns().len(), 3);
        assert!(!generated.arena().spawns().contains(&CORNERS[omitted]));
    }
}

#[test]
fn two_players_keep_the_legacy_opposite_corner_pair() {
    let generated =
        generate_multiplayer_arena([2; 32], EnabledItems::EMPTY, config(PlayerCount::Two, 3))
            .unwrap();
    assert_eq!(generated.arena().spawns(), [CORNERS[0], CORNERS[3]]);
}

#[test]
fn three_player_seats_visit_every_corner_three_times_in_twelve_rounds() {
    let mut visits = [[0_u8; 4]; 3];
    for round in 1..=12 {
        let assignment = config(PlayerCount::Three, round).assignment;
        let arena = fallback_arena(round_spawns(PlayerCount::Three, assignment));
        for seat in PlayerCount::Three.seats() {
            let spawn = assignment.spawn_for(&arena, *seat).unwrap();
            let corner = CORNERS
                .iter()
                .position(|candidate| *candidate == spawn)
                .unwrap();
            visits[seat.index()][corner] += 1;
        }
    }
    assert_eq!(visits, [[3; 4]; 3]);
}

#[test]
fn four_player_assignment_rotates_all_seats_over_four_rounds() {
    for round in 1..=4 {
        let assignment = config(PlayerCount::Four, round).assignment;
        let arena = fallback_arena(round_spawns(PlayerCount::Four, assignment));
        for seat in PlayerCount::Four.seats() {
            let expected = CORNERS[(seat.index() + (round - 1) as usize) % 4];
            assert_eq!(assignment.spawn_for(&arena, *seat), Some(expected));
        }
    }
}

fn signature(generated: &GeneratedArena) -> String {
    let mut rows = Vec::new();
    for row in 1..ARENA_HEIGHT - 1 {
        let mut encoded = String::new();
        for column in 1..ARENA_WIDTH - 1 {
            let cell = CellId::new(column, row);
            encoded.push(match generated.concealed_pickups().get(&cell) {
                Some(PickupKind::BombCapacity) => 'C',
                Some(PickupKind::BlastRange) => 'R',
                Some(PickupKind::MovementSpeed) => 'S',
                Some(PickupKind::OilBarrel) => 'O',
                Some(_) => 'I',
                None if generated.arena().blocks().contains(&cell) => '#',
                None if generated.arena().terrain(cell) == Some(Terrain::Wall) => 'X',
                None => '.',
            });
        }
        rows.push(encoded);
    }
    rows.join("/")
}

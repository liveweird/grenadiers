use std::collections::VecDeque;
use std::num::NonZeroU32;

use super::*;
use crate::game::{PlayerCount, RoundAssignment};
use crate::room::{BoardSize, BombCapacityDropMultiplier};

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

fn config(board: BoardSize, count: PlayerCount, round: u32) -> ArenaGenerationConfig {
    ArenaGenerationConfig {
        player_count: count,
        board,
        assignment: RoundAssignment::new(NonZeroU32::new(round).unwrap()),
    }
}

fn legacy(seed: [u8; 32], enabled: EnabledItems, config: ArenaGenerationConfig) -> GeneratedArena {
    match config.board {
        BoardSize::FifteenByThirteen => generate_multiplayer_arena(seed, enabled, config).unwrap(),
        BoardSize::ElevenByNine | BoardSize::ThirteenByEleven => {
            generate_variable_multiplayer_arena(seed, enabled, config).unwrap()
        }
    }
}

#[test]
fn multiplier_one_preserves_every_historical_layer_and_fallback_fact() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        for count in [PlayerCount::Two, PlayerCount::Three, PlayerCount::Four] {
            for round in 1..=4 {
                for enabled in [
                    EnabledItems::EMPTY,
                    EnabledItems::OIL_BARREL,
                    EnabledItems::ALL,
                ] {
                    let seed = [board.width() as u8 + count.get() as u8 + round as u8; 32];
                    let config = config(board, count, round);
                    let old = legacy(seed, enabled, config);
                    let weighted = generate_weighted_multiplayer_arena(
                        seed,
                        enabled,
                        config,
                        BombCapacityDropMultiplier::LEGACY,
                    )
                    .unwrap();
                    assert_eq!(weighted.arena(), old.arena());
                    assert_eq!(weighted.concealed_pickups(), old.concealed_pickups());
                    assert_eq!(weighted.record().seed(), old.record().seed());
                    assert_eq!(
                        weighted.record().fallback_used(),
                        old.record().fallback_used()
                    );
                    assert_eq!(weighted.record().version(), WEIGHTED_GENERATOR_VERSION);
                }
            }
        }
    }
}

#[test]
fn multiplier_one_preserves_scripted_exhaustion_and_draw_count() {
    let selection =
        PickupSelectionPolicy::WeightedBombCapacityV2(BombCapacityDropMultiplier::LEGACY);
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        let config = config(board, PlayerCount::Four, 1);
        let mut legacy_source = Sequence::new([], None);
        let mut weighted_source = Sequence::new([], None);
        let (legacy, weighted) = match board {
            BoardSize::FifteenByThirteen => (
                generated_v3::generate_with_source(
                    [9; 32],
                    EnabledItems::ALL,
                    config,
                    &mut legacy_source,
                ),
                generated_v3::generate_with_source_and_policy(
                    [9; 32],
                    EnabledItems::ALL,
                    config,
                    &mut weighted_source,
                    selection,
                ),
            ),
            BoardSize::ElevenByNine | BoardSize::ThirteenByEleven => (
                generated_v4::generate_with_source(
                    [9; 32],
                    EnabledItems::ALL,
                    config,
                    &mut legacy_source,
                ),
                generated_v4::generate_with_source_and_policy(
                    [9; 32],
                    EnabledItems::ALL,
                    config,
                    &mut weighted_source,
                    selection,
                ),
            ),
        };
        assert_eq!(weighted, legacy);
        assert!(weighted.record().fallback_used());
        assert_eq!(weighted_source.draws, legacy_source.draws);
    }
}

#[test]
fn weighted_generation_is_repeatable_reflected_and_checked_on_every_board() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        for multiplier in [1, 5, 10] {
            let config = config(board, PlayerCount::Four, multiplier as u32 % 4 + 1);
            let tuning = BombCapacityDropMultiplier::new(multiplier).unwrap();
            let first = generate_weighted_multiplayer_arena(
                [multiplier as u8; 32],
                EnabledItems::ALL,
                config,
                tuning,
            )
            .unwrap();
            let second = generate_weighted_multiplayer_arena(
                [multiplier as u8; 32],
                EnabledItems::ALL,
                config,
                tuning,
            )
            .unwrap();
            assert_eq!(first, second);
            assert_eq!(first.arena().spawns().len(), 4);
            assert!(
                first
                    .concealed_pickups()
                    .keys()
                    .all(|cell| first.arena().blocks().contains(cell))
            );
            assert!(
                first
                    .concealed_pickups()
                    .values()
                    .filter(|kind| **kind == PickupKind::OilBarrel)
                    .count()
                    <= crate::game::MAX_OIL_BARRELS
            );
            assert_reflected(&first);
        }
    }
}

#[test]
fn layer_trials_keep_sixty_five_and_thirty_percent_boundaries() {
    let selection =
        PickupSelectionPolicy::WeightedBombCapacityV2(BombCapacityDropMultiplier::DEFAULT);
    let first_orbit = generated_v3::eligible_orbits()[0].clone();

    let mut concealed = Sequence::new([64, 29, 0], Some(65));
    let (blocks, pickups) =
        generated_v3::generate_layers_with_policy(&mut concealed, EnabledItems::EMPTY, selection)
            .unwrap();
    assert!(first_orbit.iter().all(|cell| blocks.contains(cell)));
    assert!(
        first_orbit
            .iter()
            .all(|cell| pickups.get(cell) == Some(&PickupKind::BombCapacity))
    );

    let mut not_concealed = Sequence::new([64, 30], Some(65));
    let (blocks, pickups) = generated_v3::generate_layers_with_policy(
        &mut not_concealed,
        EnabledItems::EMPTY,
        selection,
    )
    .unwrap();
    assert!(first_orbit.iter().all(|cell| blocks.contains(cell)));
    assert!(pickups.is_empty());

    let mut no_block = Sequence::new([65], Some(65));
    let (blocks, pickups) =
        generated_v3::generate_layers_with_policy(&mut no_block, EnabledItems::EMPTY, selection)
            .unwrap();
    assert!(blocks.is_empty());
    assert!(pickups.is_empty());
}

#[test]
fn capped_oil_is_not_rerolled_and_consumes_no_extra_draw() {
    let selection =
        PickupSelectionPolicy::WeightedBombCapacityV2(BombCapacityDropMultiplier::DEFAULT);
    let orbit_count = generated_v3::eligible_orbits().len();
    let mut source = Sequence::new([0, 0, 274, 0, 0, 274, 0, 0, 274], Some(65));
    let (_, pickups) =
        generated_v3::generate_layers_with_policy(&mut source, EnabledItems::OIL_BARREL, selection)
            .unwrap();
    assert_eq!(
        pickups
            .values()
            .filter(|kind| **kind == PickupKind::OilBarrel)
            .count(),
        crate::game::MAX_OIL_BARRELS
    );
    assert!(pickups.values().all(|kind| *kind == PickupKind::OilBarrel));
    assert_eq!(source.draws, 9 + orbit_count - 3);
}

fn assert_reflected(generated: &GeneratedArena) {
    let arena = generated.arena();
    for (&cell, kind) in generated.concealed_pickups() {
        for reflected in [
            CellId::new(arena.width() - 1 - cell.column, cell.row),
            CellId::new(cell.column, arena.height() - 1 - cell.row),
        ] {
            assert_eq!(generated.concealed_pickups().get(&reflected), Some(kind));
        }
    }
}

#[path = "foreign_bomb_current_support.rs"]
mod foreign_bomb_current_support;
#[path = "foreign_bomb_current_tests.rs"]
mod foreign_bomb_current_tests;
#[path = "hard_episode_outcome.rs"]
mod outcome;
#[path = "hard65_matrix_tests.rs"]
mod reactive_matrix;
#[path = "hard_episode_reporting.rs"]
mod reporting;
#[path = "hard_episode_support.rs"]
mod support;

use grenadiers_core::{
    game::{ArenaConstructionProfile, SpikePitLayoutPolicy},
    room::{BoardSize, BombCapacityDropMultiplier, CpuDifficulty, CpuProfile},
};

use reporting::assert_operational;
use support::{Case, current_profile, run, run_clearance};

#[test]
fn current_v21_weighted_profile_matrix_is_labeled_and_productive() {
    for (label, difficulty, profile, seed) in [
        (
            "current-easy-v60",
            CpuDifficulty::Easy,
            CpuProfile::CONTACT_PUNCH_V60_EASY,
            71,
        ),
        (
            "current-standard-v60",
            CpuDifficulty::Standard,
            CpuProfile::CONTACT_PUNCH_V60_STANDARD,
            72,
        ),
        (
            "current-hard-v61",
            CpuDifficulty::Hard,
            CpuProfile::HARD_V61,
            73,
        ),
    ] {
        assert_eq!(current_profile(difficulty), profile);
        let outcome = run(
            Case {
                label,
                difficulty,
                board: BoardSize::FifteenByThirteen,
                lairs: true,
                seed,
                native: false,
            },
            840,
        );
        assert_eq!(outcome.profile, profile);
        assert_operational(&outcome);
    }
}

#[test]
fn pumped_hard_v61_is_deterministic_and_productive_on_every_board_and_lair_mode() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        for lairs in [false, true] {
            let case = Case {
                label: "pumped-hard-v61",
                difficulty: CpuDifficulty::Hard,
                board,
                lairs,
                seed: 73,
                native: false,
            };
            let first = run(case, 840);
            let second = run(case, 840);
            assert_eq!(first, second, "board={board:?} lairs={lairs}");
            assert_eq!(first.profile, CpuProfile::HARD_V61);
            assert_operational(&first);
        }
    }
}

#[test]
fn native_hard_v61_seed73_progresses_with_two_workers_at_production_pacing() {
    for lairs in [false, true] {
        let outcome = run(
            Case {
                label: "native-hard-v61-seed73",
                difficulty: CpuDifficulty::Hard,
                board: BoardSize::FifteenByThirteen,
                lairs,
                seed: 73,
                native: true,
            },
            840,
        );
        assert_eq!(outcome.profile, CpuProfile::HARD_V61);
        assert_operational(&outcome);
    }
}

#[test]
fn queued_hard_v64_on_weighted_clearance_gen7_multiplier5_has_real_progress() {
    let outcome = run_clearance(
        Case {
            label: "queued-hard-v64-gen7-mult5",
            difficulty: CpuDifficulty::Hard,
            board: BoardSize::ThirteenByEleven,
            lairs: true,
            seed: 73,
            native: false,
        },
        840,
    );
    assert_eq!(outcome.profile, CpuProfile::HARD_V64);
    assert_clearance_identity(&outcome);
    assert_eq!(outcome.event_placements, outcome.owned_bombs);
    assert!(outcome.destroyed_blocks > 0, "{outcome:?}");
    assert!(outcome.flames_seen, "{outcome:?}");
    assert!(outcome.mob_births > 0, "{outcome:?}");
    assert!(outcome.mob_moved, "{outcome:?}");
    assert!(
        outcome.first_bombs.iter().flatten().any(|bomb| {
            bomb.detonated_at.is_some_and(|tick| tick > bomb.placed_at)
                && bomb
                    .finite_flames_clear_at
                    .zip(bomb.detonated_at)
                    .is_some_and(|(clear, detonation)| clear > detonation)
                && bomb.owner_alive_at_clear == Some(true)
        }),
        "{outcome:?}"
    );
    assert_operational(&outcome);
}

#[test]
fn pumped_gen7_multiplier5_profiles_progress_on_every_board_and_lair_mode() {
    for (difficulty, profile, seed) in [
        (CpuDifficulty::Easy, CpuProfile::RECOVERY_V62_EASY, 71),
        (
            CpuDifficulty::Standard,
            CpuProfile::RECOVERY_V62_STANDARD,
            72,
        ),
        (CpuDifficulty::Hard, CpuProfile::HARD_V64, 73),
    ] {
        for board in [
            BoardSize::ElevenByNine,
            BoardSize::ThirteenByEleven,
            BoardSize::FifteenByThirteen,
        ] {
            for lairs in [false, true] {
                let outcome = run_clearance(
                    Case {
                        label: "pumped-gen7-mult5-matrix",
                        difficulty,
                        board,
                        lairs,
                        seed,
                        native: false,
                    },
                    840,
                );
                assert_eq!(outcome.profile, profile, "board={board:?} lairs={lairs}");
                assert_clearance_identity(&outcome);
                assert_eq!(outcome.event_placements, outcome.owned_bombs);
                assert_operational(&outcome);
            }
        }
    }
}

#[test]
#[ignore = "run in isolation because production worker pacing is timing-sensitive"]
fn native_hard_v64_gen7_multiplier5_progresses_at_production_pacing() {
    for lairs in [false, true] {
        let outcome = run_clearance(
            Case {
                label: "native-hard-v64-gen7-mult5",
                difficulty: CpuDifficulty::Hard,
                board: BoardSize::FifteenByThirteen,
                lairs,
                seed: 73,
                native: true,
            },
            840,
        );
        assert_eq!(outcome.profile, CpuProfile::HARD_V64);
        assert_clearance_identity(&outcome);
        assert_eq!(outcome.event_placements, outcome.owned_bombs);
        assert_operational(&outcome);
    }
}

#[test]
#[ignore = "run in isolation because production worker pacing is timing-sensitive"]
fn native_hard_v65_gen7_multiplier5_progresses_at_production_pacing() {
    for lairs in [false, true] {
        let outcome = support::run_inner(
            Case {
                label: "native-hard-v65-gen7-mult5",
                difficulty: CpuDifficulty::Hard,
                board: BoardSize::FifteenByThirteen,
                lairs,
                seed: 73,
                native: true,
            },
            840,
            CpuProfile::HARD_V65,
            true,
            |_| {},
        );
        assert_eq!(outcome.profile, CpuProfile::HARD_V65);
        assert_clearance_identity(&outcome);
        assert_eq!(outcome.event_placements, outcome.owned_bombs);
        assert_operational(&outcome);
    }
}

#[test]
#[ignore = "run in isolation because production worker pacing is timing-sensitive"]
fn native_hard_v66_gen7_multiplier5_progresses_at_production_pacing() {
    for lairs in [false, true] {
        let outcome = support::run_inner(
            Case {
                label: "native-hard-v66-gen7-mult5",
                difficulty: CpuDifficulty::Hard,
                board: BoardSize::FifteenByThirteen,
                lairs,
                seed: 73,
                native: true,
            },
            840,
            CpuProfile::HARD_V66,
            true,
            |_| {},
        );
        assert_eq!(outcome.profile, CpuProfile::HARD_V66);
        assert_clearance_identity(&outcome);
        assert_eq!(outcome.event_placements, outcome.owned_bombs);
        assert_operational(&outcome);
    }
}

#[test]
#[ignore = "run in isolation because production worker pacing is timing-sensitive"]
fn native_hard_v67_gen7_multiplier5_progresses_at_production_pacing() {
    for lairs in [false, true] {
        let outcome = support::run_inner(
            Case {
                label: "native-hard-v67-gen7-mult5",
                difficulty: CpuDifficulty::Hard,
                board: BoardSize::FifteenByThirteen,
                lairs,
                seed: 73,
                native: true,
            },
            840,
            CpuProfile::HARD_V67,
            true,
            |_| {},
        );
        assert_eq!(outcome.profile, CpuProfile::HARD_V67);
        assert_clearance_identity(&outcome);
        assert_eq!(outcome.event_placements, outcome.owned_bombs);
        assert_operational(&outcome);
    }
}

fn assert_clearance_identity(outcome: &outcome::Outcome) {
    assert_eq!(
        outcome.construction_profile,
        ArenaConstructionProfile::WeightedLairClearanceV7
    );
    assert_eq!(outcome.generation_version, 7);
    assert_eq!(outcome.pit_layout, SpikePitLayoutPolicy::LairClearanceV2);
    assert_eq!(outcome.multiplier, BombCapacityDropMultiplier::DEFAULT);
}

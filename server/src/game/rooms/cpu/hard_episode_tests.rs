#[path = "hard_episode_support.rs"]
mod support;

use grenadiers_core::room::{BoardSize, CpuDifficulty, CpuProfile};

use support::{Case, assert_operational, current_profile, run};

#[test]
fn current_v19_profile_matrix_is_labeled_and_productive() {
    for (label, difficulty, profile, seed) in [
        (
            "current-easy-v27",
            CpuDifficulty::Easy,
            CpuProfile::RELIABLE_V27_EASY,
            71,
        ),
        (
            "current-standard-v27",
            CpuDifficulty::Standard,
            CpuProfile::RELIABLE_V27_STANDARD,
            72,
        ),
        (
            "current-hard-v28",
            CpuDifficulty::Hard,
            CpuProfile::HARD_V28,
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
fn pumped_hard_v28_is_deterministic_and_productive_on_every_board_and_lair_mode() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        for lairs in [false, true] {
            let case = Case {
                label: "pumped-hard-v28",
                difficulty: CpuDifficulty::Hard,
                board,
                lairs,
                seed: 73,
                native: false,
            };
            let first = run(case, 840);
            let second = run(case, 840);
            assert_eq!(first, second, "board={board:?} lairs={lairs}");
            assert_eq!(first.profile, CpuProfile::HARD_V28);
            assert_operational(&first);
        }
    }
}

#[test]
fn native_hard_v28_seed73_progresses_with_two_workers_at_production_pacing() {
    for lairs in [false, true] {
        let outcome = run(
            Case {
                label: "native-hard-v28-seed73",
                difficulty: CpuDifficulty::Hard,
                board: BoardSize::FifteenByThirteen,
                lairs,
                seed: 73,
                native: true,
            },
            840,
        );
        assert_eq!(outcome.profile, CpuProfile::HARD_V28);
        assert_operational(&outcome);
    }
}

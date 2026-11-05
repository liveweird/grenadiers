use grenadiers_core::room::{BoardSize, CpuDifficulty, CpuProfile};

use super::{outcome::Outcome, support};

#[test]
fn fixed_gen7_matrix_reports_hard64_through_hard67_cost_and_unchanged_controls() {
    let boards = [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ];
    let mut controls = Vec::new();
    for (difficulty, profile, seed) in [
        (CpuDifficulty::Easy, CpuProfile::RECOVERY_V62_EASY, 71),
        (
            CpuDifficulty::Standard,
            CpuProfile::RECOVERY_V62_STANDARD,
            72,
        ),
    ] {
        for board in boards {
            for lairs in [false, true] {
                let outcome = support::run_clearance(
                    support::Case {
                        label: "hard65-fixed-control",
                        difficulty,
                        board,
                        lairs,
                        seed,
                        native: false,
                    },
                    840,
                );
                assert_eq!(outcome.profile, profile);
                assert_bounded(&outcome);
                controls.push(metrics(board, lairs, &outcome));
            }
        }
    }

    let mut archived = Vec::new();
    let mut reactive = Vec::new();
    let mut pickup = Vec::new();
    let mut combination = Vec::new();
    for board in boards {
        for lairs in [false, true] {
            let case = support::Case {
                label: "hard65-fixed-pair",
                difficulty: CpuDifficulty::Hard,
                board,
                lairs,
                seed: 73,
                native: false,
            };
            let hard64 = support::run_inner(case, 840, CpuProfile::HARD_V64, true, |_| {});
            let hard65 = support::run_inner(case, 840, CpuProfile::HARD_V65, true, |_| {});
            let hard66 = support::run_inner(case, 840, CpuProfile::HARD_V66, true, |_| {});
            let hard67 = support::run_inner(case, 840, CpuProfile::HARD_V67, true, |_| {});
            assert_eq!(hard64.construction_profile, hard65.construction_profile);
            assert_eq!(hard65.construction_profile, hard66.construction_profile);
            assert_eq!(hard66.construction_profile, hard67.construction_profile);
            assert_eq!(hard64.generation_version, hard65.generation_version);
            assert_eq!(hard65.generation_version, hard66.generation_version);
            assert_eq!(hard66.generation_version, hard67.generation_version);
            assert_eq!(hard64.pit_layout, hard65.pit_layout);
            assert_eq!(hard65.pit_layout, hard66.pit_layout);
            assert_eq!(hard66.pit_layout, hard67.pit_layout);
            assert_eq!(hard64.multiplier, hard65.multiplier);
            assert_eq!(hard65.multiplier, hard66.multiplier);
            assert_eq!(hard66.multiplier, hard67.multiplier);
            assert_bounded(&hard64);
            assert_bounded(&hard65);
            assert_bounded(&hard66);
            assert_bounded(&hard67);
            archived.push(metrics(board, lairs, &hard64));
            reactive.push(metrics(board, lairs, &hard65));
            pickup.push(metrics(board, lairs, &hard66));
            combination.push(metrics(board, lairs, &hard67));
        }
    }
    eprintln!(
        "HARD67_FIXED_MATRIX controls={controls:#?} hard64={archived:#?} hard65={reactive:#?} hard66={pickup:#?} hard67={combination:#?}"
    );
}

fn assert_bounded(outcome: &Outcome) {
    assert_eq!(outcome.failed, 0, "{outcome:?}");
    assert!(
        outcome.max_work <= grenadiers_core::cpu::survival::WORK_ALLOWANCE,
        "{outcome:?}"
    );
    assert_eq!(outcome.event_placements, outcome.owned_bombs);
}

fn metrics(board: BoardSize, lairs: bool, outcome: &Outcome) -> serde_json::Value {
    let admission_rejections: u32 = outcome
        .admissions
        .iter()
        .map(|trace| trace.route + trace.pulse + trace.stale_scene + trace.strategic_plant)
        .sum();
    serde_json::json!({
        "board": format!("{board:?}"),
        "lairs": lairs,
        "profile": outcome.profile.policy.version(),
        "plants": outcome.event_placements.iter().sum::<usize>(),
        "destroyedBlocks": outcome.destroyed_blocks,
        "firstPlant": outcome.first_bombs.iter().flatten().map(|bomb| bomb.placed_at.0).min(),
        "fullCohortSurvival": outcome.alive.iter().all(|alive| *alive),
        "everEliminatedSeats": outcome.ever_dead.iter().filter(|dead| **dead).count(),
        "stalledObjectiveSwitches": outcome.stalled_switches.iter().sum::<usize>(),
        "admissionRejections": admission_rejections,
        "completedObjectives": outcome.completed_switches.iter().sum::<usize>(),
        "maxWork": outcome.max_work,
    })
}

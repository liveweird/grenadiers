use grenadiers_core::{
    cpu::{
        WorkBudget,
        survival::{OriginRecoveryChoice, WORK_ALLOWANCE, plan_continuation_lairs_v54},
    },
    game::{CellId, Tick},
    room::{CpuProfile, MatchPhase},
};

use super::range_three_escape_owner_support::{Outcome, run, run_with_profile};

#[test]
fn shared_preparation_selects_recovery_on_the_actual_tick_867_basis() {
    let legacy = run(true, 1_000);
    assert!(legacy.trigger.is_some());
    assert_eq!(legacy.queued_bases.len(), 3);
    for basis in &legacy.queued_bases {
        let mut work = WorkBudget::new(WORK_ALLOWANCE);
        let selected = plan_continuation_lairs_v54(
            &basis.observation,
            &basis.hazards,
            basis.settled_stop,
            &basis.memory,
            basis.origin(),
            &mut work,
        );
        eprintln!(
            "SHARED_RANGE3_BASIS seat={:?} tick={:?} stop={:?} choice={} work={}",
            basis.seat,
            basis.observation.parts().tick,
            basis.settled_stop,
            choice_name(selected.as_ref()),
            work.spent(),
        );
        assert!(matches!(selected, Some(OriginRecoveryChoice::Pulse(_))));
        assert!(work.spent() <= WORK_ALLOWANCE);
    }
}

#[test]
fn shared_preparation_profiles_survive_productive_fresh_episodes() {
    let mut outcomes = Vec::new();
    for profile in [
        CpuProfile::HARD_V55,
        CpuProfile::RECOVERY_V54_STANDARD,
        CpuProfile::RECOVERY_V54_EASY,
    ] {
        for oil in [true, false] {
            let outcome = run_with_profile(oil, 2_000, profile);
            print_outcome(profile, &outcome);
            outcomes.push((profile, oil, outcome));
        }
    }

    for (profile, oil, outcome) in outcomes {
        assert_eq!(outcome.first_deaths, [None; 3]);
        assert_eq!(outcome.final_alive, [true; 3]);
        assert!(outcome.alive_through_cohort);
        assert_eq!(outcome.cohort_cutoff, Tick(2_180));
        assert!(outcome.cohort_resolved);
        assert_eq!(outcome.final_phase, MatchPhase::Running);
        let trigger = outcome
            .trigger
            .as_ref()
            .expect("compatibility cap must preserve the synchronized range-three wave");
        assert_eq!(trigger.placed_at, Tick(862));
        assert_eq!(trigger.detonates_at, Tick(1_042));
        assert_eq!(
            trigger.cells,
            [CellId::new(13, 3), CellId::new(3, 11), CellId::new(11, 11)]
        );
        assert_eq!(trigger.oil.len(), usize::from(oil) * 2);
        assert!(
            outcome
                .flames_clear_after_trigger
                .is_some_and(|clear| clear > Tick(1_072))
        );
        assert!(
            outcome.final_block_count < outcome.initial_blocks,
            "{profile:?} oil={oil} must clear arena blocks"
        );
        assert!(
            [
                grenadiers_core::game::SeatId::Two,
                grenadiers_core::game::SeatId::Three,
                grenadiers_core::game::SeatId::Four
            ]
            .into_iter()
            .all(|seat| outcome.bomb_births.iter().any(|bomb| bomb.seat == seat))
        );
        assert_eq!(outcome.worker.dequeued, outcome.worker.delivered);
        assert_eq!(
            (
                outcome.worker.failed,
                outcome.worker.cancelled,
                outcome.worker.dropped,
            ),
            (0, 0, 0)
        );
        assert!(
            outcome
                .worker
                .work
                .iter()
                .all(|work| *work <= WORK_ALLOWANCE)
        );
        let cohort = outcome
            .bomb_births
            .iter()
            .filter(|bomb| bomb.placed_at <= outcome.cohort_cutoff)
            .collect::<Vec<_>>();
        assert!(!cohort.is_empty());
        assert!(cohort.iter().all(|bomb| {
            bomb.detonates_at
                .is_some_and(|detonation| detonation.0 - bomb.placed_at.0 == 180)
        }));
        let last_detonation = cohort
            .iter()
            .filter_map(|bomb| bomb.detonates_at)
            .max()
            .unwrap();
        assert_eq!(outcome.cohort_deadline, Some(Tick(last_detonation.0 + 31)));
        assert!(outcome.final_tick >= outcome.cohort_deadline.unwrap());
        assert!(outcome.transitions.iter().any(|sample| sample.flags.origin));
        assert!(
            outcome
                .transitions
                .iter()
                .any(|sample| sample.flags.pulse || sample.flags.continuation)
        );
    }
}

fn choice_name(choice: Option<&OriginRecoveryChoice>) -> &'static str {
    match choice {
        Some(OriginRecoveryChoice::Continuation(_)) => "continuation",
        Some(OriginRecoveryChoice::Pulse(_)) => "pulse",
        None => "none",
    }
}

fn print_outcome(profile: CpuProfile, outcome: &Outcome) {
    let recovery = outcome
        .transitions
        .iter()
        .filter(|sample| sample.flags.origin || sample.flags.pulse || sample.flags.continuation)
        .take(9)
        .map(|sample| (sample.seat, sample.tick, sample.position, sample.flags))
        .collect::<Vec<_>>();
    eprintln!(
        "SHARED_RANGE3 profile={profile:?} oil={} trigger={:?} deaths={:?} final={:?} phase={:?} positions={:?} ranges={:?} alive={:?} cohort={:?}->{:?}/resolved={} blocks={}/{} jobs={}/{} bombs={:?} recovery={:?}",
        outcome.with_oil,
        outcome.trigger,
        outcome.first_deaths,
        outcome.final_tick,
        outcome.final_phase,
        outcome.final_positions,
        outcome.final_ranges,
        outcome.final_alive,
        outcome.cohort_cutoff,
        outcome.cohort_deadline,
        outcome.cohort_resolved,
        outcome.final_block_count,
        outcome.initial_blocks,
        outcome.worker.delivered,
        outcome.worker.dequeued,
        outcome.bomb_births,
        recovery,
    );
}

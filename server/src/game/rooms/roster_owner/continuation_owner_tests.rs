use grenadiers_core::cpu::survival::WORK_ALLOWANCE;
use tokio::time::Instant;

use super::continuation_owner_support::{CPUS, driver, run};

#[test]
fn both_current_cpus_continue_after_cross_bomb_withdrawal_and_survive() {
    for lairs in [false, true] {
        let outcome = run(lairs);
        eprintln!(
            "CONTINUATION_OWNER lairs={lairs} births={:?} routes={:?}/{:?} authority={:?} admissions={:?} renewals={:?} commands={:?} displacement={:?} survived={:?} max_work={}",
            outcome.births,
            outcome.saw_original_routes,
            outcome.saw_route_withdrawal,
            outcome.saw_continuation,
            outcome.admission_ticks,
            outcome.renewal_commits,
            outcome.command_commits,
            outcome.displacement_after_admission,
            outcome.survived_flames,
            outcome.max_work
        );
        assert_eq!(outcome.short_fuse_collected, [true, true]);
        assert!(outcome.births.into_iter().all(|birth| birth.is_some()));
        for (placed, detonates) in outcome.births.into_iter().flatten() {
            assert_eq!(
                detonates.0 - placed.0,
                90,
                "bomb did not capture Short Fuse"
            );
        }
        assert_eq!(outcome.saw_original_routes, [true, true]);
        assert_eq!(outcome.saw_route_withdrawal, [true, true]);
        for index in 0..2 {
            let placed = outcome.births[index].unwrap().0;
            let withdrawal = outcome.route_withdrawal_ticks[index].unwrap();
            assert_eq!(withdrawal, grenadiers_core::game::Tick(placed.0 + 2));
            assert!(withdrawal < outcome.admission_ticks[index].unwrap());
        }
        assert_eq!(outcome.saw_continuation, [true, true]);
        assert!(
            outcome
                .admission_ticks
                .into_iter()
                .all(|tick| tick.is_some())
        );
        assert!(outcome.renewal_commits.into_iter().all(|count| count >= 2));
        assert!(outcome.command_commits.into_iter().all(|count| count > 3));
        for index in 0..2 {
            assert_eq!(
                outcome.first_movement_command_ticks[index],
                outcome.admission_ticks[index],
            );
            assert_eq!(
                outcome.first_moves_after_admission[index],
                // The sealed B+6 command is the first nominal movement; the
                // following normal settlement exposes its physical pose.
                outcome.admission_ticks[index].map(|tick| grenadiers_core::game::Tick(tick.0 + 1)),
            );
            assert!(outcome.displacement_after_admission[index] >= 2 * 1_024);
        }
        assert_eq!(outcome.survived_flames, [true, true]);
        assert_eq!(outcome.failed_work, 0);
        assert!(outcome.max_work <= WORK_ALLOWANCE);
    }
}

#[test]
fn publication_sequence_overflow_withdraws_both_authorities_and_fences_inputs() {
    let (mut owner, planner) = driver(false);
    for _ in 0..300 {
        let report = planner.pump_for_test(usize::MAX);
        assert_eq!(report.failed, 0);
        owner.tick(Instant::now()).unwrap();
        if owner
            .cpus
            .iter()
            .all(|cpu| cpu.has_continuation_authority_for_test())
        {
            break;
        }
    }
    assert!(
        owner
            .cpus
            .iter()
            .all(|cpu| cpu.has_continuation_authority_for_test())
    );
    owner.publication_sequence = Some(u64::MAX);
    owner.publish(owner.last.clone());
    assert_eq!(owner.publication_sequence, None);
    assert!(
        owner
            .cpus
            .iter()
            .all(|cpu| !cpu.has_continuation_authority_for_test())
    );
    assert!(
        CPUS.into_iter()
            .all(|seat| owner.cpu_stop_at[seat.index()].is_some())
    );

    owner.tick(Instant::now()).unwrap();
    assert!(
        CPUS.into_iter()
            .all(|seat| owner.cpu_stationary[seat.index()])
    );
    assert!(owner.cpus.iter().all(|cpu| !cpu.has_pending()));
}

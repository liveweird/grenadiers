use grenadiers_core::{
    cpu::{
        CpuRng, MemoryKey, WorkBudget,
        forecast::ObservedHazardMemory,
        policy::UtilityMemory,
        survival::{ExecutionCertificate, SurvivalPlan, plan_blocked_births_v42_with_origin},
    },
    game::{EntityId, SeatId, Tick},
    room::{CpuDifficulty, CpuProfile},
};
use tokio::time::Instant;

use super::blocked_birth_owner_support::{driver, run};
use crate::game::rooms::roster_runtime::RosterAction;

fn blocked_plan(observation: &grenadiers_core::cpu::Observation) -> SurvivalPlan {
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: observation.parts().recipient,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(observation).unwrap();
    let plan = plan_blocked_births_v42_with_origin(
        observation,
        &UtilityMemory::new(key),
        &history,
        observation.parts().tick,
        None,
        &mut CpuRng::from_seed(key, [42; 32]),
        &mut WorkBudget::new(grenadiers_core::cpu::survival::WORK_ALLOWANCE),
    )
    .unwrap()
    .survival;
    assert_eq!(
        plan.execution
            .as_ref()
            .and_then(ExecutionCertificate::contact_plant_requires_blocked_birth_support),
        Some(true)
    );
    plan
}

#[test]
fn current_profiles_queue_blocked_birth_plant_in_active_environment_and_survive() {
    for (profile, difficulty) in [
        (
            CpuProfile::BLOCKED_BIRTHS_V42_STANDARD,
            CpuDifficulty::Standard,
        ),
        (CpuProfile::HARD_V43, CpuDifficulty::Hard),
    ] {
        let outcome = run(profile, difficulty);
        assert!(outcome.planted_at.is_some(), "{profile:?} did not plant");
        assert!(
            outcome.blocked_birth_authority,
            "{profile:?} did not admit blocked-birth authority"
        );
        assert!(outcome.target_destroyed, "{profile:?} retained block (3,5)");
        assert!(outcome.survived, "{profile:?} did not survive the blast");
        assert!(outcome.max_work <= grenadiers_core::cpu::survival::WORK_ALLOWANCE);
    }
}

#[test]
fn archived_contact_profile_cannot_admit_blocked_birth_authority() {
    let outcome = run(
        CpuProfile::CONTACT_TOPOLOGY_V40_STANDARD,
        CpuDifficulty::Standard,
    );
    assert_eq!(outcome.planted_at, None);
    assert!(!outcome.blocked_birth_authority);
    assert!(!outcome.target_destroyed);
}

#[test]
fn archived_contact_profile_rejects_injected_blocked_birth_certificate() {
    let (mut owner, _) = driver(
        CpuProfile::CONTACT_TOPOLOGY_V40_STANDARD,
        CpuDifficulty::Standard,
    );
    for _ in 0..3 {
        owner.tick(Instant::now()).unwrap();
    }
    owner.cpus[0].cancel();
    let basis = owner.cpus[0].latest_observation_for_test().unwrap();
    let plan = blocked_plan(&basis);
    let admission = plan
        .execution
        .as_ref()
        .and_then(ExecutionCertificate::manual_execution_tick)
        .unwrap()
        .0
        - 1;
    owner.cpus[0].inject_plan_result_for_test(
        basis,
        owner.runtime.lifecycle().round_attempt(),
        plan,
    );
    while owner.runtime.lifecycle().tick().0 <= admission {
        owner.tick(Instant::now()).unwrap();
    }
    assert!(!owner.cpus[0].has_committed_route());
    assert_eq!(owner.cpus[0].admission_trace_for_test().invalid_shape, 1);
}

#[test]
fn stale_blocked_birth_worker_cannot_install_authority_or_plant() {
    let (mut owner, planner) = driver(
        CpuProfile::BLOCKED_BIRTHS_V42_STANDARD,
        CpuDifficulty::Standard,
    );
    for _ in 0..8 {
        owner.tick(Instant::now()).unwrap();
        if owner.cpus[0].has_pending() {
            break;
        }
    }
    assert!(owner.cpus[0].has_pending());
    for _ in 0..=grenadiers_core::cpu::survival::MAX_PLAN_AGE_TICKS + 1 {
        owner.tick(Instant::now()).unwrap();
    }
    assert!(!owner.cpus[0].has_pending());
    let report = planner.pump_for_test(usize::MAX);
    assert_eq!((report.delivered, report.cancelled), (0, 1));
    assert!(!owner.cpus[0].has_committed_blocked_birth_contact());
    assert!(
        owner
            .runtime
            .state_for_test()
            .unwrap()
            .world()
            .bombs()
            .is_empty()
    );
}

#[test]
fn blocked_birth_authority_revokes_at_each_observed_mutation_phase() {
    for reports_before_mutation in 0..3 {
        let (mut owner, planner) = driver(
            CpuProfile::BLOCKED_BIRTHS_V42_STANDARD,
            CpuDifficulty::Standard,
        );
        for _ in 0..80 {
            planner.pump_for_test(usize::MAX);
            owner.tick(Instant::now()).unwrap();
            if owner.cpus[0].has_committed_blocked_birth_contact()
                && owner.runtime.pending_report_count_for_test() == 0
            {
                break;
            }
        }
        assert!(owner.cpus[0].has_committed_blocked_birth_contact());
        for _ in 0..reports_before_mutation {
            owner.tick(Instant::now()).unwrap();
        }
        owner
            .runtime
            .action(
                SeatId::One,
                owner.runtime.lifecycle().round_attempt(),
                RosterAction::Bomb(1),
            )
            .unwrap();
        loop {
            owner.tick(Instant::now()).unwrap();
            if owner.runtime.pending_report_count_for_test() == 0 {
                break;
            }
        }
        assert!(!owner.cpus[0].has_committed_route());
        assert!(owner.cpu_stop_at[super::blocked_birth_owner_support::CPU.index()].is_some());
        owner.tick(Instant::now()).unwrap();
        assert!(owner.cpu_stationary[super::blocked_birth_owner_support::CPU.index()]);
    }
}

#[test]
fn blocked_birth_authority_expires_before_its_exclusive_deadline() {
    let (mut owner, planner) = driver(
        CpuProfile::BLOCKED_BIRTHS_V42_STANDARD,
        CpuDifficulty::Standard,
    );
    let exclusive = loop {
        planner.pump_for_test(usize::MAX);
        owner.tick(Instant::now()).unwrap();
        let state = owner.runtime.state_for_test().unwrap();
        if let Some(bomb) = state
            .world()
            .bombs()
            .values()
            .find(|bomb| bomb.owner() == EntityId(2))
        {
            break Tick(bomb.detonates_at().unwrap().0 + state.rules().flame_ticks());
        }
    };
    while owner.runtime.lifecycle().tick().0 + 2 < exclusive.0 {
        owner.tick(Instant::now()).unwrap();
    }
    assert!(owner.cpus[0].has_committed_blocked_birth_contact());
    owner.tick(Instant::now()).unwrap();
    assert_eq!(owner.runtime.lifecycle().tick().0 + 1, exclusive.0);
    assert!(!owner.cpus[0].has_committed_route());
    owner.tick(Instant::now()).unwrap();
    assert_eq!(owner.runtime.lifecycle().tick(), exclusive);
    assert!(
        owner
            .runtime
            .state_for_test()
            .unwrap()
            .actor(SeatId::Two)
            .unwrap()
            .alive()
    );
}

use grenadiers_core::{
    cpu::{
        CpuRng, MemoryKey, Observation, WorkBudget,
        forecast::ObservedHazardMemory,
        policy::{
            StrategicIntent, StrategicObjectiveKind, StrategicPhase, StrategicSwitchReason,
            StrategicTarget, UtilityMemory,
        },
        survival::{ExecutionCertificate, SurvivalPlan, plan_contact_topology_v40_with_origin},
    },
    game::{CellId, Position},
    room::{CpuDifficulty, CpuProfile},
};
use tokio::time::Instant;

use super::contact_topology_owner_support::{CPU, driver_at_witness, run};
use crate::game::rooms::roster_runtime::RosterAction;

fn typed_plan(observation: &Observation) -> SurvivalPlan {
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let tick = observation.parts().tick;
    let memory = UtilityMemory::new(key).with_strategic_intent(StrategicIntent {
        objective: StrategicObjectiveKind::Farm,
        target: StrategicTarget::Block {
            cell: CellId::new(9, 7),
        },
        approach_cell: Some(CellId::new(10, 7)),
        phase: StrategicPhase::Approach,
        last_switch_reason: StrategicSwitchReason::Stalled,
        observation_tick: tick,
        score: 2,
        committed_until: grenadiers_core::game::Tick(tick.0 + 120),
        progress_deadline: grenadiers_core::game::Tick(tick.0 + 180),
        last_distance: 0,
        last_cpu_position: Position::center(CellId::new(10, 7)),
        deferred_until: None,
    });
    let mut history = ObservedHazardMemory::new(key);
    history.observe(observation).unwrap();
    let mut work = WorkBudget::new(grenadiers_core::cpu::survival::WORK_ALLOWANCE);
    let plan = plan_contact_topology_v40_with_origin(
        observation,
        &memory,
        &history,
        tick,
        None,
        &mut CpuRng::from_seed(key, [40; 32]),
        &mut work,
    )
    .unwrap();
    assert!(matches!(
        plan.survival.execution,
        Some(ExecutionCertificate::ContactPlant(_))
    ));
    plan.survival
}

fn stopped_owner(profile: CpuProfile) -> super::driver::Driver {
    let (mut owner, _) = driver_at_witness(profile, profile.difficulty);
    for _ in 0..3 {
        owner.tick(Instant::now()).unwrap();
    }
    owner.cpus[0].cancel();
    owner
}

fn poll_through_manual_admission(owner: &mut super::driver::Driver, plan: &SurvivalPlan) {
    let admission = plan
        .execution
        .as_ref()
        .and_then(ExecutionCertificate::manual_execution_tick)
        .unwrap()
        .0
        - 1;
    while owner.runtime.lifecycle().tick().0 <= admission {
        owner.tick(Instant::now()).unwrap();
    }
}

#[test]
fn current_contact_topology_profiles_use_natural_mob_to_destroy_target_and_survive() {
    for (profile, difficulty) in [
        (
            CpuProfile::CONTACT_TOPOLOGY_V40_STANDARD,
            CpuDifficulty::Standard,
        ),
        (CpuProfile::HARD_V41, CpuDifficulty::Hard),
    ] {
        let outcome = run(profile, difficulty);
        assert_eq!(
            outcome
                .planted_at
                .and_then(|tick| tick.0.checked_sub(outcome.witness_start.0)),
            Some(10),
            "{profile:?} did not place after the normal Stop/worker admission handshake"
        );
        assert!(
            outcome.observed_typed_contact,
            "{profile:?} never installed typed contact authority"
        );
        assert!(outcome.target_destroyed, "{profile:?} retained block (9,7)");
        assert!(
            outcome.survived,
            "{profile:?} did not survive its flame lifetime"
        );
        assert!(outcome.max_work <= grenadiers_core::cpu::survival::WORK_ALLOWANCE);
    }
}

#[test]
fn archived_farm_origin_profile_cannot_use_the_trapped_mob_target() {
    let outcome = run(
        CpuProfile::FARM_ORIGINS_V38_STANDARD,
        CpuDifficulty::Standard,
    );
    assert!(!outcome.observed_typed_contact);
    assert_eq!(outcome.planted_at, None);
    assert!(!outcome.target_destroyed);
}

#[test]
fn archived_profile_rejects_an_injected_contact_plant_before_core_admission() {
    let mut owner = stopped_owner(CpuProfile::FARM_ORIGINS_V38_STANDARD);
    let basis = owner.cpus[0].latest_observation_for_test().unwrap();
    let plan = typed_plan(&basis);
    owner.cpus[0].inject_plan_result_for_test(
        basis,
        owner.runtime.lifecycle().round_attempt(),
        plan.clone(),
    );
    poll_through_manual_admission(&mut owner, &plan);
    assert!(!owner.cpus[0].has_committed_route());
    assert_eq!(owner.cpus[0].admission_trace_for_test().invalid_shape, 1);
}

#[test]
fn plain_plant_without_legacy_lease_cannot_bypass_mandatory_contact() {
    let mut owner = stopped_owner(CpuProfile::CONTACT_TOPOLOGY_V40_STANDARD);
    let basis = owner.cpus[0].latest_observation_for_test().unwrap();
    let mut plan = typed_plan(&basis);
    let Some(ExecutionCertificate::ContactPlant(contact)) = plan.execution.take() else {
        unreachable!()
    };
    plan.execution = Some(ExecutionCertificate::Plant(Box::new(
        contact.plant().clone(),
    )));
    owner.cpus[0].inject_plan_result_for_test(
        basis,
        owner.runtime.lifecycle().round_attempt(),
        plan.clone(),
    );
    poll_through_manual_admission(&mut owner, &plan);
    assert!(!owner.cpus[0].has_committed_route());
    assert_eq!(owner.cpus[0].admission_trace_for_test().contact, 1);
}

#[test]
fn foreign_bomb_revokes_at_each_real_publication_phase_before_next_update() {
    for reports_before_mutation in 0..3 {
        let (mut owner, planner) = driver_at_witness(
            CpuProfile::CONTACT_TOPOLOGY_V40_STANDARD,
            CpuDifficulty::Standard,
        );
        for _ in 0..700 {
            planner.pump_for_test(usize::MAX);
            owner.tick(Instant::now()).unwrap();
            if owner.cpus[0].has_committed_topology_contact()
                && owner.runtime.pending_report_count_for_test() == 0
            {
                break;
            }
        }
        assert!(owner.cpus[0].has_committed_topology_contact());
        assert_eq!(owner.runtime.pending_report_count_for_test(), 0);
        if reports_before_mutation == 0 {
            // Admission accepted Stop; the certified route retains a later
            // movement transition even though the owner is stationary now.
            assert!(owner.cpu_stationary[CPU.index()]);
            assert!(owner.cpus[0].has_committed_route());
        }
        for _ in 0..reports_before_mutation {
            owner.tick(Instant::now()).unwrap();
        }
        assert_eq!(
            owner.runtime.pending_report_count_for_test(),
            reports_before_mutation
        );
        owner
            .runtime
            .action(
                grenadiers_core::game::SeatId::One,
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
        assert!(owner.cpu_stop_at[CPU.index()].is_some());

        // The proof is conditional between publications. This only promises
        // withdrawal at the first observed mismatch and Stop on its next service.
        let mismatch_tick = owner.runtime.lifecycle().tick();
        owner.tick(Instant::now()).unwrap();
        assert_eq!(owner.runtime.lifecycle().tick().0, mismatch_tick.0 + 1);
        assert!(owner.cpu_stationary[CPU.index()]);
        assert!(!owner.cpus[0].has_pending());
    }
}

#[test]
fn late_contact_topology_worker_is_cancelled_without_authority_or_plant() {
    let (mut owner, planner) = driver_at_witness(
        CpuProfile::CONTACT_TOPOLOGY_V40_STANDARD,
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
    assert_eq!(report.delivered, 0);
    assert_eq!(report.cancelled, 1);
    assert!(!owner.cpus[0].has_committed_topology_contact());
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
fn exact_contact_expiry_revokes_one_update_before_exclusive_deadline() {
    let (mut owner, planner) = driver_at_witness(
        CpuProfile::CONTACT_TOPOLOGY_V40_STANDARD,
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
            .find(|bomb| bomb.owner() == grenadiers_core::game::EntityId(2))
        {
            break grenadiers_core::game::Tick(
                bomb.detonates_at().unwrap().0 + state.rules().flame_ticks(),
            );
        }
    };
    while owner.runtime.lifecycle().tick().0 + 2 < exclusive.0 {
        owner.tick(Instant::now()).unwrap();
    }
    assert_eq!(owner.runtime.lifecycle().tick().0 + 2, exclusive.0);
    assert!(owner.cpus[0].has_committed_topology_contact());

    owner.tick(Instant::now()).unwrap();
    assert_eq!(owner.runtime.lifecycle().tick().0 + 1, exclusive.0);
    assert!(!owner.cpus[0].has_committed_route());
    assert!(owner.cpu_stop_at[CPU.index()].is_some());

    owner.tick(Instant::now()).unwrap();
    assert_eq!(owner.runtime.lifecycle().tick(), exclusive);
    assert!(owner.cpu_stationary[CPU.index()]);
    assert!(
        owner
            .runtime
            .state_for_test()
            .unwrap()
            .actor(CPU)
            .unwrap()
            .alive()
    );
}

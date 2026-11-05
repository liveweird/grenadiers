use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::UtilityMemory;
use crate::cpu::{
    BombKnowledge, CpuRng, MemoryKey, Observation, ObservedBomb, ObservedBombTrigger,
    UnknownBombFacts, WorkBudget,
};
use crate::game::{BlastKind, CellId, EntityId, PunchActivationPolicy, Tick};

use super::foreign_bomb_farming_clear_fixture::captured_clear_basis;
use super::{
    WORK_ALLOWANCE, plan_hard_demolition_v64_with_origin, plan_hard_pickup_v66_with_origin,
    plan_hard_progress_v69_with_origin, plan_hard_reactive_v65_with_origin,
};

fn reactive_basis() -> (Observation, ObservedHazardMemory, UtilityMemory, MemoryKey) {
    let captured = captured_clear_basis();
    let observation = captured
        .observation
        .map_parts(|parts| {
            parts.rules.punch_activation_policy = PunchActivationPolicy::ContactV2;
            parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
                id: EntityId(70),
                placed_at: Tick(400),
                cell: CellId::new(1, 10),
                owner: EntityId(3),
                range: 2,
                trigger: ObservedBombTrigger::Timed {
                    remaining_ticks: 120,
                    paused: false,
                },
                blast: BlastKind::Standard,
                unknown: UnknownBombFacts::ALL,
            }]);
        })
        .unwrap();
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: observation.parts().recipient,
    };
    let mut history = ObservedHazardMemory::new(key);
    let previous = previous_sample(&observation);
    history.observe(&previous).unwrap();
    history.observe(&observation).unwrap();
    (observation, history, captured.memory, key)
}

fn previous_sample(observation: &Observation) -> Observation {
    observation
        .clone()
        .map_parts(|parts| {
            parts.tick = Tick(parts.tick.0 - 1);
            let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                unreachable!()
            };
            let ObservedBombTrigger::Timed {
                remaining_ticks, ..
            } = &mut bombs[0].trigger
            else {
                unreachable!()
            };
            *remaining_ticks += 1;
        })
        .unwrap()
}

#[test]
fn hard69_fresh_reactive_plan_preserves_hard66_output_work_and_rng() {
    let (observation, history, _, key) = reactive_basis();
    let memory = UtilityMemory::new(key);
    let mut old_rng = CpuRng::from_seed(key, [69; 32]);
    let mut new_rng = old_rng.clone();
    let mut old_work = WorkBudget::new(WORK_ALLOWANCE);
    let mut new_work = WorkBudget::new(WORK_ALLOWANCE);
    let old = plan_hard_pickup_v66_with_origin(
        &observation,
        &memory,
        &history,
        observation.parts().tick,
        None,
        &mut old_rng,
        &mut old_work,
    );
    let new = plan_hard_progress_v69_with_origin(
        &observation,
        &memory,
        &history,
        observation.parts().tick,
        None,
        &mut new_rng,
        &mut new_work,
    );
    let old = old.unwrap();
    let new = new.unwrap();
    assert_eq!(new.survival, old.survival);
    assert_eq!(format!("{:?}", new.contact), format!("{:?}", old.contact));
    assert_eq!(new_work.spent(), old_work.spent());
    assert_eq!(new_rng.draws(), old_rng.draws());
}

#[test]
fn hard65_keeps_the_bounded_fallback_when_all_remaining_candidate_work_is_exhausted() {
    let (observation, history, memory, key) = reactive_basis();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let mut rng = CpuRng::from_seed(key, [65; 32]);
    let planned = plan_hard_reactive_v65_with_origin(
        &observation,
        &memory,
        &history,
        observation.parts().tick,
        None,
        &mut rng,
        &mut work,
    )
    .unwrap();
    assert!(planned.survival.execution.is_none());
    assert_eq!(work.spent(), WORK_ALLOWANCE - 1);

    let mut guard = WorkBudget::new(8_192);
    assert!(
        super::reactive_assumptions::ReactivePlantAssumptions::certify(
            &observation,
            &history,
            observation.parts().tick,
            &mut guard,
        )
        .is_some()
    );
    let remaining = WORK_ALLOWANCE - guard.spent();
    let mut fallback_work = WorkBudget::new(remaining.min(32_768).min(remaining / 2));
    let mut fallback_rng = CpuRng::from_seed(key, [65; 32]);
    let fallback = plan_hard_demolition_v64_with_origin(
        &observation,
        &memory,
        &history,
        observation.parts().tick,
        None,
        &mut fallback_rng,
        &mut fallback_work,
    )
    .unwrap();
    assert_eq!(planned.survival.control, fallback.survival.control);
    assert_eq!(rng.draws(), fallback_rng.draws());
}

#[test]
fn stale_history_ending_before_the_basis_cannot_enable_hard65() {
    let (observation, _, memory, key) = reactive_basis();
    let mut stale = ObservedHazardMemory::new(key);
    stale.observe(&previous_sample(&observation)).unwrap();
    let planned = plan_hard_reactive_v65_with_origin(
        &observation,
        &memory,
        &stale,
        observation.parts().tick,
        None,
        &mut CpuRng::from_seed(key, [65; 32]),
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    assert!(
        planned
            .survival
            .execution
            .is_none_or(|execution| !execution.requires_reactive_support())
    );
}

#[test]
fn hard65_retains_the_v64_contact_policy_gate_and_preflight_accounting() {
    let (observation, history, memory, key) = reactive_basis();
    let wrong_policy = observation
        .map_parts(|parts| {
            parts.rules.punch_activation_policy = PunchActivationPolicy::ManualV1;
        })
        .unwrap();
    let mut expected = WorkBudget::new(super::reactive_plant::REACTIVE_GUARD_ALLOWANCE);
    let assumptions = super::reactive_assumptions::ReactivePlantAssumptions::certify(
        &wrong_policy,
        &history,
        wrong_policy.parts().tick,
        &mut expected,
    );
    if assumptions.is_none() {
        let _ = super::reactive_assumptions::ReactivePlantAssumptions::certify_escape(
            &wrong_policy,
            &history,
            wrong_policy.parts().tick,
            &mut expected,
        );
    }
    let mut v65_rng = CpuRng::from_seed(key, [65; 32]);
    let mut v65_work = WorkBudget::new(WORK_ALLOWANCE);
    let v65 = plan_hard_reactive_v65_with_origin(
        &wrong_policy,
        &memory,
        &history,
        wrong_policy.parts().tick,
        None,
        &mut v65_rng,
        &mut v65_work,
    );
    assert_eq!(
        v65.err(),
        Some(crate::cpu::policy::PolicyError::InvalidObservation)
    );
    assert_eq!(v65_work.spent(), expected.spent());
    assert_eq!(v65_rng.draws(), 0);
}

#[test]
fn own_escape_preflight_accepts_finite_flames_but_plant_preflight_does_not() {
    let (observation, history, _, _) = reactive_basis();
    let observation = observation
        .map_parts(|parts| {
            let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                unreachable!()
            };
            bombs[0].owner = EntityId(2);
            parts.flames.push(crate::cpu::ObservedFlame {
                cell: CellId::new(13, 9),
                remaining_ticks: 20,
            });
        })
        .unwrap();
    let mut escape_work = WorkBudget::new(8_192);
    assert!(
        super::reactive_assumptions::ReactivePlantAssumptions::certify_escape(
            &observation,
            &history,
            observation.parts().tick,
            &mut escape_work,
        )
        .is_some()
    );
    let mut plant_work = WorkBudget::new(8_192);
    assert!(
        super::reactive_assumptions::ReactivePlantAssumptions::certify(
            &observation,
            &history,
            observation.parts().tick,
            &mut plant_work,
        )
        .is_none()
    );
}

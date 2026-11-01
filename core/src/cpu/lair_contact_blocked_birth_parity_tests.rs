use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::UtilityMemory;
use crate::cpu::survival::{
    WORK_ALLOWANCE, plan_blocked_births_v42_with_origin, plan_contact_topology_v40_with_origin,
    plan_hard_blocked_births_v43_with_origin, plan_hard_contact_topology_v41_with_origin,
};
use crate::cpu::{CpuRng, MemoryKey, Observation, WorkBudget};

use super::late_farm_fixture::{CPU, captured_tick_10741};
use super::physical_fixture::{CPU as TRAPPED_CPU, controlled_state, memory};

#[test]
fn blocked_birth_profiles_preserve_exact_no_lair_planning() {
    let captured = captured_tick_10741();
    let observation = Observation::new_rules_v10(
        captured.parts().clone(),
        captured.environment().unwrap().clone(),
        None,
        captured.spike_pits().cloned(),
    )
    .unwrap();
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let memory = UtilityMemory::new(key);
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observation).unwrap();
    for hard in [false, true] {
        let mut old_rng = CpuRng::from_seed(key, [90; 32]);
        let mut new_rng = old_rng.clone();
        let mut old_work = WorkBudget::new(WORK_ALLOWANCE);
        let mut new_work = WorkBudget::new(WORK_ALLOWANCE);
        let old = if hard {
            plan_hard_contact_topology_v41_with_origin(
                &observation,
                &memory,
                &history,
                observation.parts().tick,
                None,
                &mut old_rng,
                &mut old_work,
            )
        } else {
            plan_contact_topology_v40_with_origin(
                &observation,
                &memory,
                &history,
                observation.parts().tick,
                None,
                &mut old_rng,
                &mut old_work,
            )
        }
        .unwrap();
        let new = if hard {
            plan_hard_blocked_births_v43_with_origin(
                &observation,
                &memory,
                &history,
                observation.parts().tick,
                None,
                &mut new_rng,
                &mut new_work,
            )
        } else {
            plan_blocked_births_v42_with_origin(
                &observation,
                &memory,
                &history,
                observation.parts().tick,
                None,
                &mut new_rng,
                &mut new_work,
            )
        }
        .unwrap();
        assert_eq!(new.survival, old.survival);
        assert_eq!(new.contact, old.contact);
        assert_eq!(new_work.spent(), old_work.spent());
        assert_eq!(new_rng.draws(), old_rng.draws());
    }
}

#[test]
fn blocked_birth_mode_retains_existing_mob_as_nonempty_authority() {
    let (mut state, settings) = controlled_state();
    for _ in 0..crate::room::RespawnLairSettings::DEFAULT.spawn_ticks() {
        state.step(&[]).unwrap();
    }
    assert!(!state.mobs().is_empty());
    let observation = crate::cpu::survival::observe_with_lairs(
        &state,
        (settings, crate::room::RespawnLairSettings::DEFAULT),
    );
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: TRAPPED_CPU,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observation).unwrap();
    let mut old_rng = CpuRng::from_seed(key, [42; 32]);
    let mut new_rng = old_rng.clone();
    let mut old_work = WorkBudget::new(WORK_ALLOWANCE);
    let mut new_work = WorkBudget::new(WORK_ALLOWANCE);
    let old = plan_contact_topology_v40_with_origin(
        &observation,
        &memory(&observation),
        &history,
        observation.parts().tick,
        None,
        &mut old_rng,
        &mut old_work,
    )
    .unwrap();
    let plan = plan_blocked_births_v42_with_origin(
        &observation,
        &memory(&observation),
        &history,
        observation.parts().tick,
        None,
        &mut new_rng,
        &mut new_work,
    )
    .unwrap();
    assert_eq!(
        plan.survival
            .execution
            .as_ref()
            .and_then(|execution| execution.contact_plant_requires_blocked_birth_support()),
        Some(false),
        "an Existing source must remain bound instead of becoming empty authority"
    );
    assert_eq!(plan.survival, old.survival);
    assert_eq!(plan.contact, old.contact);
    assert_eq!(new_work.spent(), old_work.spent());
    assert_eq!(new_work.spent(), 13_548);
    assert_eq!(new_rng.draws(), old_rng.draws());
}

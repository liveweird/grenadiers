use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::UtilityMemory;
use crate::cpu::{CpuRng, MemoryKey, MovementIntent, WorkBudget};
use crate::game::{InputAction, SeatInput, Tick};

use super::refuge_centering_test_fixture::{
    CPU, captured_scene, physical_observation, physical_state,
};
use super::{
    CommittedPulse, PulseStatus, WORK_ALLOWANCE, plan_hard_pickup_progress_v32_with_origin,
    plan_refuge_guidance_v34_with_origin,
};

fn run_episode(centered: bool) -> (bool, Tick, u32) {
    let mut state = physical_state();
    let template = captured_scene(CPU);
    let first = physical_observation(&state, &template);
    let key = MemoryKey {
        world: first.parts().world,
        cpu_seat: CPU,
    };
    let mut memory = UtilityMemory::new(key);
    let mut history = ObservedHazardMemory::new(key);
    let mut rng = CpuRng::from_seed(key, [131; 32]);
    let mut max_work = 0;

    while state.tick().0 < 701 && state.actor(CPU).unwrap().alive() {
        let observation = physical_observation(&state, &template);
        history.observe(&observation).unwrap();
        let mut work = WorkBudget::new(WORK_ALLOWANCE);
        let plan = if centered {
            plan_refuge_guidance_v34_with_origin(
                &observation,
                &memory,
                &history,
                observation.parts().tick,
                None,
                &mut rng,
                &mut work,
            )
        } else {
            plan_hard_pickup_progress_v32_with_origin(
                &observation,
                &memory,
                &history,
                observation.parts().tick,
                None,
                &mut rng,
                &mut work,
            )
        }
        .unwrap();
        max_work = max_work.max(work.spent());
        let Some(lease) = plan.survival.lease else {
            state
                .step(&[SeatInput {
                    seat: CPU,
                    action: InputAction::Clear,
                }])
                .unwrap();
            continue;
        };
        for _ in 0..6 {
            state
                .step(&[SeatInput {
                    seat: CPU,
                    action: InputAction::Clear,
                }])
                .unwrap();
        }
        let current = physical_observation(&state, &template);
        let Ok(pulse) = CommittedPulse::admit(lease.clone(), observation, &current, state.tick())
        else {
            continue;
        };
        memory = plan.survival.control.next_memory;
        for _ in 0..lease.hold_ticks() {
            let action = match lease.movement() {
                MovementIntent::Stop => InputAction::Clear,
                MovementIntent::Direction(direction) => InputAction::Press(direction),
            };
            state.step(&[SeatInput { seat: CPU, action }]).unwrap();
            let current = physical_observation(&state, &template);
            if pulse.validate(&current, state.tick()) == PulseStatus::Invalidated {
                break;
            }
        }
        state
            .step(&[SeatInput {
                seat: CPU,
                action: InputAction::Clear,
            }])
            .unwrap();
        let current = physical_observation(&state, &template);
        let _ = pulse.validate(&current, state.tick());
    }
    (state.actor(CPU).unwrap().alive(), state.tick(), max_work)
}

#[test]
fn captured_local_blast_episode_old_policy_dies_and_centering_policy_survives() {
    let old = run_episode(false);
    let centered = run_episode(true);

    assert_eq!((old.0, old.1), (false, Tick(670)));
    assert_eq!((centered.0, centered.1), (true, Tick(701)));
    assert!(old.2 <= WORK_ALLOWANCE, "old policy spent {}", old.2);
    assert!(
        centered.2 <= WORK_ALLOWANCE,
        "centering policy spent {}",
        centered.2
    );
}

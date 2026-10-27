use grenadiers_core::{
    cpu::{
        Intent, MemoryKey,
        policy::{ControlPlan, UtilityMemory},
        survival::SurvivalPlan,
    },
    game::{Direction, SeatId, Tick},
};

use super::tests::{open_fixture_state, publish_next_cpu_observation, running_driver};

#[test]
fn non_lease_long_move_keeps_its_deadline_across_intermediate_publications() {
    let mut driver = running_driver(23);
    driver
        .runtime
        .replace_state_for_test(open_fixture_state(driver.runtime.lifecycle().tick()));
    let initial = driver.runtime.current_notice().unwrap();
    driver.consider_cpu_observation(&initial);
    let basis = publish_next_cpu_observation(&mut driver);
    driver.cpu.as_mut().unwrap().cancel();
    let admission_tick = driver.runtime.lifecycle().tick();
    let deadline = Tick(admission_tick.0 + 12);
    let epoch = driver.runtime.lifecycle().round_attempt();
    let key = MemoryKey {
        world: basis.parts().world,
        cpu_seat: SeatId::Two,
    };
    driver.cpu.as_mut().unwrap().inject_plan_result_for_test(
        basis,
        epoch,
        SurvivalPlan {
            control: ControlPlan {
                intent: Intent::moving(Direction::West),
                hold_ticks: 12,
                next_memory: UtilityMemory::new(key),
            },
            lease: None,
            execution: None,
        },
    );

    driver.poll_cpu_result().unwrap();
    assert_eq!(driver.cpu_stop_at, Some(deadline));
    assert!(!driver.cpu_stationary);

    for _ in 0..3 {
        driver.apply_cpu_stop().unwrap();
        let _ = publish_next_cpu_observation(&mut driver);
        assert_eq!(driver.cpu_stop_at, Some(deadline));
        assert!(!driver.cpu_stationary);
    }

    while driver.runtime.lifecycle().tick() <= deadline {
        driver.apply_cpu_stop().unwrap();
        let _ = publish_next_cpu_observation(&mut driver);
    }

    assert!(driver.cpu_stationary);
    assert!(driver.cpu_stop_at.is_none());
    assert!(
        driver.cpu.as_ref().unwrap().has_pending(),
        "the post-deadline Stop snapshot should confirm settlement and request the next plan"
    );
}

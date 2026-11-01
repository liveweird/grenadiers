use grenadiers_core::{
    cpu::{DecisionId, Intent, MovementIntent, Observation, policy::UtilityMemory},
    game::{Direction, SeatId, Tick},
    room::CpuProfile,
};

use super::{AcceptedPlan, CpuController};
use crate::game::rooms::cpu::{CpuPlanner, worker_teleporter_tests::teleporter_observation};

#[test]
fn archived_teleporter_easy_retains_the_real_stop_through_the_same_twelve_tick_pause() {
    assert_teleporter_easy_cadence(CpuProfile::TELEPORTER_V46_EASY);
}

#[test]
fn current_teleporter_easy_retains_the_real_stop_through_the_same_twelve_tick_pause() {
    assert_teleporter_easy_cadence(CpuProfile::TELEPORTER_V48_EASY);
}

#[test]
fn multiple_teleporter_easy_retains_the_real_stop_through_the_same_twelve_tick_pause() {
    assert_teleporter_easy_cadence(CpuProfile::TELEPORTER_V50_EASY);
}

#[test]
fn pit_avoidance_easy_retains_the_real_stop_through_the_same_twelve_tick_pause() {
    assert_teleporter_easy_cadence(CpuProfile::PIT_AVOIDANCE_V52_EASY);
}

fn assert_teleporter_easy_cadence(profile: CpuProfile) {
    let basis = at_tick(teleporter_observation(), Tick(20));
    let mut controller = CpuController::new(
        CpuPlanner::without_workers(),
        SeatId::Two,
        7,
        [46; 32],
        profile,
    );
    assert!(controller.accept_observation(&basis));
    controller.settled_stop_tick = Some(basis.parts().tick);
    let key = controller.memory.as_ref().unwrap().key();
    let intent = Intent::moving(Direction::West);
    let accepted = AcceptedPlan {
        decision: DecisionId {
            world: basis.parts().world,
            command_epoch: 4,
            observation_tick: basis.parts().tick,
            checked_request_serial: 1,
        },
        intent,
        hold_ticks: 1,
        next_memory: UtilityMemory::new(key),
        next_rng: controller.rng.as_ref().unwrap().clone(),
        committed_pulse: None,
        committed_route: None,
        contact: None,
        certified_manual: None,
        escape_origin: false,
        ordinary_navigation: true,
        refuge_handoff: false,
    };
    controller.on_movement_admitted();
    controller.commit(accepted, intent);
    assert_eq!(intent.movement, MovementIntent::Direction(Direction::West));
    controller.on_stop_admitted(Tick(20));

    for tick in 21..33 {
        let observation = at_tick(basis.clone(), Tick(tick));
        assert!(controller.accept_observation(&observation));
        assert!(!controller.submit(observation, 4));
        assert_eq!(controller.settled_stop_tick, Some(Tick(21)));
    }
    let due = at_tick(basis, Tick(33));
    assert!(controller.accept_observation(&due));
    assert_eq!(controller.settled_stop_tick, Some(Tick(21)));
    assert!(controller.submit(due, 4));
}

fn at_tick(observation: Observation, tick: Tick) -> Observation {
    observation
        .map_parts(|parts| {
            let elapsed = tick.0 - parts.tick.0;
            parts.tick = tick;
            parts.rules.running_updates =
                parts.rules.running_updates.saturating_add(elapsed as u16);
            parts.rules.remaining_ticks =
                parts.rules.remaining_ticks.saturating_sub(elapsed as u16);
        })
        .unwrap()
}

use grenadiers_core::cpu::ObservedEnvironment;
use grenadiers_core::cpu::{Observation, ObservedActor, survival::PulseStatus};
use grenadiers_core::game::{CellId, EntityId, Position, SeatId, StormCloudConfig};

use super::{CpuController, CpuPlanner};

fn disabled_clear_scene() -> Observation {
    let basis = crate::game::rooms::cpu::tests::respawn_observation();
    let environment =
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0), Some(0)]);
    let mut parts = basis.into_parts();
    parts.arena.blocks.clear();
    let ObservedActor::Visible(cpu) = &parts.actors[1] else {
        unreachable!()
    };
    let mut rival = cpu.clone();
    rival.id = EntityId(1);
    rival.seat = SeatId::One;
    rival.position = Position::center(CellId::new(11, 7));
    parts.actors[0] = ObservedActor::Visible(rival);
    Observation::new_rules_v9(parts, environment, None).unwrap()
}

fn next_stopped(observation: Observation) -> Observation {
    let environment = observation.environment().unwrap().clone();
    let mut parts = observation.into_parts();
    parts.tick.0 += 1;
    parts.rules.running_updates += 1;
    parts.rules.remaining_ticks -= 1;
    Observation::new_rules_v9(parts, environment, None).unwrap()
}

#[test]
fn queued_rules_nine_disabled_lair_navigation_admits_a_checked_pulse() {
    let planner = CpuPlanner::without_workers();
    let mut controller = CpuController::new(
        planner.clone(),
        SeatId::Two,
        7,
        [21; 32],
        grenadiers_core::room::CpuProfile::RESPAWN_LAIRS_STANDARD,
    );
    let basis = disabled_clear_scene();
    assert!(controller.accept_observation(&basis));
    controller.on_stop_admitted(basis.parts().tick);
    let current = next_stopped(basis);
    assert!(controller.accept_observation(&current));
    assert!(controller.submit(current.clone(), 4));
    let report = planner.pump_for_test(1);
    assert_eq!(report.delivered, 1);

    let accepted = controller
        .poll(current.parts().tick, 4)
        .expect("checked disabled-lair pulse is admitted");
    assert_ne!(
        accepted.intent.movement,
        grenadiers_core::cpu::MovementIntent::Stop
    );
    let intent = accepted.intent;
    controller.on_movement_admitted();
    controller.commit(accepted, intent);
    assert_eq!(
        controller.validate_pulse(&current, current.parts().tick),
        Some(PulseStatus::Continue)
    );
}

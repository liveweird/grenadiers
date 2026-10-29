use crate::cpu::forecast::TickWindow;
use crate::cpu::survival::{CommittedPulse, PulseStatus};
use crate::cpu::{MovementIntent, Observation, ObservedEnvironment, WorkBudget};
use crate::game::types::HALF_FOOTPRINT;
use crate::game::{
    CellId, Direction, Position, STORM_CLOUD_HALF_EXTENT, STORM_CLOUD_SPEED_PER_TICK,
    StormCloudConfig, StormCloudView, Tick,
};

use super::storm_tests::custom_environment_observation;
use super::tests::forecast_for;
use super::{PulseRequest, RouteMode, certify_pulse};

#[test]
fn stopped_suffix_refresh_is_retained_through_completion_validation() {
    let actor_position = Position::center(CellId::new(2, 1));
    let contact_extent = STORM_CLOUD_HALF_EXTENT + HALF_FOOTPRINT;
    let cloud_position = Position {
        x: actor_position.x - contact_extent - 12,
        y: actor_position.y,
    };
    let cloud = StormCloudView {
        position: cloud_position,
        direction: Direction::East,
        leg_remaining_ticks: 128,
    };
    let basis = custom_environment_observation(cloud, actor_position, 0);
    let forecast = forecast_for(&basis);
    let request = PulseRequest {
        movement: MovementIntent::Stop,
        hold_ticks: 1,
        start_window: TickWindow {
            start: Tick(1),
            end: Tick(2),
        },
        stopped_safe_until: Tick(23),
        mode: RouteMode::CertifiedOnly,
        settled_stop_tick: Tick(0),
    };
    let lease = certify_pulse(&basis, &forecast, request, &mut WorkBudget::new(65_536)).unwrap();
    let refreshed = lease.storm_deadlines_at(Tick(1), Tick(2)).unwrap();
    assert_eq!(refreshed.lower, Tick(182));
    assert_eq!(refreshed.upper, Tick(182));

    let mut current_parts = basis.clone().into_parts();
    current_parts.tick = Tick(2);
    let current = Observation::new_rules_v7(
        current_parts,
        ObservedEnvironment::new(
            StormCloudConfig::Single,
            Some(StormCloudView {
                position: Position {
                    x: cloud_position.x + 2 * STORM_CLOUD_SPEED_PER_TICK,
                    y: cloud_position.y,
                },
                direction: Direction::East,
                leg_remaining_ticks: 126,
            }),
            vec![Some(0), Some(180)],
        ),
    )
    .unwrap();
    let committed = CommittedPulse::admit(lease, basis.clone(), &basis, Tick(0)).unwrap();
    assert_eq!(
        committed.validate(&current, Tick(2)),
        PulseStatus::Completed
    );
}

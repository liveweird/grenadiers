use crate::game::{StormCloudConfig, StormCloudView};

use super::observation::{ObservationError, ObservationParts, ObservedActor};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedEnvironment {
    storm_cloud_config: StormCloudConfig,
    storm_cloud: Option<StormCloudView>,
    actor_storm_slow_remaining_ticks: Vec<Option<u16>>,
}

impl ObservedEnvironment {
    #[must_use]
    pub fn new(
        storm_cloud_config: StormCloudConfig,
        storm_cloud: Option<StormCloudView>,
        actor_storm_slow_remaining_ticks: Vec<Option<u16>>,
    ) -> Self {
        Self {
            storm_cloud_config,
            storm_cloud,
            actor_storm_slow_remaining_ticks,
        }
    }

    #[must_use]
    pub const fn storm_cloud_config(&self) -> StormCloudConfig {
        self.storm_cloud_config
    }

    #[must_use]
    pub const fn storm_cloud(&self) -> Option<StormCloudView> {
        self.storm_cloud
    }

    #[must_use]
    pub fn actor_storm_slow_remaining_ticks(&self) -> &[Option<u16>] {
        &self.actor_storm_slow_remaining_ticks
    }
}

pub(super) fn validate_environment(
    parts: &ObservationParts,
    environment: &ObservedEnvironment,
) -> Result<(), ObservationError> {
    let timers = environment.actor_storm_slow_remaining_ticks();
    let timer_shape_valid = timers.len() == parts.actors.len()
        && timers
            .iter()
            .zip(&parts.actors)
            .all(|(timer, actor)| match actor {
                ObservedActor::Visible(_) => timer
                    .is_some_and(|remaining| u64::from(remaining) <= crate::game::STORM_SLOW_TICKS),
                ObservedActor::Hidden { .. } => timer.is_none(),
            });
    let cloud_valid = match (environment.storm_cloud_config(), environment.storm_cloud()) {
        (StormCloudConfig::Disabled, None) => timers.iter().flatten().all(|timer| *timer == 0),
        (StormCloudConfig::Single, Some(cloud)) => {
            let min = crate::game::Position::center(crate::game::CellId::new(1, 1));
            let max = crate::game::Position::center(crate::game::CellId::new(
                parts.arena.width - 2,
                parts.arena.height - 2,
            ));
            cloud.leg_remaining_ticks <= crate::game::STORM_CLOUD_LEG_TICKS
                && (min.x..=max.x).contains(&cloud.position.x)
                && (min.y..=max.y).contains(&cloud.position.y)
        }
        _ => false,
    };
    if timer_shape_valid && cloud_valid {
        Ok(())
    } else {
        Err(ObservationError::InvalidActors)
    }
}

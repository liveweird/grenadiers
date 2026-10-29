use std::sync::Arc;

use grenadiers_core::{
    game::{SeatId, StormCloudConfig},
    room::{MatchChange, RoomSettings},
};

use crate::round_seed::RoundSeedSource;

use super::{
    error::{RosterRuntimeError, lifecycle},
    publication_v15,
    runtime::{RosterRuntime, SimulationSeedDerivation},
    types::{RosterIdentity, RosterNotice},
};

impl RosterRuntime {
    pub(in crate::game::rooms) const fn environment(&self) -> Option<StormCloudConfig> {
        self.environment
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::game::rooms) fn new_environment(
        identity: RosterIdentity,
        settings: RoomSettings,
        environment: StormCloudConfig,
        round_seed_source: RoundSeedSource,
        simulation_seed: Arc<SimulationSeedDerivation>,
    ) -> Result<Self, RosterRuntimeError> {
        let mut runtime = Self::new(identity, settings, round_seed_source, simulation_seed)?;
        runtime.environment = Some(environment);
        Ok(runtime)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::game::rooms) fn update_environment_settings(
        &mut self,
        seat: SeatId,
        expected_epoch: u64,
        expected: u32,
        settings: RoomSettings,
        environment: StormCloudConfig,
    ) -> Result<Option<RosterNotice>, RosterRuntimeError> {
        let mut next = self.clone();
        let notice =
            next.apply_environment_settings(seat, expected_epoch, expected, settings, environment)?;
        *self = next;
        Ok(notice)
    }

    fn apply_environment_settings(
        &mut self,
        seat: SeatId,
        expected_epoch: u64,
        expected: u32,
        settings: RoomSettings,
        environment: StormCloudConfig,
    ) -> Result<Option<RosterNotice>, RosterRuntimeError> {
        self.validate_actor(seat, expected_epoch)?;
        let current = self
            .environment
            .ok_or(RosterRuntimeError::UnsupportedConfiguration)?;
        if self.lifecycle.settings_editor() != Some(seat) {
            return Err(RosterRuntimeError::SettingsLocked);
        }
        if expected != self.settings_revision {
            return Err(RosterRuntimeError::StaleSettings);
        }
        if settings == self.lifecycle.settings() && environment == current {
            let room = publication_v15::room_view(
                &self.identity,
                &self.lifecycle,
                current,
                self.revision,
                self.settings_revision,
            )?;
            return Ok(Some(publication_v15::view_notice(room)));
        }
        let mut change = self
            .lifecycle
            .set_settings(seat, settings)
            .map_err(lifecycle)?;
        let environment_changed = environment != current;
        if environment_changed && change.change() == MatchChange::Unchanged {
            for human in self
                .identity
                .configuration
                .human_seats()
                .collect::<Vec<_>>()
            {
                if self.lifecycle.ready(human).map_err(lifecycle)? {
                    change = self.lifecycle.set_ready(human, false).map_err(lifecycle)?;
                }
            }
        }
        self.environment = Some(environment);
        self.settings_revision = self
            .settings_revision
            .checked_add(1)
            .ok_or(RosterRuntimeError::CapacityReached)?;
        if environment_changed && change.change() == MatchChange::Unchanged {
            self.publish_view().map(Some)
        } else {
            self.notice_for(change.change(), false)
        }
    }
}

use std::sync::Arc;

use grenadiers_core::{
    game::{SeatId, StormCloudConfig},
    room::{
        DepartureReason, LairTuning, MatchChange, RoomSettings, RosterParticipant, RosterUpdate,
    },
};

use crate::round_seed::RoundSeedSource;

use super::{
    error::{RosterRuntimeError, lifecycle},
    publication_v15, publication_v16,
    runtime::{EnvironmentProfile, RosterRuntime, SimulationSeedDerivation},
    types::{RosterIdentity, RosterNotice},
};

impl RosterRuntime {
    pub(in crate::game::rooms) const fn forming_replacements(&self) -> bool {
        self.environment_profile.is_some() && !self.settings_locked
    }

    pub(super) const fn settings_authority_locked(&self) -> bool {
        self.settings_locked || self.creator_settings_revoked
    }

    pub(super) fn apply_departures(
        &mut self,
        departures: [Option<DepartureReason>; 4],
    ) -> Result<RosterUpdate, RosterRuntimeError> {
        let forming = self.forming_replacements();
        let update = if forming {
            self.lifecycle.depart_forming_with_grace(departures)
        } else {
            self.lifecycle.depart_with_grace(departures)
        }
        .map_err(lifecycle)?;
        if forming
            && departures[SeatId::One.index()] == Some(DepartureReason::Explicit)
            && self.lifecycle.participant(SeatId::One).map_err(lifecycle)?
                == RosterParticipant::NeverAdmitted
        {
            self.creator_settings_revoked = true;
        }
        Ok(update)
    }

    pub(super) fn expire_departures(&mut self) -> Result<RosterUpdate, RosterRuntimeError> {
        let forming = self.forming_replacements();
        let creator_was_reserved = matches!(
            self.lifecycle.participant(SeatId::One).map_err(lifecycle)?,
            RosterParticipant::Reserved { .. }
        );
        let update = if forming {
            self.lifecycle.expire_forming_reconnect_grace()
        } else {
            self.lifecycle.expire_reconnect_grace()
        }
        .map_err(lifecycle)?;
        if forming
            && creator_was_reserved
            && self.lifecycle.participant(SeatId::One).map_err(lifecycle)?
                == RosterParticipant::NeverAdmitted
        {
            self.creator_settings_revoked = true;
        }
        Ok(update)
    }

    pub(in crate::game::rooms) const fn environment(&self) -> Option<StormCloudConfig> {
        match self.environment_profile {
            Some(profile) => Some(profile.cloud()),
            None => None,
        }
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
        runtime.environment_profile = Some(EnvironmentProfile::StormV7(environment));
        Ok(runtime)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::game::rooms) fn new_lairs(
        identity: RosterIdentity,
        settings: RoomSettings,
        cloud: StormCloudConfig,
        tuning: LairTuning,
        round_seed_source: RoundSeedSource,
        simulation_seed: Arc<SimulationSeedDerivation>,
    ) -> Result<Self, RosterRuntimeError> {
        let mut runtime = Self::new(identity, settings, round_seed_source, simulation_seed)?;
        runtime.environment_profile = Some(EnvironmentProfile::LairsV8 { cloud, tuning });
        Ok(runtime)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::game::rooms) const fn lair_tuning(&self) -> Option<LairTuning> {
        match self.environment_profile {
            Some(EnvironmentProfile::LairsV8 { tuning, .. }) => Some(tuning),
            Some(EnvironmentProfile::StormV7(_)) | None => None,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::game::rooms) fn update_lair_settings(
        &mut self,
        seat: SeatId,
        expected_epoch: u64,
        expected: u32,
        settings: RoomSettings,
        cloud: StormCloudConfig,
        tuning: LairTuning,
    ) -> Result<Option<RosterNotice>, RosterRuntimeError> {
        let mut next = self.clone();
        let notice =
            next.apply_lair_settings(seat, expected_epoch, expected, settings, cloud, tuning)?;
        *self = next;
        Ok(notice)
    }

    fn apply_lair_settings(
        &mut self,
        seat: SeatId,
        expected_epoch: u64,
        expected: u32,
        settings: RoomSettings,
        cloud: StormCloudConfig,
        tuning: LairTuning,
    ) -> Result<Option<RosterNotice>, RosterRuntimeError> {
        self.validate_actor(seat, expected_epoch)?;
        let Some(EnvironmentProfile::LairsV8 {
            cloud: current_cloud,
            tuning: current_tuning,
        }) = self.environment_profile
        else {
            return Err(RosterRuntimeError::UnsupportedConfiguration);
        };
        if self.settings_authority_locked() || self.lifecycle.settings_editor() != Some(seat) {
            return Err(RosterRuntimeError::SettingsLocked);
        }
        if expected != self.settings_revision {
            return Err(RosterRuntimeError::StaleSettings);
        }
        if settings == self.lifecycle.settings()
            && cloud == current_cloud
            && tuning == current_tuning
        {
            let room = publication_v16::room_view(
                &self.identity,
                &self.lifecycle,
                current_cloud,
                current_tuning,
                self.revision,
                self.settings_revision,
                self.settings_authority_locked(),
            )?;
            return Ok(Some(publication_v16::view_notice(room)));
        }
        let mut change = self
            .lifecycle
            .set_settings(seat, settings)
            .map_err(lifecycle)?;
        let environment_changed = cloud != current_cloud || tuning != current_tuning;
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
        self.environment_profile = Some(EnvironmentProfile::LairsV8 { cloud, tuning });
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
        let Some(EnvironmentProfile::StormV7(current)) = self.environment_profile else {
            return Err(RosterRuntimeError::UnsupportedConfiguration);
        };
        if self.settings_authority_locked() || self.lifecycle.settings_editor() != Some(seat) {
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
                self.settings_authority_locked(),
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
        self.environment_profile = Some(EnvironmentProfile::StormV7(environment));
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

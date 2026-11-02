use std::sync::Arc;

use grenadiers_core::{
    game::{SeatId, StormCloudConfig},
    room::{
        DepartureReason, LairTuning, MatchChange, RespawnLairTuning, RoomSettings,
        RosterParticipant, RosterUpdate,
    },
};

use crate::round_seed::RoundSeedSource;

use super::{
    error::{RosterRuntimeError, lifecycle},
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
    pub(in crate::game::rooms) fn new_respawn(
        identity: RosterIdentity,
        settings: RoomSettings,
        cloud: StormCloudConfig,
        tuning: RespawnLairTuning,
        round_seed_source: RoundSeedSource,
        simulation_seed: Arc<SimulationSeedDerivation>,
    ) -> Result<Self, RosterRuntimeError> {
        let mut runtime = Self::new(identity, settings, round_seed_source, simulation_seed)?;
        runtime.environment_profile = Some(EnvironmentProfile::RespawnV9 { cloud, tuning });
        Ok(runtime)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    #[allow(clippy::too_many_arguments)]
    pub(in crate::game::rooms) fn new_respawn_with_navigation(
        identity: RosterIdentity,
        settings: RoomSettings,
        cloud: StormCloudConfig,
        tuning: RespawnLairTuning,
        round_seed_source: RoundSeedSource,
        simulation_seed: Arc<SimulationSeedDerivation>,
        navigation: grenadiers_core::game::MobNavigationPolicy,
    ) -> Result<Self, RosterRuntimeError> {
        let mut runtime = Self::new_respawn(
            identity,
            settings,
            cloud,
            tuning,
            round_seed_source,
            simulation_seed,
        )?;
        runtime.mob_navigation = navigation;
        Ok(runtime)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::game::rooms) const fn lair_tuning(&self) -> Option<LairTuning> {
        match self.environment_profile {
            Some(EnvironmentProfile::LairsV8 { tuning, .. }) => Some(tuning),
            Some(
                EnvironmentProfile::StormV7(_)
                | EnvironmentProfile::RespawnV9 { .. }
                | EnvironmentProfile::SpikeV10 { .. },
            )
            | None => None,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::game::rooms) const fn respawn_tuning(&self) -> Option<RespawnLairTuning> {
        match self.environment_profile {
            Some(EnvironmentProfile::RespawnV9 { tuning, .. }) => Some(tuning),
            Some(EnvironmentProfile::SpikeV10 { lairs, .. }) => Some(lairs),
            Some(EnvironmentProfile::StormV7(_) | EnvironmentProfile::LairsV8 { .. }) | None => {
                None
            }
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::game::rooms) fn update_respawn_settings(
        &mut self,
        seat: SeatId,
        expected_epoch: u64,
        expected: u32,
        settings: RoomSettings,
        cloud: StormCloudConfig,
        tuning: RespawnLairTuning,
    ) -> Result<Option<RosterNotice>, RosterRuntimeError> {
        let mut next = self.clone();
        let notice =
            next.apply_respawn_settings(seat, expected_epoch, expected, settings, cloud, tuning)?;
        *self = next;
        Ok(notice)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn apply_respawn_settings(
        &mut self,
        seat: SeatId,
        expected_epoch: u64,
        expected: u32,
        settings: RoomSettings,
        cloud: StormCloudConfig,
        tuning: RespawnLairTuning,
    ) -> Result<Option<RosterNotice>, RosterRuntimeError> {
        if !matches!(
            self.environment_profile,
            Some(EnvironmentProfile::RespawnV9 { .. })
        ) {
            return Err(RosterRuntimeError::UnsupportedConfiguration);
        }
        self.apply_profile_settings(
            seat,
            expected_epoch,
            expected,
            settings,
            EnvironmentProfile::RespawnV9 { cloud, tuning },
        )
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
        if !matches!(
            self.environment_profile,
            Some(EnvironmentProfile::LairsV8 { .. })
        ) {
            return Err(RosterRuntimeError::UnsupportedConfiguration);
        }
        self.apply_profile_settings(
            seat,
            expected_epoch,
            expected,
            settings,
            EnvironmentProfile::LairsV8 { cloud, tuning },
        )
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
        if !matches!(
            self.environment_profile,
            Some(EnvironmentProfile::StormV7(_))
        ) {
            return Err(RosterRuntimeError::UnsupportedConfiguration);
        }
        self.apply_profile_settings(
            seat,
            expected_epoch,
            expected,
            settings,
            EnvironmentProfile::StormV7(environment),
        )
    }

    pub(super) fn apply_profile_settings(
        &mut self,
        seat: SeatId,
        expected_epoch: u64,
        expected: u32,
        settings: RoomSettings,
        profile: EnvironmentProfile,
    ) -> Result<Option<RosterNotice>, RosterRuntimeError> {
        self.validate_actor(seat, expected_epoch)?;
        if self.settings_authority_locked() || self.lifecycle.settings_editor() != Some(seat) {
            return Err(RosterRuntimeError::SettingsLocked);
        }
        if expected != self.settings_revision {
            return Err(RosterRuntimeError::StaleSettings);
        }
        if settings == self.lifecycle.settings() && Some(profile) == self.environment_profile {
            return self.profile_view_notice().map(Some);
        }
        let mut change = self
            .lifecycle
            .set_settings(seat, settings)
            .map_err(lifecycle)?;
        let environment_changed = Some(profile) != self.environment_profile;
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
        self.environment_profile = Some(profile);
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

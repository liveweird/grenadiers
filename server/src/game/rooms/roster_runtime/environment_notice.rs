use super::{
    error::RosterRuntimeError,
    publication_v15, publication_v16, publication_v17, publication_v18, publication_v19,
    publication_v20,
    runtime::{EnvironmentProfile, RosterRuntime, SpikePublicationVersion},
    types::RosterNotice,
};

impl RosterRuntime {
    pub(super) fn profile_view_notice(&self) -> Result<RosterNotice, RosterRuntimeError> {
        match self.environment_profile {
            Some(EnvironmentProfile::StormV7(cloud)) => publication_v15::room_view(
                &self.identity,
                &self.lifecycle,
                cloud,
                self.revision,
                self.settings_revision,
                self.settings_authority_locked(),
            )
            .map(publication_v15::view_notice),
            Some(EnvironmentProfile::LairsV8 { cloud, tuning }) => publication_v16::room_view(
                &self.identity,
                &self.lifecycle,
                EnvironmentProfile::LairsV8 { cloud, tuning },
                self.revision,
                self.settings_revision,
                self.settings_authority_locked(),
            )
            .map(publication_v16::view_notice),
            Some(EnvironmentProfile::RespawnV9 { cloud, tuning }) => publication_v17::room_view(
                &self.identity,
                &self.lifecycle,
                EnvironmentProfile::RespawnV9 { cloud, tuning },
                self.revision,
                self.settings_revision,
                self.settings_authority_locked(),
            )
            .map(publication_v17::view_notice),
            Some(profile @ EnvironmentProfile::SpikeV10 { .. }) => self.spike_view_notice(profile),
            None => Err(RosterRuntimeError::UnsupportedConfiguration),
        }
    }

    fn spike_view_notice(
        &self,
        profile: EnvironmentProfile,
    ) -> Result<RosterNotice, RosterRuntimeError> {
        match self.spike_publication {
            SpikePublicationVersion::V18 => publication_v18::room_view(
                &self.identity,
                &self.lifecycle,
                profile,
                self.revision,
                self.settings_revision,
                self.settings_authority_locked(),
            )
            .map(publication_v18::view_notice),
            SpikePublicationVersion::V19 => publication_v19::room_view(
                &self.identity,
                &self.lifecycle,
                profile,
                self.revision,
                self.settings_revision,
                self.settings_authority_locked(),
            )
            .map(publication_v19::view_notice),
            SpikePublicationVersion::V20 => publication_v20::room_view(
                &self.identity,
                &self.lifecycle,
                profile,
                self.revision,
                self.settings_revision,
                self.settings_authority_locked(),
            )
            .map(publication_v20::view_notice),
        }
    }
}

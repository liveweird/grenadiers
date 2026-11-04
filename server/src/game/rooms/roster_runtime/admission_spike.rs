use grenadiers_core::{
    game::GenerationRecord,
    protocol::{Counter, v18, v19, v20, v21},
    room::RosterMatchLifecycle,
};

use super::{
    admission_projection::PreparedProjection,
    error::RosterRuntimeError,
    publication_v18, publication_v19, publication_v20, publication_v21,
    runtime::{EnvironmentProfile, RosterRuntime, SimRound, SpikePublicationVersion},
    types::RosterNotice,
};

#[derive(Clone, Copy)]
struct AdmissionState {
    epoch: Option<Counter>,
    generation: Option<GenerationRecord>,
}

impl RosterRuntime {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn prepare_spike_admission(
        &self,
        lifecycle: &RosterMatchLifecycle,
        profile: EnvironmentProfile,
        revision: u32,
        body: Option<&SimRound>,
        state_epoch: Option<Counter>,
        generation: Option<GenerationRecord>,
    ) -> Result<(PreparedProjection, RosterNotice), RosterRuntimeError> {
        if self.spike_publication == SpikePublicationVersion::V21 {
            return self.prepare_spike_v21_admission(
                lifecycle,
                profile,
                revision,
                body,
                AdmissionState {
                    epoch: state_epoch,
                    generation,
                },
            );
        }
        if self.spike_publication == SpikePublicationVersion::V20 {
            return self.prepare_spike_v20_admission(
                lifecycle,
                profile,
                revision,
                body,
                state_epoch,
                generation,
            );
        }
        if self.spike_publication == SpikePublicationVersion::V19 {
            return self.prepare_spike_v19_admission(
                lifecycle,
                profile,
                revision,
                body,
                state_epoch,
                generation,
            );
        }
        let room = publication_v18::room_view(
            &self.identity,
            lifecycle,
            profile,
            revision,
            self.settings_revision,
            self.settings_authority_locked(),
        )?;
        let capture = body
            .map(|sim| {
                v18::capture_expanded(&sim.state, &[]).map_err(|_| RosterRuntimeError::Publication)
            })
            .transpose()?;
        let notice = if let Some(sim) = body {
            publication_v18::snapshot(room.clone(), sim.epoch, &sim.state, &[], sim.generation)?
        } else {
            publication_v18::view_notice(room.clone())
        };
        Ok((
            PreparedProjection::Spike {
                room,
                state_epoch,
                capture,
                generation,
            },
            notice,
        ))
    }

    fn prepare_spike_v21_admission(
        &self,
        lifecycle: &RosterMatchLifecycle,
        profile: EnvironmentProfile,
        revision: u32,
        body: Option<&SimRound>,
        state: AdmissionState,
    ) -> Result<(PreparedProjection, RosterNotice), RosterRuntimeError> {
        let room = publication_v21::room_view(
            &self.identity,
            lifecycle,
            profile,
            revision,
            self.settings_revision,
            self.settings_authority_locked(),
        )?;
        let capture = body
            .map(|sim| {
                v21::capture_expanded(&sim.state, &[]).map_err(|_| RosterRuntimeError::Publication)
            })
            .transpose()?;
        let notice = if let Some(sim) = body {
            publication_v21::snapshot(
                room.clone(),
                sim.epoch,
                &sim.state,
                &[],
                sim.generation,
                self.construction_profile,
            )?
        } else {
            publication_v21::view_notice(room.clone())
        };
        Ok((
            PreparedProjection::SpikeV21 {
                room,
                state_epoch: state.epoch,
                capture,
                generation: state.generation,
                construction: self.construction_profile,
            },
            notice,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare_spike_v19_admission(
        &self,
        lifecycle: &RosterMatchLifecycle,
        profile: EnvironmentProfile,
        revision: u32,
        body: Option<&SimRound>,
        state_epoch: Option<Counter>,
        generation: Option<GenerationRecord>,
    ) -> Result<(PreparedProjection, RosterNotice), RosterRuntimeError> {
        let room = publication_v19::room_view(
            &self.identity,
            lifecycle,
            profile,
            revision,
            self.settings_revision,
            self.settings_authority_locked(),
        )?;
        let capture = body
            .map(|sim| {
                v19::capture_expanded(&sim.state, &[]).map_err(|_| RosterRuntimeError::Publication)
            })
            .transpose()?;
        let notice = if let Some(sim) = body {
            publication_v19::snapshot(room.clone(), sim.epoch, &sim.state, &[], sim.generation)?
        } else {
            publication_v19::view_notice(room.clone())
        };
        Ok((
            PreparedProjection::SpikeV19 {
                room,
                state_epoch,
                capture,
                generation,
            },
            notice,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare_spike_v20_admission(
        &self,
        lifecycle: &RosterMatchLifecycle,
        profile: EnvironmentProfile,
        revision: u32,
        body: Option<&SimRound>,
        state_epoch: Option<Counter>,
        generation: Option<GenerationRecord>,
    ) -> Result<(PreparedProjection, RosterNotice), RosterRuntimeError> {
        let room = publication_v20::room_view(
            &self.identity,
            lifecycle,
            profile,
            revision,
            self.settings_revision,
            self.settings_authority_locked(),
        )?;
        let capture = body
            .map(|sim| {
                v20::capture_expanded(&sim.state, &[]).map_err(|_| RosterRuntimeError::Publication)
            })
            .transpose()?;
        let notice = if let Some(sim) = body {
            publication_v20::snapshot(
                room.clone(),
                sim.epoch,
                &sim.state,
                &[],
                sim.generation,
                self.construction_profile,
            )?
        } else {
            publication_v20::view_notice(room.clone())
        };
        Ok((
            PreparedProjection::SpikeV20 {
                room,
                state_epoch,
                capture,
                generation,
                construction: self.construction_profile,
            },
            notice,
        ))
    }
}

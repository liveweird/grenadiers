use grenadiers_core::{
    game::SeatId,
    protocol::{Counter, v14, v15, v16, v17},
    room::{MatchPhase, RosterMatchLifecycle, RosterParticipant},
};

use super::{
    admission_projection::PreparedProjection,
    error::{RosterRuntimeError, lifecycle as lifecycle_error},
    publication, publication_v15, publication_v16, publication_v17,
    runtime::{EnvironmentProfile, RosterRuntime},
    types::RosterNotice,
};

pub(in crate::game::rooms) struct PreparedRosterAdmission {
    pub(super) lifecycle: RosterMatchLifecycle,
    pub(super) revision: u32,
    pub(super) seat: SeatId,
    pub(in crate::game::rooms) is_replacement: bool,
    projection: PreparedProjection,
    pub(in crate::game::rooms) last_action_id: Option<Counter>,
    pub(in crate::game::rooms) notice: RosterNotice,
}

impl PreparedRosterAdmission {
    pub(in crate::game::rooms) fn encode(
        &self,
        sequence: Counter,
        resume_token: v14::ResumeToken,
    ) -> Result<String, RosterRuntimeError> {
        self.projection
            .encode(self.seat, self.last_action_id, sequence, resume_token)
            .map_err(|_| RosterRuntimeError::Publication)
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn legacy_room(&self) -> Option<&v14::RoomView> {
        match &self.projection {
            PreparedProjection::Legacy { room, .. } => Some(room),
            PreparedProjection::Environment { .. }
            | PreparedProjection::Lairs { .. }
            | PreparedProjection::Respawn { .. } => None,
            PreparedProjection::Spike { .. } | PreparedProjection::SpikeV19 { .. } => None,
        }
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn environment_room(&self) -> Option<&v15::RoomView> {
        match &self.projection {
            PreparedProjection::Environment { room, .. } => Some(room),
            PreparedProjection::Legacy { .. } => None,
            PreparedProjection::Lairs { .. }
            | PreparedProjection::Respawn { .. }
            | PreparedProjection::Spike { .. }
            | PreparedProjection::SpikeV19 { .. } => None,
        }
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn lair_room(&self) -> Option<&v16::RoomView> {
        match &self.projection {
            PreparedProjection::Lairs { room, .. } => Some(room),
            PreparedProjection::Legacy { .. }
            | PreparedProjection::Environment { .. }
            | PreparedProjection::Respawn { .. }
            | PreparedProjection::Spike { .. }
            | PreparedProjection::SpikeV19 { .. } => None,
        }
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn respawn_room(&self) -> Option<&v17::RoomView> {
        match &self.projection {
            PreparedProjection::Respawn { room, .. } => Some(room),
            PreparedProjection::Legacy { .. }
            | PreparedProjection::Environment { .. }
            | PreparedProjection::Lairs { .. }
            | PreparedProjection::Spike { .. }
            | PreparedProjection::SpikeV19 { .. } => None,
        }
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn spike_room(
        &self,
    ) -> Option<&grenadiers_core::protocol::v18::RoomView> {
        match &self.projection {
            PreparedProjection::Spike { room, .. } => Some(room),
            _ => None,
        }
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn spike_v19_room(
        &self,
    ) -> Option<&grenadiers_core::protocol::v19::RoomView> {
        match &self.projection {
            PreparedProjection::SpikeV19 { room, .. } => Some(room),
            _ => None,
        }
    }

    pub(in crate::game::rooms) fn room_metadata(
        &self,
    ) -> (&grenadiers_core::protocol::RoomId, u64) {
        match &self.projection {
            PreparedProjection::Legacy { room, .. } => {
                (&room.room_id, u64::from(room.round_id.get()))
            }
            PreparedProjection::Environment { room, .. } => {
                (&room.room_id, u64::from(room.round_id.get()))
            }
            PreparedProjection::Lairs { room, .. } => {
                (&room.room_id, u64::from(room.round_id.get()))
            }
            PreparedProjection::Respawn { room, .. } => {
                (&room.room_id, u64::from(room.round_id.get()))
            }
            PreparedProjection::Spike { room, .. } => {
                (&room.room_id, u64::from(room.round_id.get()))
            }
            PreparedProjection::SpikeV19 { room, .. } => {
                (&room.room_id, u64::from(room.round_id.get()))
            }
        }
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn test_metadata(
        &self,
    ) -> (&grenadiers_core::protocol::RoomCode, u32) {
        match &self.projection {
            PreparedProjection::Legacy { room, .. } => (&room.room_code, room.revision.get()),
            PreparedProjection::Environment { room, .. } => (&room.room_code, room.revision.get()),
            PreparedProjection::Lairs { room, .. } => (&room.room_code, room.revision.get()),
            PreparedProjection::Respawn { room, .. } => (&room.room_code, room.revision.get()),
            PreparedProjection::Spike { room, .. } => (&room.room_code, room.revision.get()),
            PreparedProjection::SpikeV19 { room, .. } => (&room.room_code, room.revision.get()),
        }
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn legacy_room_mut(&mut self) -> Option<&mut v14::RoomView> {
        match &mut self.projection {
            PreparedProjection::Legacy { room, .. } => Some(room),
            PreparedProjection::Environment { .. }
            | PreparedProjection::Lairs { .. }
            | PreparedProjection::Respawn { .. }
            | PreparedProjection::Spike { .. }
            | PreparedProjection::SpikeV19 { .. } => None,
        }
    }
}

impl RosterRuntime {
    pub(in crate::game::rooms) fn prepare_join(
        &self,
        seat: SeatId,
    ) -> Result<PreparedRosterAdmission, RosterRuntimeError> {
        let mut lifecycle = self.lifecycle.clone();
        lifecycle.occupy(seat).map_err(lifecycle_error)?;
        self.prepare_admission(seat, lifecycle, false)
    }

    pub(in crate::game::rooms) fn prepare_resume(
        &self,
        seat: SeatId,
    ) -> Result<PreparedRosterAdmission, RosterRuntimeError> {
        let mut lifecycle = self.lifecycle.clone();
        let is_replacement = match lifecycle.participant(seat).map_err(lifecycle_error)? {
            RosterParticipant::Reserved { .. } => {
                lifecycle.resume_connection(seat).map_err(lifecycle_error)?;
                false
            }
            RosterParticipant::Connected => true,
            RosterParticipant::NeverAdmitted
            | RosterParticipant::Departed
            | RosterParticipant::Cpu => return Err(RosterRuntimeError::StaleEpoch),
        };
        self.prepare_admission(seat, lifecycle, is_replacement)
    }

    fn prepare_admission(
        &self,
        seat: SeatId,
        lifecycle: RosterMatchLifecycle,
        is_replacement: bool,
    ) -> Result<PreparedRosterAdmission, RosterRuntimeError> {
        let revision = self.next_revision()?;
        let body = self.sim.as_ref().filter(|_| {
            matches!(
                lifecycle.phase(),
                MatchPhase::Running | MatchPhase::Result { .. } | MatchPhase::MatchResult { .. }
            )
        });
        let state_epoch = body
            .map(|sim| publication::counter64(sim.epoch))
            .transpose()?;
        let generation = body.map(|sim| sim.generation);
        let room_round_id = publication::counter64(lifecycle.round_attempt())?;
        let last_action_id = if state_epoch == Some(room_round_id) {
            self.inputs
                .last_manual_action(seat)
                .map(publication::counter)
                .transpose()?
        } else {
            None
        };
        let (projection, notice) = match self.environment_profile {
            Some(EnvironmentProfile::StormV7(environment)) => {
                let room = publication_v15::room_view(
                    &self.identity,
                    &lifecycle,
                    environment,
                    revision,
                    self.settings_revision,
                    self.settings_authority_locked(),
                )?;
                let capture = body
                    .map(|sim| {
                        v15::capture_expanded(&sim.state, &[])
                            .map_err(|_| RosterRuntimeError::Publication)
                    })
                    .transpose()?;
                let notice = if let Some(sim) = body {
                    publication_v15::snapshot(
                        room.clone(),
                        sim.epoch,
                        &sim.state,
                        &[],
                        sim.generation,
                    )?
                } else {
                    publication_v15::view_notice(room.clone())
                };
                (
                    PreparedProjection::Environment {
                        room,
                        state_epoch,
                        capture,
                        generation,
                    },
                    notice,
                )
            }
            Some(EnvironmentProfile::LairsV8 { cloud, tuning }) => {
                let room = publication_v16::room_view(
                    &self.identity,
                    &lifecycle,
                    EnvironmentProfile::LairsV8 { cloud, tuning },
                    revision,
                    self.settings_revision,
                    self.settings_authority_locked(),
                )?;
                let capture = body
                    .map(|sim| {
                        v16::capture_expanded(&sim.state, &[])
                            .map_err(|_| RosterRuntimeError::Publication)
                    })
                    .transpose()?;
                let notice = if let Some(sim) = body {
                    publication_v16::snapshot(
                        room.clone(),
                        sim.epoch,
                        &sim.state,
                        &[],
                        sim.generation,
                    )?
                } else {
                    publication_v16::view_notice(room.clone())
                };
                (
                    PreparedProjection::Lairs {
                        room,
                        state_epoch,
                        capture,
                        generation,
                    },
                    notice,
                )
            }
            Some(EnvironmentProfile::RespawnV9 { cloud, tuning }) => {
                let room = publication_v17::room_view(
                    &self.identity,
                    &lifecycle,
                    EnvironmentProfile::RespawnV9 { cloud, tuning },
                    revision,
                    self.settings_revision,
                    self.settings_authority_locked(),
                )?;
                let capture = body
                    .map(|sim| {
                        v17::capture_expanded(&sim.state, &[])
                            .map_err(|_| RosterRuntimeError::Publication)
                    })
                    .transpose()?;
                let notice = if let Some(sim) = body {
                    publication_v17::snapshot(
                        room.clone(),
                        sim.epoch,
                        &sim.state,
                        &[],
                        sim.generation,
                    )?
                } else {
                    publication_v17::view_notice(room.clone())
                };
                (
                    PreparedProjection::Respawn {
                        room,
                        state_epoch,
                        capture,
                        generation,
                    },
                    notice,
                )
            }
            Some(EnvironmentProfile::SpikeV10 {
                cloud,
                lairs,
                spike_pits,
            }) => self.prepare_spike_admission(
                &lifecycle,
                EnvironmentProfile::SpikeV10 {
                    cloud,
                    lairs,
                    spike_pits,
                },
                revision,
                body,
                state_epoch,
                generation,
            )?,
            None => {
                let room = publication::room_view(
                    &self.identity,
                    &lifecycle,
                    revision,
                    self.settings_revision,
                )?;
                let capture = body
                    .map(|sim| {
                        v14::capture_expanded(&sim.state, &[])
                            .map_err(|_| RosterRuntimeError::Publication)
                    })
                    .transpose()?;
                let notice = if let Some(sim) = body {
                    publication::snapshot(room.clone(), sim.epoch, &sim.state, &[], sim.generation)?
                } else {
                    publication::view_notice(room.clone())
                };
                (
                    PreparedProjection::Legacy {
                        room,
                        state_epoch,
                        capture,
                        generation,
                    },
                    notice,
                )
            }
        };
        Ok(PreparedRosterAdmission {
            lifecycle,
            revision,
            seat,
            is_replacement,
            projection,
            last_action_id,
            notice,
        })
    }
}

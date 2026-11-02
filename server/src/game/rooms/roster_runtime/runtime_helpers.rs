#[cfg(test)]
use grenadiers_core::room::RoomSettings;
use grenadiers_core::{
    game::SeatId,
    room::{MatchPhase, RosterParticipant},
};

#[cfg(test)]
use super::types::RosterNotice;
use super::{
    error::{RosterRuntimeError, lifecycle},
    publication,
    runtime::RosterRuntime,
};

impl RosterRuntime {
    #[cfg(test)]
    pub(super) fn update_settings(
        &mut self,
        seat: SeatId,
        expected: u32,
        settings: RoomSettings,
    ) -> Result<Option<RosterNotice>, RosterRuntimeError> {
        if self.settings_locked || self.lifecycle.settings_editor() != Some(seat) {
            return Err(RosterRuntimeError::SettingsLocked);
        }
        if expected != self.settings_revision {
            return Err(RosterRuntimeError::StaleSettings);
        }
        if settings == self.lifecycle.settings() {
            let room = publication::room_view(
                &self.identity,
                &self.lifecycle,
                self.revision,
                self.settings_revision,
            )?;
            return Ok(Some(publication::view_notice(room)));
        }
        let change = self
            .lifecycle
            .set_settings(seat, settings)
            .map_err(lifecycle)?;
        self.settings_revision = self
            .settings_revision
            .checked_add(1)
            .ok_or(RosterRuntimeError::CapacityReached)?;
        self.notice_for(change.change(), false)
    }

    pub(super) fn validate_actor(
        &self,
        seat: SeatId,
        epoch: u64,
    ) -> Result<(), RosterRuntimeError> {
        if epoch != self.lifecycle.round_attempt() {
            return Err(RosterRuntimeError::StaleEpoch);
        }
        if !matches!(
            self.lifecycle.participant(seat).map_err(lifecycle)?,
            RosterParticipant::Connected | RosterParticipant::Cpu
        ) {
            return Err(RosterRuntimeError::VacantSeat);
        }
        Ok(())
    }

    pub(super) fn require_running(&self) -> Result<(), RosterRuntimeError> {
        (self.lifecycle.phase() == MatchPhase::Running)
            .then_some(())
            .ok_or(RosterRuntimeError::InvalidPhase)
    }

    pub(super) fn next_revision(&self) -> Result<u32, RosterRuntimeError> {
        self.revision
            .checked_add(1)
            .and_then(|value| publication::counter(value).ok())
            .map(|counter| counter.get())
            .ok_or(RosterRuntimeError::CapacityReached)
    }

    pub(super) fn reset_for_rematch(&mut self) {
        self.sim = None;
        self.reports.clear();
        self.cpu_publication = None;
        self.environment_cpu_publication = None;
        self.lair_cpu_publication = None;
        self.respawn_cpu_publication = None;
        self.spike_cpu_publication = None;
        self.spike_v19_cpu_publication = None;
        self.spike_v20_cpu_publication = None;
        self.inputs.reset_round();
    }
}

use grenadiers_core::{
    game::SeatId,
    room::{DepartureReason, MatchChange, MatchPhase, RoomSettings, reconnect::ReservationState},
};

use super::{MatchNotice, MatchRuntime, MatchRuntimeError, map_lifecycle_error};

impl MatchRuntime {
    pub(super) fn ready(
        &mut self,
        seat: SeatId,
        ready: bool,
    ) -> Result<Option<MatchNotice>, MatchRuntimeError> {
        if !matches!(
            self.lifecycle.phase(),
            MatchPhase::Waiting
                | MatchPhase::Countdown {
                    kind: grenadiers_core::room::CountdownKind::Initial,
                    ..
                }
        ) {
            return Err(MatchRuntimeError::InvalidPhase);
        }
        self.ensure_revision()?;
        let previous_epoch = self.lifecycle.round_attempt();
        let change = self
            .lifecycle
            .set_ready(seat, ready)
            .map_err(map_lifecycle_error)?;
        if previous_epoch != self.lifecycle.round_attempt() {
            self.inputs.reset_round();
        }
        self.notice_for_change(change, false)
    }

    pub(super) fn ready_checked(
        &mut self,
        seat: SeatId,
        ready: bool,
        expected_revision: u32,
    ) -> Result<Option<MatchNotice>, MatchRuntimeError> {
        if ready && expected_revision != self.settings_revision {
            return Err(MatchRuntimeError::StaleSettings);
        }
        self.ready(seat, ready)
    }

    pub(super) fn update_settings(
        &mut self,
        seat: SeatId,
        expected_revision: u32,
        settings: RoomSettings,
    ) -> Result<Option<MatchNotice>, MatchRuntimeError> {
        if self.settings_editor != Some(seat)
            || self.lifecycle.phase() != MatchPhase::Waiting
            || self.lifecycle.seats()[seat.index()].ready()
        {
            return Err(MatchRuntimeError::SettingsLocked);
        }
        if expected_revision != self.settings_revision {
            return Err(MatchRuntimeError::StaleSettings);
        }
        if settings == self.lifecycle.settings() {
            let room = super::super::publication::room_view(
                &self.identity,
                &self.lifecycle,
                self.revision,
                self.settings_revision,
                self.settings_editor,
            )?;
            return Ok(Some(MatchNotice {
                room,
                frames: None,
                cpu_observation: None,
            }));
        }
        self.ensure_revision()?;
        let next_settings_revision = self
            .settings_revision
            .checked_add(1)
            .and_then(|value| grenadiers_core::protocol::Counter::new(value).ok())
            .map(|counter| counter.get())
            .ok_or(MatchRuntimeError::CapacityReached)?;
        let change = self
            .lifecycle
            .set_settings(settings)
            .map_err(map_lifecycle_error)?;
        if change != MatchChange::Changed {
            return Err(MatchRuntimeError::Publication);
        }
        self.settings_revision = next_settings_revision;
        self.publish_view().map(Some)
    }

    pub(super) fn release_settings_editor(&mut self, seat: SeatId) {
        if self.settings_editor == Some(seat) {
            self.settings_editor = None;
        }
    }

    pub(super) fn release_departed_editor(&mut self, departures: [Option<DepartureReason>; 2]) {
        for (seat, departure) in SeatId::ALL.into_iter().zip(departures) {
            let permanently_released = matches!(
                self.lifecycle.reconnect_grace().seat(seat),
                ReservationState::Vacant | ReservationState::Expired
            );
            if departure == Some(DepartureReason::Explicit) || permanently_released {
                self.release_settings_editor(seat);
            }
        }
    }

    pub(super) fn release_expired_editor(&mut self) {
        if self.settings_editor.is_some_and(|seat| {
            matches!(
                self.lifecycle.reconnect_grace().seat(seat),
                ReservationState::Expired
            )
        }) {
            self.settings_editor = None;
        }
    }

    pub(super) fn rematch(
        &mut self,
        seat: SeatId,
        ready: bool,
    ) -> Result<Option<MatchNotice>, MatchRuntimeError> {
        if !matches!(self.lifecycle.phase(), MatchPhase::MatchResult { .. }) {
            return Err(MatchRuntimeError::InvalidPhase);
        }
        self.ensure_revision()?;
        let change = self
            .lifecycle
            .set_rematch_vote(seat, ready)
            .map_err(map_lifecycle_error)?;
        if change == MatchChange::RematchStarted {
            self.clear_round();
            return self.publish_view().map(Some);
        }
        self.notice_for_change(change, true)
    }
}

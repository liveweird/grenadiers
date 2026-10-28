//! Pure two-to-four-seat roster and match lifecycle facts.

use crate::game::{RoundAssignment, RoundResult, SeatId, Tick};
use crate::room::series::{CompletedRound, MatchOutcome};
use crate::room::{
    DepartureReason, InterruptReason, MatchChange, MatchPhase, RoomConfiguration, RoomSettings,
    SeatControl,
};

pub const ROSTER_LIFECYCLE_PROFILE: u16 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RosterParticipant {
    NeverAdmitted,
    Connected,
    Reserved { expires_at: Tick },
    Departed,
    Cpu,
}

impl RosterParticipant {
    const fn eligible(self) -> bool {
        matches!(self, Self::Connected | Self::Reserved { .. } | Self::Cpu)
    }

    const fn connected_human(self) -> bool {
        matches!(self, Self::Connected)
    }

    const fn retained_human(self) -> bool {
        matches!(self, Self::Connected | Self::Reserved { .. })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RosterError {
    InvalidPhase,
    UnsupportedSeat(SeatId),
    CpuTarget(SeatId),
    SeatAlreadyAdmitted(SeatId),
    SeatNeverAdmitted(SeatId),
    SeatNotConnected(SeatId),
    SeatNotReserved(SeatId),
    GraceExpired { seat: SeatId, expires_at: Tick },
    UnauthorizedSettingsEditor(SeatId),
    InvalidDepartureTail(SeatId),
    InvalidInterruptReason,
    TickOverflow,
    RoundAttemptOverflow,
    RoundNumberOverflow,
    ScoreOverflow,
    RematchUnavailable,
}

impl std::fmt::Display for RosterError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for RosterError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RosterUpdate {
    change: MatchChange,
    newly_ineligible: [bool; 4],
}

impl RosterUpdate {
    const fn new(change: MatchChange) -> Self {
        Self {
            change,
            newly_ineligible: [false; 4],
        }
    }

    #[must_use]
    pub const fn change(self) -> MatchChange {
        self.change
    }

    pub fn newly_ineligible_seats(self) -> impl Iterator<Item = SeatId> {
        SeatId::ALL
            .into_iter()
            .zip(self.newly_ineligible)
            .filter_map(|(seat, lost)| lost.then_some(seat))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActiveRound {
    assignment: RoundAssignment,
    started_at: Tick,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RosterMatchLifecycle {
    configuration: RoomConfiguration,
    settings: RoomSettings,
    tick: Tick,
    round_attempt: u64,
    phase: MatchPhase,
    participants: [RosterParticipant; 4],
    ready: [bool; 4],
    rematch_votes: [bool; 4],
    scores: [u8; 4],
    actual_round_number: u32,
    active: Option<ActiveRound>,
    latest_completed: Option<CompletedRound>,
    outcome: Option<MatchOutcome>,
}

impl RosterMatchLifecycle {
    #[must_use]
    pub fn new(configuration: RoomConfiguration, settings: RoomSettings) -> Self {
        let mut participants = [RosterParticipant::Departed; 4];
        for seat in configuration.player_count().seats() {
            participants[seat.index()] = match configuration.controller(*seat) {
                Some(SeatControl::Human) => RosterParticipant::NeverAdmitted,
                Some(SeatControl::Cpu) => RosterParticipant::Cpu,
                None => unreachable!(),
            };
        }
        Self {
            configuration,
            settings,
            tick: Tick(0),
            round_attempt: 1,
            phase: MatchPhase::Waiting,
            participants,
            ready: [false; 4],
            rematch_votes: [false; 4],
            scores: [0; 4],
            actual_round_number: 0,
            active: None,
            latest_completed: None,
            outcome: None,
        }
    }

    #[must_use]
    pub const fn configuration(&self) -> RoomConfiguration {
        self.configuration
    }
    #[must_use]
    pub const fn settings(&self) -> RoomSettings {
        self.settings
    }
    #[must_use]
    pub const fn tick(&self) -> Tick {
        self.tick
    }
    #[must_use]
    pub const fn round_attempt(&self) -> u64 {
        self.round_attempt
    }
    #[must_use]
    pub const fn phase(&self) -> MatchPhase {
        self.phase
    }
    #[must_use]
    pub fn participants(&self) -> &[RosterParticipant] {
        &self.participants[..self.configuration.player_count().get()]
    }
    pub fn participant(&self, seat: SeatId) -> Result<RosterParticipant, RosterError> {
        self.ensure_seat(seat)?;
        Ok(self.participants[seat.index()])
    }
    #[must_use]
    pub fn scores(&self) -> &[u8] {
        &self.scores[..self.configuration.player_count().get()]
    }
    #[must_use]
    pub const fn latest_completed(&self) -> Option<CompletedRound> {
        self.latest_completed
    }
    #[must_use]
    pub const fn actual_round_number(&self) -> u32 {
        self.actual_round_number
    }
    #[must_use]
    pub const fn outcome(&self) -> Option<MatchOutcome> {
        self.outcome
    }
    pub fn ready(&self, seat: SeatId) -> Result<bool, RosterError> {
        self.ensure_seat(seat)?;
        Ok(match self.configuration.controller(seat) {
            Some(SeatControl::Human) => self.ready[seat.index()],
            Some(SeatControl::Cpu) => matches!(
                self.phase,
                MatchPhase::Waiting | MatchPhase::Countdown { .. }
            ),
            None => unreachable!(),
        })
    }
    pub fn rematch_vote(&self, seat: SeatId) -> Result<bool, RosterError> {
        self.ensure_seat(seat)?;
        Ok(match self.configuration.controller(seat) {
            Some(SeatControl::Human) => self.rematch_votes[seat.index()],
            Some(SeatControl::Cpu) => matches!(
                self.phase,
                MatchPhase::MatchResult { rematch_at, .. } if self.tick >= rematch_at
            ),
            None => unreachable!(),
        })
    }
    #[must_use]
    pub fn retained_human_count(&self) -> usize {
        self.participants()
            .iter()
            .filter(|state| state.retained_human())
            .count()
    }
    pub fn eligible_seats(&self) -> impl Iterator<Item = SeatId> + '_ {
        self.configuration
            .player_count()
            .seats()
            .iter()
            .copied()
            .filter(|seat| self.participants[seat.index()].eligible())
    }
    #[must_use]
    pub fn settings_editor(&self) -> Option<SeatId> {
        self.participants[0]
            .connected_human()
            .then_some(SeatId::One)
    }

    pub fn occupy(&mut self, seat: SeatId) -> Result<RosterUpdate, RosterError> {
        self.transact(|next| next.occupy_inner(seat))
    }
    pub fn set_ready(&mut self, seat: SeatId, ready: bool) -> Result<RosterUpdate, RosterError> {
        self.transact(|next| next.set_ready_inner(seat, ready))
    }
    pub fn set_settings(
        &mut self,
        editor: SeatId,
        settings: RoomSettings,
    ) -> Result<RosterUpdate, RosterError> {
        self.transact(|next| next.set_settings_inner(editor, settings))
    }
    pub fn advance_idle_tick(&mut self) -> Result<RosterUpdate, RosterError> {
        self.transact(Self::advance_idle_tick_inner)
    }
    pub fn complete_running_tick(
        &mut self,
        result: Option<RoundResult>,
    ) -> Result<RosterUpdate, RosterError> {
        self.transact(|next| next.complete_running_tick_inner(result))
    }
    pub fn complete_current_tick(
        &mut self,
        result: RoundResult,
    ) -> Result<RosterUpdate, RosterError> {
        self.transact(|next| next.complete_current_tick_inner(result))
    }
    pub fn depart_with_grace(
        &mut self,
        departures: [Option<DepartureReason>; 4],
    ) -> Result<RosterUpdate, RosterError> {
        self.transact(|next| next.depart_with_grace_inner(departures))
    }
    pub fn resume_connection(&mut self, seat: SeatId) -> Result<RosterUpdate, RosterError> {
        self.transact(|next| next.resume_connection_inner(seat))
    }
    pub fn expire_reconnect_grace(&mut self) -> Result<RosterUpdate, RosterError> {
        self.transact(Self::expire_reconnect_grace_inner)
    }
    pub fn interrupt(&mut self, reason: InterruptReason) -> Result<RosterUpdate, RosterError> {
        self.transact(|next| next.interrupt_inner(reason))
    }
    pub fn set_rematch_vote(
        &mut self,
        seat: SeatId,
        ready: bool,
    ) -> Result<RosterUpdate, RosterError> {
        self.transact(|next| next.set_rematch_vote_inner(seat, ready))
    }

    fn transact<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, RosterError>,
    ) -> Result<T, RosterError> {
        let mut next = self.clone();
        let result = operation(&mut next)?;
        *self = next;
        Ok(result)
    }

    fn ensure_seat(&self, seat: SeatId) -> Result<(), RosterError> {
        self.configuration
            .player_count()
            .contains(seat)
            .then_some(())
            .ok_or(RosterError::UnsupportedSeat(seat))
    }
}

#[path = "operations.rs"]
mod operations;

#[path = "policy.rs"]
mod policy;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

//! Dormant pure match-level room lifecycle.

use crate::game::{RoundAssignment, RoundResult, SeatId, Tick};

use super::reconnect::{ReconnectError, ReconnectGrace};
use super::series::{MatchOutcome, MatchSeries, RematchStatus, SeriesError};
use super::{COUNTDOWN_TICKS, DepartureReason, InterruptReason, SeatState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CountdownKind {
    Initial,
    InterRound,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchPhase {
    Waiting,
    Countdown {
        kind: CountdownKind,
        ends_at: Tick,
    },
    Running,
    Result {
        next_countdown_at: Tick,
    },
    MatchResult {
        outcome: MatchOutcome,
        rematch_at: Tick,
    },
    Interrupted {
        ended_at: Tick,
        reason: InterruptReason,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchChange {
    Unchanged,
    Changed,
    CountdownStarted,
    RoundStarted(RoundAssignment),
    RoundFinished,
    MatchFinished(MatchOutcome),
    RematchStarted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchError {
    InvalidPhase,
    SeatOccupied(SeatId),
    SeatVacant(SeatId),
    InvalidInterruptReason,
    TickOverflow,
    RoundAttemptOverflow,
    Series(SeriesError),
    Reconnect(ReconnectError),
}

impl std::fmt::Display for MatchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for MatchError {}

impl From<SeriesError> for MatchError {
    fn from(error: SeriesError) -> Self {
        Self::Series(error)
    }
}

impl From<ReconnectError> for MatchError {
    fn from(error: ReconnectError) -> Self {
        Self::Reconnect(error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchLifecycle {
    tick: Tick,
    round_attempt: u64,
    phase: MatchPhase,
    seats: [SeatState; 2],
    series: MatchSeries,
    reconnect: ReconnectGrace,
}

impl Default for MatchLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl MatchLifecycle {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            tick: Tick(0),
            round_attempt: 1,
            phase: MatchPhase::Waiting,
            seats: [SeatState {
                occupied: false,
                ready: false,
            }; 2],
            series: MatchSeries::new(),
            reconnect: ReconnectGrace::new(),
        }
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
    pub const fn seats(&self) -> &[SeatState; 2] {
        &self.seats
    }

    #[must_use]
    pub const fn series(&self) -> &MatchSeries {
        &self.series
    }

    #[must_use]
    pub const fn reconnect_grace(&self) -> &ReconnectGrace {
        &self.reconnect
    }

    pub fn occupy(&mut self, seat: SeatId) -> Result<MatchChange, MatchError> {
        self.transact(|next| next.occupy_inner(seat))
    }

    pub fn set_ready(&mut self, seat: SeatId, ready: bool) -> Result<MatchChange, MatchError> {
        self.transact(|next| next.set_ready_inner(seat, ready))
    }

    pub fn advance_idle_tick(&mut self) -> Result<MatchChange, MatchError> {
        self.transact(Self::advance_idle_tick_inner)
    }

    pub fn complete_running_tick(
        &mut self,
        result: Option<RoundResult>,
    ) -> Result<MatchChange, MatchError> {
        self.transact(|next| next.complete_running_tick_inner(result))
    }

    pub fn depart_batch(
        &mut self,
        departures: [Option<DepartureReason>; 2],
    ) -> Result<MatchChange, MatchError> {
        self.transact(|next| next.depart_batch_inner(departures))
    }

    pub fn depart_with_grace(
        &mut self,
        departures: [Option<DepartureReason>; 2],
    ) -> Result<MatchChange, MatchError> {
        self.transact(|next| next.depart_with_grace_inner(departures))
    }

    pub fn resume_connection(&mut self, seat: SeatId) -> Result<MatchChange, MatchError> {
        self.transact(|next| next.resume_connection_inner(seat))
    }

    pub fn expire_reconnect_grace(&mut self) -> Result<MatchChange, MatchError> {
        self.transact(|next| next.depart_with_grace_inner([None; 2]))
    }

    pub fn interrupt(&mut self, reason: InterruptReason) -> Result<MatchChange, MatchError> {
        self.transact(|next| next.interrupt_inner(reason))
    }

    pub fn set_rematch_vote(
        &mut self,
        seat: SeatId,
        ready: bool,
    ) -> Result<MatchChange, MatchError> {
        self.transact(|next| next.set_rematch_vote_inner(seat, ready))
    }

    fn occupy_inner(&mut self, seat: SeatId) -> Result<MatchChange, MatchError> {
        if self.phase != MatchPhase::Waiting {
            return Err(MatchError::InvalidPhase);
        }
        let state = &mut self.seats[seat.index()];
        if state.occupied {
            return Err(MatchError::SeatOccupied(seat));
        }
        self.reconnect.occupy(seat, self.tick)?;
        state.occupied = true;
        Ok(MatchChange::Changed)
    }

    fn set_ready_inner(&mut self, seat: SeatId, ready: bool) -> Result<MatchChange, MatchError> {
        if !matches!(
            self.phase,
            MatchPhase::Waiting
                | MatchPhase::Countdown {
                    kind: CountdownKind::Initial,
                    ..
                }
        ) {
            return Err(MatchError::InvalidPhase);
        }
        let index = seat.index();
        if !self.seats[index].occupied {
            return Err(MatchError::SeatVacant(seat));
        }
        if !self.reconnect.seat(seat).connected() {
            return Err(MatchError::Reconnect(ReconnectError::SeatNotConnected(
                seat,
            )));
        }
        if self.seats[index].ready == ready {
            return Ok(MatchChange::Unchanged);
        }
        if matches!(self.phase, MatchPhase::Countdown { .. }) {
            let next_attempt = self.next_attempt()?;
            self.clear_readiness();
            self.round_attempt = next_attempt;
            self.phase = MatchPhase::Waiting;
            return Ok(MatchChange::Changed);
        }

        let starts = ready
            && SeatId::ALL.into_iter().all(|candidate| {
                self.reconnect.seat(candidate).connected()
                    && (candidate == seat || self.seats[candidate.index()].ready)
            });
        let ends_at = starts.then(|| self.deadline(COUNTDOWN_TICKS)).transpose()?;
        self.seats[index].ready = ready;
        if let Some(ends_at) = ends_at {
            self.phase = MatchPhase::Countdown {
                kind: CountdownKind::Initial,
                ends_at,
            };
            Ok(MatchChange::CountdownStarted)
        } else {
            Ok(MatchChange::Changed)
        }
    }

    fn advance_idle_tick_inner(&mut self) -> Result<MatchChange, MatchError> {
        if self.phase == MatchPhase::Running {
            return Err(MatchError::InvalidPhase);
        }
        if matches!(self.phase, MatchPhase::Interrupted { .. }) {
            return Ok(MatchChange::Unchanged);
        }
        let next_tick = self.next_tick()?;
        match self.phase {
            MatchPhase::Countdown { ends_at, .. }
                if next_tick == ends_at && self.both_connected() =>
            {
                let assignment = self.series.start_round(self.round_attempt, next_tick)?;
                self.tick = next_tick;
                self.clear_readiness();
                self.phase = MatchPhase::Running;
                Ok(MatchChange::RoundStarted(assignment))
            }
            MatchPhase::Result { next_countdown_at }
                if next_tick >= next_countdown_at && self.both_connected() =>
            {
                let next_attempt = self.next_attempt()?;
                let ends_at = Tick(
                    next_tick
                        .0
                        .checked_add(COUNTDOWN_TICKS)
                        .ok_or(MatchError::TickOverflow)?,
                );
                self.tick = next_tick;
                self.round_attempt = next_attempt;
                self.phase = MatchPhase::Countdown {
                    kind: CountdownKind::InterRound,
                    ends_at,
                };
                Ok(MatchChange::CountdownStarted)
            }
            MatchPhase::Waiting
            | MatchPhase::Countdown { .. }
            | MatchPhase::Result { .. }
            | MatchPhase::MatchResult { .. } => {
                self.tick = next_tick;
                Ok(MatchChange::Changed)
            }
            MatchPhase::Running | MatchPhase::Interrupted { .. } => unreachable!(),
        }
    }

    fn complete_running_tick_inner(
        &mut self,
        result: Option<RoundResult>,
    ) -> Result<MatchChange, MatchError> {
        if self.phase != MatchPhase::Running {
            return Err(MatchError::InvalidPhase);
        }
        let next_tick = self.next_tick()?;
        let Some(result) = result else {
            self.tick = next_tick;
            return Ok(MatchChange::Changed);
        };
        let outcome = self
            .series
            .complete_round(self.round_attempt, result, next_tick)?;
        self.tick = next_tick;
        if let Some(outcome) = outcome {
            let rematch_at = self.series.hold_deadline().expect("completed result hold");
            self.phase = MatchPhase::MatchResult {
                outcome,
                rematch_at,
            };
            Ok(MatchChange::MatchFinished(outcome))
        } else {
            let next_countdown_at = self.series.hold_deadline().expect("completed result hold");
            self.phase = MatchPhase::Result { next_countdown_at };
            Ok(MatchChange::RoundFinished)
        }
    }

    fn transact<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, MatchError>,
    ) -> Result<T, MatchError> {
        let mut next = self.clone();
        let result = operation(&mut next)?;
        *self = next;
        Ok(result)
    }

    fn clear_readiness(&mut self) {
        self.seats.iter_mut().for_each(|seat| seat.ready = false);
    }
}

#[path = "match_lifecycle_clock.rs"]
mod clock;

#[path = "match_lifecycle_operations.rs"]
mod operations;

#[path = "match_lifecycle_reconnect.rs"]
mod reconnect;

#[cfg(test)]
#[path = "match_lifecycle_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "match_lifecycle_departure_tests.rs"]
mod departure_tests;

#[cfg(test)]
#[path = "match_lifecycle_overflow_tests.rs"]
mod overflow_tests;

#[cfg(test)]
#[path = "match_lifecycle_reconnect_tests.rs"]
mod reconnect_tests;

#[cfg(test)]
#[path = "match_lifecycle_reconnect_terminal_tests.rs"]
mod reconnect_terminal_tests;

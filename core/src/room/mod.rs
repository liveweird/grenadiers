//! Pure room admission and one-round lifecycle state.
//!
//! ```
//! use grenadiers_core::game::{RoundResult, SeatId};
//! use grenadiers_core::room::{Change, Phase, RoomLifecycle, COUNTDOWN_TICKS};
//!
//! let mut room = RoomLifecycle::new();
//! room.occupy(SeatId::One).unwrap();
//! room.occupy(SeatId::Two).unwrap();
//! room.set_ready(SeatId::One, true).unwrap();
//! room.set_ready(SeatId::Two, true).unwrap();
//! for _ in 0..COUNTDOWN_TICKS {
//!     room.advance_idle_tick().unwrap();
//! }
//! assert_eq!(room.phase(), Phase::Running);
//! assert_eq!(
//!     room.complete_running_tick(Some(RoundResult::Winner(SeatId::One))),
//!     Ok(Change::Finished),
//! );
//! ```

pub mod match_lifecycle;
pub mod reconnect;
pub mod series;

pub use match_lifecycle::{CountdownKind, MatchChange, MatchError, MatchLifecycle, MatchPhase};

use crate::game::{RoundResult, SeatId, Tick};

pub const COUNTDOWN_TICKS: u64 = 180;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterruptReason {
    ParticipantLeft,
    ConnectionLost,
    ServerOverloaded,
    ServerStopped,
    InternalError,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepartureReason {
    Explicit,
    TransportLoss,
}

impl DepartureReason {
    const fn interrupt_reason(self) -> InterruptReason {
        match self {
            Self::Explicit => InterruptReason::ParticipantLeft,
            Self::TransportLoss => InterruptReason::ConnectionLost,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Waiting,
    Countdown {
        ends_at: Tick,
    },
    Running,
    Result {
        ended_at: Tick,
        result: RoundResult,
    },
    Interrupted {
        ended_at: Tick,
        reason: InterruptReason,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SeatState {
    occupied: bool,
    ready: bool,
}

impl SeatState {
    #[must_use]
    pub const fn occupied(self) -> bool {
        self.occupied
    }

    #[must_use]
    pub const fn ready(self) -> bool {
        self.ready
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    Unchanged,
    Changed,
    Started,
    Finished,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleError {
    InvalidPhase,
    SeatOccupied(SeatId),
    SeatVacant(SeatId),
    TickOverflow,
    RoundAttemptOverflow,
}

impl std::fmt::Display for LifecycleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidPhase => "operation is invalid in the current room phase",
            Self::SeatOccupied(_) => "room seat is already occupied",
            Self::SeatVacant(_) => "room seat is vacant",
            Self::TickOverflow => "room tick overflow",
            Self::RoundAttemptOverflow => "round attempt overflow",
        })
    }
}

impl std::error::Error for LifecycleError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomLifecycle {
    tick: Tick,
    round_attempt: u64,
    phase: Phase,
    seats: [SeatState; 2],
}

impl Default for RoomLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl RoomLifecycle {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            tick: Tick(0),
            round_attempt: 1,
            phase: Phase::Waiting,
            seats: [SeatState {
                occupied: false,
                ready: false,
            }; 2],
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
    pub const fn phase(&self) -> Phase {
        self.phase
    }

    #[must_use]
    pub const fn seats(&self) -> &[SeatState; 2] {
        &self.seats
    }

    pub fn occupy(&mut self, seat: SeatId) -> Result<Change, LifecycleError> {
        if self.phase != Phase::Waiting {
            return Err(LifecycleError::InvalidPhase);
        }
        let state = &mut self.seats[seat.index()];
        if state.occupied {
            return Err(LifecycleError::SeatOccupied(seat));
        }
        state.occupied = true;
        Ok(Change::Changed)
    }

    pub fn set_ready(&mut self, seat: SeatId, ready: bool) -> Result<Change, LifecycleError> {
        if !matches!(self.phase, Phase::Waiting | Phase::Countdown { .. }) {
            return Err(LifecycleError::InvalidPhase);
        }
        let index = seat.index();
        if !self.seats[index].occupied {
            return Err(LifecycleError::SeatVacant(seat));
        }
        if self.seats[index].ready == ready {
            return Ok(Change::Unchanged);
        }
        if matches!(self.phase, Phase::Countdown { .. }) {
            let next_attempt = self.next_attempt()?;
            self.clear_readiness();
            self.round_attempt = next_attempt;
            self.phase = Phase::Waiting;
            return Ok(Change::Changed);
        }

        let starts_countdown =
            ready
                && self.seats.iter().enumerate().all(|(candidate, state)| {
                    state.occupied && (candidate == index || state.ready)
                });
        let ends_at = if starts_countdown {
            Some(Tick(
                self.tick
                    .0
                    .checked_add(COUNTDOWN_TICKS)
                    .ok_or(LifecycleError::TickOverflow)?,
            ))
        } else {
            None
        };
        self.seats[index].ready = ready;
        if let Some(ends_at) = ends_at {
            self.phase = Phase::Countdown { ends_at };
        }
        Ok(Change::Changed)
    }

    pub fn depart(
        &mut self,
        seat: SeatId,
        reason: DepartureReason,
    ) -> Result<Change, LifecycleError> {
        let index = seat.index();
        if !self.seats[index].occupied {
            return Ok(Change::Unchanged);
        }
        match self.phase {
            Phase::Countdown { .. } => {
                let next_attempt = self.next_attempt()?;
                self.seats[index].occupied = false;
                self.clear_readiness();
                self.round_attempt = next_attempt;
                self.phase = Phase::Waiting;
                Ok(Change::Changed)
            }
            Phase::Running => {
                self.seats[index] = SeatState::default();
                self.clear_readiness();
                self.phase = Phase::Interrupted {
                    ended_at: self.tick,
                    reason: reason.interrupt_reason(),
                };
                Ok(Change::Finished)
            }
            Phase::Waiting => {
                self.seats[index] = SeatState::default();
                self.clear_readiness();
                Ok(Change::Changed)
            }
            Phase::Result { .. } | Phase::Interrupted { .. } => {
                self.seats[index] = SeatState::default();
                Ok(Change::Changed)
            }
        }
    }

    pub fn advance_idle_tick(&mut self) -> Result<Change, LifecycleError> {
        match self.phase {
            Phase::Running => return Err(LifecycleError::InvalidPhase),
            Phase::Interrupted { .. } => return Ok(Change::Unchanged),
            Phase::Waiting | Phase::Countdown { .. } | Phase::Result { .. } => {}
        }
        let next_tick = self.next_tick()?;
        let starts = matches!(self.phase, Phase::Countdown { ends_at } if next_tick == ends_at);
        self.tick = next_tick;
        if starts {
            self.clear_readiness();
            self.phase = Phase::Running;
            Ok(Change::Started)
        } else {
            Ok(Change::Changed)
        }
    }

    pub fn complete_running_tick(
        &mut self,
        result: Option<RoundResult>,
    ) -> Result<Change, LifecycleError> {
        if self.phase != Phase::Running {
            return Err(LifecycleError::InvalidPhase);
        }
        let completed_tick = self.next_tick()?;
        self.tick = completed_tick;
        if let Some(result) = result {
            self.phase = Phase::Result {
                ended_at: completed_tick,
                result,
            };
            Ok(Change::Finished)
        } else {
            Ok(Change::Changed)
        }
    }

    pub fn interrupt(&mut self, reason: InterruptReason) -> Change {
        if matches!(self.phase, Phase::Result { .. } | Phase::Interrupted { .. }) {
            return Change::Unchanged;
        }
        self.clear_readiness();
        self.phase = Phase::Interrupted {
            ended_at: self.tick,
            reason,
        };
        Change::Finished
    }

    fn next_tick(&self) -> Result<Tick, LifecycleError> {
        self.tick
            .0
            .checked_add(1)
            .map(Tick)
            .ok_or(LifecycleError::TickOverflow)
    }

    fn next_attempt(&self) -> Result<u64, LifecycleError> {
        self.round_attempt
            .checked_add(1)
            .ok_or(LifecycleError::RoundAttemptOverflow)
    }

    fn clear_readiness(&mut self) {
        self.seats.iter_mut().for_each(|seat| seat.ready = false);
    }
}

#[cfg(test)]
#[path = "room_tests.rs"]
mod tests;

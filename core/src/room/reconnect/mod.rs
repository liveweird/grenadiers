//! Pure two-seat reservation and reconnect-grace facts.

use crate::game::{SeatId, Tick};

pub const RECONNECT_GRACE_TICKS: u64 = 600;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReservationState {
    #[default]
    Vacant,
    Connected,
    Disconnected {
        expires_at: Tick,
    },
    Expired,
}

impl ReservationState {
    #[must_use]
    pub const fn reserved(self) -> bool {
        !matches!(self, Self::Vacant)
    }

    #[must_use]
    pub const fn connected(self) -> bool {
        matches!(self, Self::Connected)
    }

    #[must_use]
    pub const fn resume_deadline(self) -> Option<Tick> {
        match self {
            Self::Disconnected { expires_at } => Some(expires_at),
            Self::Vacant | Self::Connected | Self::Expired => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisconnectStatus {
    Started { expires_at: Tick },
    AlreadyDisconnected { expires_at: Tick },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExpiryFacts {
    at: Tick,
    newly_expired: [bool; 2],
    states: [ReservationState; 2],
}

impl ExpiryFacts {
    #[must_use]
    pub const fn at(self) -> Tick {
        self.at
    }

    #[must_use]
    pub const fn newly_expired(self) -> [bool; 2] {
        self.newly_expired
    }

    #[must_use]
    pub const fn states(self) -> [ReservationState; 2] {
        self.states
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReconnectError {
    NonMonotonicTick { received: Tick, earliest: Tick },
    SeatAlreadyReserved(SeatId),
    SeatVacant(SeatId),
    SeatNotConnected(SeatId),
    SeatNotDisconnected(SeatId),
    SeatExpired(SeatId),
    GraceExpired { seat: SeatId, expires_at: Tick },
    TickOverflow,
}

impl std::fmt::Display for ReconnectError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ReconnectError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReconnectGrace {
    latest_tick: Tick,
    seats: [ReservationState; 2],
}

impl Default for ReconnectGrace {
    fn default() -> Self {
        Self::new()
    }
}

impl ReconnectGrace {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            latest_tick: Tick(0),
            seats: [ReservationState::Vacant; 2],
        }
    }

    #[must_use]
    pub const fn latest_tick(&self) -> Tick {
        self.latest_tick
    }

    #[must_use]
    pub const fn seats(&self) -> &[ReservationState; 2] {
        &self.seats
    }

    #[must_use]
    pub const fn seat(&self, seat: SeatId) -> ReservationState {
        self.seats[seat.index()]
    }

    pub fn occupy(&mut self, seat: SeatId, at: Tick) -> Result<(), ReconnectError> {
        self.check_tick(at)?;
        if self.seat(seat) != ReservationState::Vacant {
            return Err(ReconnectError::SeatAlreadyReserved(seat));
        }
        self.latest_tick = at;
        self.seats[seat.index()] = ReservationState::Connected;
        Ok(())
    }

    pub fn disconnect(
        &mut self,
        seat: SeatId,
        at: Tick,
    ) -> Result<DisconnectStatus, ReconnectError> {
        self.check_tick(at)?;
        match self.seat(seat) {
            ReservationState::Connected => {
                let expires_at = Tick(
                    at.0.checked_add(RECONNECT_GRACE_TICKS)
                        .ok_or(ReconnectError::TickOverflow)?,
                );
                self.latest_tick = at;
                self.seats[seat.index()] = ReservationState::Disconnected { expires_at };
                Ok(DisconnectStatus::Started { expires_at })
            }
            ReservationState::Disconnected { expires_at } => {
                self.latest_tick = at;
                Ok(DisconnectStatus::AlreadyDisconnected { expires_at })
            }
            ReservationState::Vacant => Err(ReconnectError::SeatVacant(seat)),
            ReservationState::Expired => Err(ReconnectError::SeatNotConnected(seat)),
        }
    }

    pub fn resume(&mut self, seat: SeatId, at: Tick) -> Result<(), ReconnectError> {
        self.check_tick(at)?;
        match self.seat(seat) {
            ReservationState::Disconnected { expires_at } if at < expires_at => {
                self.latest_tick = at;
                self.seats[seat.index()] = ReservationState::Connected;
                Ok(())
            }
            ReservationState::Disconnected { expires_at } => {
                Err(ReconnectError::GraceExpired { seat, expires_at })
            }
            ReservationState::Expired => Err(ReconnectError::SeatExpired(seat)),
            ReservationState::Vacant => Err(ReconnectError::SeatVacant(seat)),
            ReservationState::Connected => Err(ReconnectError::SeatNotDisconnected(seat)),
        }
    }

    pub fn expire_due(&mut self, at: Tick) -> Result<ExpiryFacts, ReconnectError> {
        self.check_tick(at)?;
        let newly_expired = self.seats.map(
            |state| matches!(state, ReservationState::Disconnected { expires_at } if at >= expires_at),
        );
        for (state, expires) in self.seats.iter_mut().zip(newly_expired) {
            if expires {
                *state = ReservationState::Expired;
            }
        }
        self.latest_tick = at;
        Ok(ExpiryFacts {
            at,
            newly_expired,
            states: self.seats,
        })
    }

    pub fn release(&mut self, seat: SeatId, at: Tick) -> Result<bool, ReconnectError> {
        self.check_tick(at)?;
        let changed = self.seat(seat) != ReservationState::Vacant;
        self.latest_tick = at;
        self.seats[seat.index()] = ReservationState::Vacant;
        Ok(changed)
    }

    fn check_tick(&self, received: Tick) -> Result<(), ReconnectError> {
        if received < self.latest_tick {
            return Err(ReconnectError::NonMonotonicTick {
                received,
                earliest: self.latest_tick,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

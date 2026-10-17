use super::*;

impl MatchLifecycle {
    pub(super) fn next_tick(&self) -> Result<Tick, MatchError> {
        self.tick
            .0
            .checked_add(1)
            .map(Tick)
            .ok_or(MatchError::TickOverflow)
    }

    pub(super) fn deadline(&self, duration: u64) -> Result<Tick, MatchError> {
        self.tick
            .0
            .checked_add(duration)
            .map(Tick)
            .ok_or(MatchError::TickOverflow)
    }

    pub(super) fn next_attempt(&self) -> Result<u64, MatchError> {
        self.round_attempt
            .checked_add(1)
            .ok_or(MatchError::RoundAttemptOverflow)
    }

    pub(super) fn both_connected(&self) -> bool {
        SeatId::ALL
            .into_iter()
            .all(|seat| self.reconnect.seat(seat).connected())
    }
}

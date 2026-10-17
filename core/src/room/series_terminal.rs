use super::*;

impl MatchSeries {
    pub fn resolve_departures(
        &mut self,
        departures: [Option<DepartureReason>; 2],
        tick: Tick,
    ) -> Result<Option<MatchOutcome>, SeriesError> {
        if let Some(outcome) = self.outcome {
            return Ok(Some(outcome));
        }
        if self.actual_round_number == 0 || departures == [None, None] {
            return Ok(None);
        }
        self.check_tick(tick)?;
        let result = if departures.contains(&Some(DepartureReason::TransportLoss)) {
            MatchResult::Interrupted(InterruptReason::ConnectionLost)
        } else if departures == [Some(DepartureReason::Explicit); 2] {
            MatchResult::Abandoned
        } else if departures[0] == Some(DepartureReason::Explicit) {
            MatchResult::ForfeitWinner(SeatId::Two)
        } else {
            MatchResult::ForfeitWinner(SeatId::One)
        };
        Ok(Some(self.finish(result, tick)?))
    }

    pub fn interrupt(
        &mut self,
        reason: InterruptReason,
        tick: Tick,
    ) -> Result<Option<MatchOutcome>, SeriesError> {
        if let Some(outcome) = self.outcome {
            return Ok(Some(outcome));
        }
        if self.actual_round_number == 0 {
            return Ok(None);
        }
        self.check_tick(tick)?;
        Ok(Some(self.finish(MatchResult::Interrupted(reason), tick)?))
    }

    pub(in crate::room) fn forfeit(
        &mut self,
        winner: SeatId,
        at: Tick,
    ) -> Result<Option<MatchOutcome>, SeriesError> {
        self.terminal_fact(MatchResult::ForfeitWinner(winner), at)
    }

    pub(in crate::room) fn abandon(
        &mut self,
        at: Tick,
    ) -> Result<Option<MatchOutcome>, SeriesError> {
        self.terminal_fact(MatchResult::Abandoned, at)
    }

    fn terminal_fact(
        &mut self,
        result: MatchResult,
        at: Tick,
    ) -> Result<Option<MatchOutcome>, SeriesError> {
        if let Some(outcome) = self.outcome {
            return Ok(Some(outcome));
        }
        if self.actual_round_number == 0 {
            return Ok(None);
        }
        self.check_tick(at)?;
        Ok(Some(self.finish(result, at)?))
    }

    fn finish(&mut self, result: MatchResult, tick: Tick) -> Result<MatchOutcome, SeriesError> {
        let hold_deadline = Tick(
            tick.0
                .checked_add(RESULT_HOLD_TICKS)
                .ok_or(SeriesError::TickOverflow)?,
        );
        let outcome = MatchOutcome {
            ended_at: tick,
            result,
        };
        self.latest_tick = tick;
        self.active = None;
        self.hold_deadline = Some(hold_deadline);
        self.outcome = Some(outcome);
        Ok(outcome)
    }
}

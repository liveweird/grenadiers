//! Pure first-to-three series facts; room orchestration remains owner-controlled.

use crate::game::{RoundAssignment, RoundResult, SeatId, Tick};

use super::{DepartureReason, InterruptReason};

pub const RESULT_HOLD_TICKS: u64 = 180;
pub const WINS_TO_FINISH: u8 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompletedRound {
    epoch: u64,
    assignment: RoundAssignment,
    started_at: Tick,
    ended_at: Tick,
    result: RoundResult,
}

impl CompletedRound {
    #[must_use]
    pub const fn epoch(self) -> u64 {
        self.epoch
    }

    #[must_use]
    pub const fn assignment(self) -> RoundAssignment {
        self.assignment
    }

    #[must_use]
    pub const fn started_at(self) -> Tick {
        self.started_at
    }

    #[must_use]
    pub const fn ended_at(self) -> Tick {
        self.ended_at
    }

    #[must_use]
    pub const fn result(self) -> RoundResult {
        self.result
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchResult {
    ScoreWinner(SeatId),
    ForfeitWinner(SeatId),
    Abandoned,
    Interrupted(InterruptReason),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchOutcome {
    ended_at: Tick,
    result: MatchResult,
}

impl MatchOutcome {
    #[must_use]
    pub const fn ended_at(self) -> Tick {
        self.ended_at
    }

    #[must_use]
    pub const fn result(self) -> MatchResult {
        self.result
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RematchStatus {
    WaitingForOpponent,
    Started,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeriesError {
    ZeroCommandEpoch,
    StaleCommandEpoch { received: u64, last: u64 },
    RoundAlreadyActive,
    NoActiveRound,
    WrongRoundEpoch { received: u64, active: u64 },
    MatchFinished,
    ResultHoldActive { ready_at: Tick },
    NonMonotonicTick { received: Tick, earliest: Tick },
    TickOverflow,
    RoundNumberOverflow,
    RematchUnavailable,
}

impl std::fmt::Display for SeriesError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for SeriesError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActiveRound {
    epoch: u64,
    assignment: RoundAssignment,
    started_at: Tick,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchSeries {
    scores: [u8; 2],
    actual_round_number: u32,
    last_command_epoch: u64,
    latest_tick: Tick,
    active: Option<ActiveRound>,
    latest_completed: Option<CompletedRound>,
    hold_deadline: Option<Tick>,
    outcome: Option<MatchOutcome>,
    rematch_votes: [bool; 2],
}

impl Default for MatchSeries {
    fn default() -> Self {
        Self::new()
    }
}

impl MatchSeries {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            scores: [0, 0],
            actual_round_number: 0,
            last_command_epoch: 0,
            latest_tick: Tick(0),
            active: None,
            latest_completed: None,
            hold_deadline: None,
            outcome: None,
            rematch_votes: [false; 2],
        }
    }

    #[must_use]
    pub const fn scores(&self) -> [u8; 2] {
        self.scores
    }

    #[must_use]
    pub const fn actual_round_number(&self) -> u32 {
        self.actual_round_number
    }

    #[must_use]
    pub const fn last_command_epoch(&self) -> u64 {
        self.last_command_epoch
    }

    #[must_use]
    pub const fn active_round(&self) -> Option<RoundAssignment> {
        match self.active {
            Some(active) => Some(active.assignment),
            None => None,
        }
    }

    #[must_use]
    pub const fn latest_completed_round(&self) -> Option<CompletedRound> {
        self.latest_completed
    }

    #[must_use]
    pub const fn hold_deadline(&self) -> Option<Tick> {
        self.hold_deadline
    }

    #[must_use]
    pub const fn outcome(&self) -> Option<MatchOutcome> {
        self.outcome
    }

    #[must_use]
    pub const fn rematch_votes(&self) -> [bool; 2] {
        self.rematch_votes
    }

    #[must_use]
    pub fn ready_for_next(&self, tick: Tick) -> bool {
        tick >= self.latest_tick
            && self.active.is_none()
            && self.outcome.is_none()
            && self.hold_deadline.is_none_or(|deadline| tick >= deadline)
    }

    pub fn start_round(
        &mut self,
        command_epoch: u64,
        tick: Tick,
    ) -> Result<RoundAssignment, SeriesError> {
        self.check_new_epoch(command_epoch)?;
        if self.outcome.is_some() {
            return Err(SeriesError::MatchFinished);
        }
        if self.active.is_some() {
            return Err(SeriesError::RoundAlreadyActive);
        }
        self.check_tick(tick)?;
        if let Some(ready_at) = self.hold_deadline
            && tick < ready_at
        {
            return Err(SeriesError::ResultHoldActive { ready_at });
        }
        let number = self
            .actual_round_number
            .checked_add(1)
            .and_then(std::num::NonZeroU32::new)
            .ok_or(SeriesError::RoundNumberOverflow)?;
        let assignment = RoundAssignment::new(number);
        self.actual_round_number = number.get();
        self.last_command_epoch = command_epoch;
        self.latest_tick = tick;
        self.active = Some(ActiveRound {
            epoch: command_epoch,
            assignment,
            started_at: tick,
        });
        self.hold_deadline = None;
        Ok(assignment)
    }

    pub fn complete_round(
        &mut self,
        command_epoch: u64,
        result: RoundResult,
        ended_at: Tick,
    ) -> Result<Option<MatchOutcome>, SeriesError> {
        let active = self.active.ok_or(SeriesError::NoActiveRound)?;
        if command_epoch != active.epoch {
            return Err(SeriesError::WrongRoundEpoch {
                received: command_epoch,
                active: active.epoch,
            });
        }
        self.check_tick(ended_at)?;
        let hold_deadline = Tick(
            ended_at
                .0
                .checked_add(RESULT_HOLD_TICKS)
                .ok_or(SeriesError::TickOverflow)?,
        );
        let mut scores = self.scores;
        let outcome = if let RoundResult::Winner(seat) = result {
            let score = &mut scores[seat.index()];
            *score = score
                .checked_add(1)
                .ok_or(SeriesError::RoundNumberOverflow)?;
            (*score == WINS_TO_FINISH).then_some(MatchOutcome {
                ended_at,
                result: MatchResult::ScoreWinner(seat),
            })
        } else {
            None
        };
        self.scores = scores;
        self.latest_tick = ended_at;
        self.active = None;
        self.latest_completed = Some(CompletedRound {
            epoch: active.epoch,
            assignment: active.assignment,
            started_at: active.started_at,
            ended_at,
            result,
        });
        self.hold_deadline = Some(hold_deadline);
        self.outcome = outcome;
        Ok(outcome)
    }

    pub fn set_rematch_vote(
        &mut self,
        seat: SeatId,
        ready: bool,
        now: Tick,
    ) -> Result<RematchStatus, SeriesError> {
        if self.outcome.is_none() {
            return Err(SeriesError::RematchUnavailable);
        }
        self.check_tick(now)?;
        if let Some(ready_at) = self.hold_deadline
            && now < ready_at
        {
            return Err(SeriesError::ResultHoldActive { ready_at });
        }
        self.latest_tick = now;
        self.rematch_votes[seat.index()] = ready;
        if self.rematch_votes.iter().all(|vote| *vote) {
            self.reset_for_rematch();
            Ok(RematchStatus::Started)
        } else {
            Ok(RematchStatus::WaitingForOpponent)
        }
    }

    fn reset_for_rematch(&mut self) {
        self.scores = [0, 0];
        self.actual_round_number = 0;
        self.active = None;
        self.latest_completed = None;
        self.hold_deadline = None;
        self.outcome = None;
        self.rematch_votes = [false; 2];
    }

    fn check_new_epoch(&self, received: u64) -> Result<(), SeriesError> {
        if received == 0 {
            return Err(SeriesError::ZeroCommandEpoch);
        }
        if received <= self.last_command_epoch {
            return Err(SeriesError::StaleCommandEpoch {
                received,
                last: self.last_command_epoch,
            });
        }
        Ok(())
    }

    fn check_tick(&self, received: Tick) -> Result<(), SeriesError> {
        if received < self.latest_tick {
            return Err(SeriesError::NonMonotonicTick {
                received,
                earliest: self.latest_tick,
            });
        }
        Ok(())
    }
}

#[path = "series_terminal.rs"]
mod terminal;

#[cfg(test)]
#[path = "series_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "series_terminal_tests.rs"]
mod terminal_tests;

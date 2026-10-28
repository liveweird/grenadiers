use super::{CompletedRound, MatchOutcome, MatchResult};
use crate::game::{RoundAssignment, RoundResult, Tick};

pub(in crate::room) const fn completed_round(
    epoch: u64,
    assignment: RoundAssignment,
    started_at: Tick,
    ended_at: Tick,
    result: RoundResult,
) -> CompletedRound {
    CompletedRound {
        epoch,
        assignment,
        started_at,
        ended_at,
        result,
    }
}

pub(in crate::room) const fn match_outcome(ended_at: Tick, result: MatchResult) -> MatchOutcome {
    MatchOutcome { ended_at, result }
}

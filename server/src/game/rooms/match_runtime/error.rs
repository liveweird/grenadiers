use grenadiers_core::room::{MatchError, series::SeriesError};

use super::super::input::InputError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) enum MatchRuntimeError {
    StaleEpoch,
    StaleSettings,
    SettingsLocked,
    VacantSeat,
    InvalidPhase,
    RateLimited,
    DuplicateAction,
    CapacityReached,
    InvalidInterrupt,
    Publication,
}

pub(super) fn map_input_error(error: InputError) -> MatchRuntimeError {
    match error {
        InputError::RateLimited => MatchRuntimeError::RateLimited,
        InputError::DuplicateAction => MatchRuntimeError::DuplicateAction,
        InputError::TickOverflow => MatchRuntimeError::CapacityReached,
    }
}

pub(super) fn map_lifecycle_error(error: MatchError) -> MatchRuntimeError {
    match error {
        MatchError::InvalidInterruptReason => MatchRuntimeError::InvalidInterrupt,
        MatchError::TickOverflow
        | MatchError::RoundAttemptOverflow
        | MatchError::Series(SeriesError::TickOverflow | SeriesError::RoundNumberOverflow) => {
            MatchRuntimeError::CapacityReached
        }
        _ => MatchRuntimeError::InvalidPhase,
    }
}

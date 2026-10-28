use grenadiers_core::room::RosterError;

use crate::game::rooms::input::InputError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) enum RosterRuntimeError {
    UnsupportedConfiguration,
    StaleEpoch,
    StaleSettings,
    SettingsLocked,
    VacantSeat,
    InvalidPhase,
    RateLimited,
    DuplicateAction,
    CapacityReached,
    Publication,
    SeedUnavailable,
}

pub(super) fn lifecycle(error: RosterError) -> RosterRuntimeError {
    match error {
        RosterError::TickOverflow
        | RosterError::RoundAttemptOverflow
        | RosterError::RoundNumberOverflow
        | RosterError::ScoreOverflow => RosterRuntimeError::CapacityReached,
        RosterError::SeatNeverAdmitted(_)
        | RosterError::SeatNotConnected(_)
        | RosterError::SeatNotReserved(_) => RosterRuntimeError::VacantSeat,
        _ => RosterRuntimeError::InvalidPhase,
    }
}

pub(super) fn input(error: InputError) -> RosterRuntimeError {
    match error {
        InputError::UnknownSeat => RosterRuntimeError::VacantSeat,
        InputError::RateLimited => RosterRuntimeError::RateLimited,
        InputError::DuplicateAction => RosterRuntimeError::DuplicateAction,
        InputError::TickOverflow => RosterRuntimeError::CapacityReached,
    }
}

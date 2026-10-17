mod admission;
mod driver;
mod runner;
mod types;

pub(super) use types::{
    ClosingReason, MatchAdmission, MatchOwnerHandle, MatchOwnerNotice, PreparedMatchOwner,
    ShutdownState,
};

#[cfg(test)]
mod admission_tests;
#[cfg(test)]
mod departure_tests;
#[cfg(test)]
mod round_tests;
#[cfg(test)]
mod runner_tests;
#[cfg(test)]
mod test_support;

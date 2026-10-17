mod admission;
mod driver;
mod runner;
mod types;

#[cfg(test)]
pub(super) use types::ShutdownState;
pub(crate) use types::{ClosingReason, MatchAdmission, MatchOwnerNotice};
pub(super) use types::{MatchOwnerHandle, PreparedMatchOwner};

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

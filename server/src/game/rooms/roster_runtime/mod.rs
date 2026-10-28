//! Transport-independent runtime for configured two-to-four-seat rooms.

mod admission;
mod error;
mod owner_helpers;
mod publication;
mod round;
mod runtime;
mod types;

#[cfg(test)]
mod replay_inspection;

pub(in crate::game::rooms) use admission::PreparedRosterAdmission;
pub(in crate::game::rooms) use error::RosterRuntimeError;
#[cfg(test)]
pub(in crate::game::rooms) use replay_inspection::RosterRuntimeInspection;
pub(in crate::game::rooms) use runtime::RosterRuntime;
pub(in crate::game::rooms) use types::RosterIdentity;
pub(crate) use types::{RosterAction, RosterNotice};

#[cfg(test)]
mod boundary_tests;
#[cfg(test)]
mod replacement_tests;
#[cfg(test)]
mod tests;

//! Transport-independent runtime for configured two-to-four-seat rooms.

mod admission;
mod environment;
mod error;
mod owner_helpers;
mod publication;
mod publication_v15;
mod publication_v16;
mod round;
mod runtime;
mod runtime_publication;
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
mod environment_recovery_tests;
#[cfg(test)]
mod environment_settings_tests;
#[cfg(test)]
mod lair_settings_tests;
#[cfg(test)]
mod replacement_tests;
#[cfg(test)]
mod tests;

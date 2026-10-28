//! Dormant, transport-independent runtime for configured two-to-four-seat rooms.

mod admission;
mod error;
mod owner_helpers;
mod publication;
mod round;
mod runtime;
mod types;

pub(in crate::game::rooms) use admission::PreparedRosterAdmission;
pub(in crate::game::rooms) use error::RosterRuntimeError;
pub(in crate::game::rooms) use runtime::RosterRuntime;
pub(in crate::game::rooms) use types::{
    RosterAction, RosterIdentity, RosterNotice, RosterSeatFrames,
};

#[cfg(test)]
mod boundary_tests;
#[cfg(test)]
mod tests;

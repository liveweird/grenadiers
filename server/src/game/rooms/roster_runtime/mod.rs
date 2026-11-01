//! Transport-independent runtime for configured two-to-four-seat rooms.

mod admission;
mod admission_commit;
mod admission_projection;
mod admission_spike;
mod environment;
mod environment_spike;
mod error;
mod owner_helpers;
mod publication;
mod publication_v15;
mod publication_v16;
mod publication_v17;
mod publication_v18;
mod publication_v19;
mod round;
mod runtime;
mod runtime_helpers;
mod runtime_publication;
mod seat_wire;
mod types;

#[cfg(test)]
mod test_scene;

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
mod respawn_settings_tests;
#[cfg(test)]
mod spike_settings_tests;
#[cfg(test)]
mod terminal_transport_tests;
#[cfg(test)]
mod tests;

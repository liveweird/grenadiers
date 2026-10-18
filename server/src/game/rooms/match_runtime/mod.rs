mod error;
mod frames;
mod publication;
mod runtime;

pub(super) use error::MatchRuntimeError;
pub(crate) use frames::SeatFrames;
pub(super) use runtime::MatchRuntime;
pub(super) use runtime::admission::PreparedRuntimeAdmission;
pub(super) use runtime::admission::ResumeKind;
#[cfg(test)]
pub(super) use runtime::replay::RuntimeInspection;
pub(crate) use runtime::{MatchAction, MatchNotice};

#[cfg(test)]
#[path = "input_tests.rs"]
mod input_tests;
#[cfg(test)]
#[path = "lifecycle_tests.rs"]
mod lifecycle_tests;
#[cfg(test)]
#[path = "performance_tests.rs"]
mod performance_tests;
#[cfg(test)]
#[path = "publication_tests.rs"]
mod publication_tests;
#[cfg(test)]
#[path = "settings_tests.rs"]
mod settings_tests;
#[cfg(test)]
#[path = "test_support.rs"]
mod test_support;

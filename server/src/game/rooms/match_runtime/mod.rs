mod error;
mod publication;
mod runtime;

pub(super) use error::MatchRuntimeError;
pub(super) use runtime::MatchRuntime;
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
#[path = "publication_tests.rs"]
mod publication_tests;
#[cfg(test)]
#[path = "test_support.rs"]
mod test_support;

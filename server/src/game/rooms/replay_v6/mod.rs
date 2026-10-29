mod record;
mod recording;
mod runner;
mod validation;

#[allow(unused_imports)]
pub(in crate::game::rooms) use record::{
    CpuProvenance, Journal, Operation, ReplayAction, ReplayHeader, ReplayStatus,
};
#[allow(unused_imports)]
pub(in crate::game::rooms) use recording::RecordingRosterRuntime;
#[allow(unused_imports)]
pub(in crate::game::rooms) use runner::{replay, verify_exact};

#[cfg(test)]
mod environment_acceptance_tests;
#[cfg(test)]
mod tests;

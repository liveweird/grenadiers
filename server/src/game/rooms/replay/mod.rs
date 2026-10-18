mod record;
#[cfg(test)]
mod runner;

#[cfg(test)]
pub(in crate::game::rooms) use record::M5_LIFECYCLE_PROFILE;
#[cfg(test)]
use record::ReplayStatus;
pub(in crate::game::rooms) use record::{Header, Journal, Operation, ReplayAction};
#[cfg(test)]
use runner::{ReplayError, replay};

#[cfg(test)]
mod capture_tests;
#[cfg(test)]
mod settings_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod shield_tests;

#[cfg(test)]
mod golden_flame_tests;

#[cfg(test)]
mod short_fuse_tests;

#[cfg(test)]
mod short_flame_tests;

#[cfg(test)]
mod long_fuse_tests;

#[cfg(test)]
mod reverse_controls_tests;

#[cfg(test)]
mod diarrhea_tests;

#[cfg(test)]
mod slow_foot_tests;

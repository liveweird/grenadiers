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

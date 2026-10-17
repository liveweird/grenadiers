mod record;
#[cfg(test)]
mod runner;

#[cfg(test)]
use record::ReplayStatus;
pub(in crate::game::rooms) use record::{Header, Journal, Operation, ReplayAction};
#[cfg(test)]
use runner::{ReplayError, replay};

#[cfg(test)]
mod capture_tests;
#[cfg(test)]
mod tests;

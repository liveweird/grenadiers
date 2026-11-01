mod controller;
mod worker;

#[cfg(test)]
pub(in crate::game::rooms) use controller::admission::AdmissionTrace;
pub(super) use controller::{AcceptedPlan, CpuController, contact::ContactStatus};
pub(super) use worker::CpuPlanner;
#[cfg(test)]
pub(in crate::game::rooms) use worker::PumpReport;

#[cfg(test)]
mod controller_reliable_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod worker_v27_tests;

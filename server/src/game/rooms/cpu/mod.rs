mod controller;
mod worker;

#[cfg(test)]
pub(in crate::game::rooms) use controller::admission::AdmissionTrace;
pub(super) use controller::{
    AcceptedPlan, ContinuationService, CpuController, TeleporterService, contact::ContactStatus,
};
#[cfg(test)]
pub(in crate::game::rooms) use controller::{ContinuationTrace, RouteTrace};
pub(super) use worker::CpuPlanner;
#[cfg(test)]
pub(in crate::game::rooms) use worker::{PlannedResultWitness, PlanningBasis, PumpReport};

#[cfg(test)]
mod controller_reliable_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod worker_teleporter_tests;
#[cfg(test)]
mod worker_v27_tests;
#[cfg(test)]
mod worker_v65_tests;
#[cfg(test)]
mod worker_v66_tests;

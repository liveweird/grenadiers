use std::sync::{Arc, atomic::AtomicBool, mpsc::Receiver};

use grenadiers_core::cpu::policy::UtilityMemory;
use grenadiers_core::cpu::survival::{CommittedPulse, CommittedRoute};
use grenadiers_core::cpu::{CpuRng, DecisionId, Intent, ManualIntent, MobContactLease};
use grenadiers_core::game::Tick;

use super::super::worker::PlanResult;
use super::continuation_types::ContinuationClock;

pub(super) struct Pending {
    pub(super) decision: DecisionId,
    pub(super) receiver: Receiver<PlanResult>,
    pub(super) live: Arc<AtomicBool>,
    pub(super) completed: Option<PlanResult>,
    pub(super) refuge_handoff: Option<grenadiers_core::cpu::survival::TerminalRefugeHandoff>,
    pub(super) continuation: Option<ContinuationClock>,
}

#[derive(Clone, Copy)]
pub(super) struct StopAdmission {
    pub(super) tick: Tick,
}

pub(in crate::game::rooms) struct AcceptedPlan {
    pub(in crate::game::rooms) decision: DecisionId,
    pub(in crate::game::rooms) intent: Intent,
    pub(in crate::game::rooms) hold_ticks: u8,
    pub(super) next_memory: UtilityMemory,
    pub(super) next_rng: CpuRng,
    pub(super) committed_pulse: Option<CommittedPulse>,
    pub(super) committed_route: Option<CommittedRoute>,
    pub(super) contact: Option<MobContactLease>,
    pub(super) certified_manual: Option<ManualIntent>,
    pub(super) escape_origin: bool,
    pub(super) ordinary_navigation: bool,
    pub(super) refuge_handoff: bool,
}

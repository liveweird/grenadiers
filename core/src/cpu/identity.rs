use crate::game::Tick;

use super::{CpuMemory, Intent, WorkBudget, WorkError};

/// Identity of one physical arena, independent of transport commands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorldId {
    pub room_instance: u64,
    pub physical_round: u64,
}

/// Identity checked before a CPU decision may perform policy work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecisionId {
    pub world: WorldId,
    pub command_epoch: u64,
    pub observation_tick: Tick,
    pub checked_request_serial: u64,
}

/// Caller-current identity against which a completed decision is admitted.
///
/// Keeping this separate means a stale result cannot validate its own identity.
/// The server supplies the serial only after checking the current CPU request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdmissionContext {
    pub world: WorldId,
    pub command_epoch: u64,
    pub observation_tick: Tick,
    pub checked_request_serial: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RejectionReason {
    WrongWorld,
    StaleCommandEpoch,
    FutureCommandEpoch,
    StaleObservation,
    FutureObservation,
    DuplicateRequest,
    StaleRequest,
    FutureRequest,
    WorkExhausted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecisionAdmission {
    Accepted,
    Rejected(RejectionReason),
}

impl DecisionAdmission {
    /// Rejections always fail closed: stop and request no manual action.
    #[must_use]
    pub const fn fallback_intent(self) -> Option<Intent> {
        match self {
            Self::Accepted => None,
            Self::Rejected(_) => Some(Intent::STOP),
        }
    }
}

/// Checks identity and charges one deterministic admission work unit.
///
/// A changed command epoch (for example after reconnect) does not reset memory
/// or RNG. A changed physical world must be handled explicitly with
/// [`CpuMemory::reset_for`].
pub fn admit_decision(
    memory: &mut CpuMemory,
    current: AdmissionContext,
    candidate: DecisionId,
    work: &mut WorkBudget,
) -> DecisionAdmission {
    if current.world != memory.key().world || candidate.world != current.world {
        return DecisionAdmission::Rejected(RejectionReason::WrongWorld);
    }
    if candidate.command_epoch < current.command_epoch {
        return DecisionAdmission::Rejected(RejectionReason::StaleCommandEpoch);
    }
    if candidate.command_epoch > current.command_epoch {
        return DecisionAdmission::Rejected(RejectionReason::FutureCommandEpoch);
    }
    if candidate.observation_tick < current.observation_tick {
        return DecisionAdmission::Rejected(RejectionReason::StaleObservation);
    }
    if candidate.observation_tick > current.observation_tick {
        return DecisionAdmission::Rejected(RejectionReason::FutureObservation);
    }
    if candidate.checked_request_serial < current.checked_request_serial {
        return DecisionAdmission::Rejected(RejectionReason::StaleRequest);
    }
    if candidate.checked_request_serial > current.checked_request_serial {
        return DecisionAdmission::Rejected(RejectionReason::FutureRequest);
    }
    if let Some(previous) = memory.last_admitted() {
        if candidate.command_epoch < previous.command_epoch {
            return DecisionAdmission::Rejected(RejectionReason::StaleCommandEpoch);
        }
        if candidate.observation_tick < previous.observation_tick {
            return DecisionAdmission::Rejected(RejectionReason::StaleObservation);
        }
        if candidate.command_epoch == previous.command_epoch {
            if candidate.checked_request_serial == previous.checked_request_serial {
                return DecisionAdmission::Rejected(RejectionReason::DuplicateRequest);
            }
            if candidate.checked_request_serial < previous.checked_request_serial {
                return DecisionAdmission::Rejected(RejectionReason::StaleRequest);
            }
        }
    }
    if work.try_take(1) == Err(WorkError::Exhausted) {
        return DecisionAdmission::Rejected(RejectionReason::WorkExhausted);
    }
    memory.set_last_admitted(candidate);
    DecisionAdmission::Accepted
}

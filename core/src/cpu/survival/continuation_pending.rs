//! Linear pending-publication clock for inactive continuation admission.

use crate::cpu::routing::{ContinuationCertificate, RouteFailure};
use crate::cpu::{DecisionId, MemoryKey, Observation, WorkBudget};
use crate::game::Tick;

use super::continuation_execution::{
    ContinuationPublication, PreparedContinuation, prepare_admission,
};

pub(crate) struct PendingContinuation {
    certificate: Option<Box<ContinuationCertificate>>,
    decision: DecisionId,
    key: MemoryKey,
    next_tick: Tick,
    next_sequence: u64,
    stage: u8,
    revoked: bool,
}

impl PendingContinuation {
    pub(crate) fn new(
        certificate: Box<ContinuationCertificate>,
        decision: DecisionId,
        key: MemoryKey,
        basis_sequence: u64,
    ) -> Result<Self, RouteFailure> {
        if decision.world != certificate.key().world
            || decision.observation_tick != certificate.basis_tick()
            || key != certificate.key()
        {
            return Err(RouteFailure::IdentityMismatch);
        }
        Ok(Self {
            next_tick: Tick(
                decision
                    .observation_tick
                    .0
                    .checked_add(3)
                    .ok_or(RouteFailure::InvalidRequest)?,
            ),
            next_sequence: basis_sequence
                .checked_add(1)
                .ok_or(RouteFailure::InvalidRequest)?,
            certificate: Some(certificate),
            decision,
            key,
            stage: 0,
            revoked: false,
        })
    }

    pub(crate) fn observe(
        &mut self,
        current: &Observation,
        publication: ContinuationPublication,
        work: &mut WorkBudget,
    ) -> Result<Option<PreparedContinuation>, RouteFailure> {
        if self.revoked
            || self.stage > 1
            || publication.decision != self.decision
            || publication.key != self.key
            || publication.sequence != self.next_sequence
            || publication.tick != self.next_tick
            || current.parts().tick != self.next_tick
            || current.parts().world != self.key.world
            || current.parts().recipient != self.key.cpu_seat
            || work.try_take(1).is_err()
        {
            self.revoked = true;
            return Err(RouteFailure::IdentityMismatch);
        }
        if self.stage == 0 {
            let Some(next_tick) = self.next_tick.0.checked_add(3).map(Tick) else {
                self.revoked = true;
                return Err(RouteFailure::InvalidRequest);
            };
            let Some(next_sequence) = self.next_sequence.checked_add(1) else {
                self.revoked = true;
                return Err(RouteFailure::InvalidRequest);
            };
            self.stage = 1;
            self.next_tick = next_tick;
            self.next_sequence = next_sequence;
            return Ok(None);
        }
        let certificate = self
            .certificate
            .take()
            .ok_or(RouteFailure::IdentityMismatch)?;
        self.stage = 2;
        match prepare_admission(certificate, current, publication, work) {
            Ok(prepared) => Ok(Some(prepared)),
            Err(failure) => {
                self.revoked = true;
                Err(failure)
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn fork_for_test(&self) -> Self {
        Self {
            certificate: self.certificate.clone(),
            decision: self.decision,
            key: self.key,
            next_tick: self.next_tick,
            next_sequence: self.next_sequence,
            stage: self.stage,
            revoked: self.revoked,
        }
    }

    #[cfg(test)]
    pub(crate) fn shorten_horizon_for_test(&mut self, survive_until: Tick) {
        self.certificate
            .as_mut()
            .expect("test pending certificate")
            .set_survive_until_for_test(survive_until);
    }
}

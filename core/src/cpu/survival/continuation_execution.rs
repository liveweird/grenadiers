//! Inactive atomic executor for charged continuation grants.

use std::sync::Arc;

use crate::cpu::routing::{
    CONTINUATION_RENEWAL_ALLOWANCE, ContinuationCertificate, PreparedFrame, PreparedGrant,
    RouteFailure,
};
use crate::cpu::{DecisionId, MemoryKey, MovementIntent, Observation, WorkBudget};
use crate::game::Tick;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ContinuationPublication {
    pub(crate) decision: DecisionId,
    pub(crate) key: MemoryKey,
    pub(crate) sequence: u64,
    pub(crate) tick: Tick,
}

#[derive(Debug)]
struct EpisodeToken;

#[derive(Clone, Debug)]
pub(crate) struct ContinuationCommand {
    token: Arc<EpisodeToken>,
    movement: MovementIntent,
    admission_tick: Tick,
    input_index: u8,
}

impl ContinuationCommand {
    pub(crate) const fn movement(&self) -> MovementIntent {
        self.movement
    }

    pub(crate) const fn admission_tick(&self) -> Tick {
        self.admission_tick
    }
}

#[derive(Clone, Debug)]
pub(crate) enum ContinuationStatus {
    Continue,
    Wait,
    Submit(ContinuationCommand),
    RenewalRequired,
    Completed,
    Invalidated,
}

pub(crate) struct PreparedContinuation {
    token: Arc<EpisodeToken>,
    certificate: Box<ContinuationCertificate>,
    publication: ContinuationPublication,
    frame: Box<PreparedGrant>,
}

pub(crate) struct PreparedRenewal {
    token: Arc<EpisodeToken>,
    publication: ContinuationPublication,
    kind: PreparedRenewalKind,
}

enum PreparedRenewalKind {
    Grant(Box<PreparedGrant>),
    Complete,
}

pub(crate) struct CommittedContinuation {
    certificate: Box<ContinuationCertificate>,
    token: Arc<EpisodeToken>,
    publication: ContinuationPublication,
    frame: Box<PreparedGrant>,
    next_input: usize,
    movement: MovementIntent,
    revoked: bool,
    completed: bool,
    pending: Option<ContinuationPublication>,
}

pub(super) fn prepare_admission(
    certificate: Box<ContinuationCertificate>,
    current: &Observation,
    publication: ContinuationPublication,
    work: &mut WorkBudget,
) -> Result<PreparedContinuation, RouteFailure> {
    if publication.decision.world != current.parts().world
        || publication.decision.world != certificate.key().world
        || publication.key != certificate.key()
        || publication.decision.observation_tick != certificate.basis_tick()
        || publication.tick != current.parts().tick
        || current.parts().tick != certificate.admission_tick()
    {
        return Err(RouteFailure::IdentityMismatch);
    }
    let frame = prepare_checked(&certificate, current, work)?;
    let PreparedFrame::Grant(frame) = frame else {
        return Err(RouteFailure::InvalidRequest);
    };
    Ok(PreparedContinuation {
        token: Arc::new(EpisodeToken),
        certificate,
        publication,
        frame,
    })
}

impl PreparedContinuation {
    pub(crate) fn command(&self) -> ContinuationCommand {
        command(&self.token, &self.frame, 0)
    }

    pub(crate) fn commit(
        self,
        accepted: ContinuationCommand,
    ) -> Result<CommittedContinuation, RouteFailure> {
        if !accepted.matches(&self.command()) {
            return Err(RouteFailure::IdentityMismatch);
        }
        Ok(CommittedContinuation {
            certificate: self.certificate,
            token: self.token,
            publication: self.publication,
            movement: accepted.movement,
            frame: self.frame,
            next_input: 1,
            revoked: false,
            completed: false,
            pending: None,
        })
    }
}

impl PreparedRenewal {
    pub(crate) fn command(&self) -> ContinuationCommand {
        match &self.kind {
            PreparedRenewalKind::Grant(frame) => command(&self.token, frame, 0),
            PreparedRenewalKind::Complete => ContinuationCommand {
                token: Arc::clone(&self.token),
                movement: MovementIntent::Stop,
                admission_tick: self.publication.tick,
                input_index: 0,
            },
        }
    }

    const fn publication(&self) -> ContinuationPublication {
        self.publication
    }

    fn token(&self) -> &Arc<EpisodeToken> {
        &self.token
    }

    #[cfg(test)]
    pub(crate) fn movements_for_test(&self) -> Option<[MovementIntent; 3]> {
        match &self.kind {
            PreparedRenewalKind::Grant(frame) => Some(frame.movements()),
            PreparedRenewalKind::Complete => None,
        }
    }
}

impl CommittedContinuation {
    #[cfg(test)]
    pub(crate) fn fork_for_test(&self) -> Self {
        Self {
            certificate: self.certificate.clone(),
            token: Arc::clone(&self.token),
            publication: self.publication,
            frame: self.frame.clone(),
            next_input: self.next_input,
            movement: self.movement,
            revoked: self.revoked,
            completed: self.completed,
            pending: self.pending,
        }
    }

    #[cfg(test)]
    pub(crate) fn shorten_horizon_for_test(&mut self, survive_until: Tick) {
        self.certificate.set_survive_until_for_test(survive_until);
    }

    pub(crate) fn prepare_grant(
        &mut self,
        current: &Observation,
        publication: ContinuationPublication,
        work: &mut WorkBudget,
    ) -> Result<PreparedRenewal, RouteFailure> {
        let expected_tick = self
            .frame
            .publication()
            .0
            .checked_add(3)
            .map(Tick)
            .ok_or(RouteFailure::InvalidRequest)?;
        let expected_sequence = self
            .publication
            .sequence
            .checked_add(1)
            .ok_or(RouteFailure::InvalidRequest)?;
        if self.revoked
            || self.completed
            || self.pending.is_some()
            || self.next_input != 3
            || current.parts().tick != expected_tick
            || publication.tick != expected_tick
            || publication.decision != self.publication.decision
            || publication.key != self.publication.key
            || publication.sequence != expected_sequence
        {
            self.revoked = true;
            return Err(RouteFailure::IdentityMismatch);
        }
        let frame = match prepare_checked(&self.certificate, current, work) {
            Ok(frame) => frame,
            Err(failure) => {
                self.revoked = true;
                return Err(failure);
            }
        };
        self.pending = Some(publication);
        Ok(PreparedRenewal {
            token: Arc::clone(&self.token),
            publication,
            kind: match frame {
                PreparedFrame::Grant(frame) => PreparedRenewalKind::Grant(frame),
                PreparedFrame::Complete => PreparedRenewalKind::Complete,
            },
        })
    }

    pub(crate) fn poll_command(
        &mut self,
        current_tick: Tick,
        command_epoch: u64,
    ) -> ContinuationStatus {
        if self.revoked
            || current_tick >= self.certificate.survive_until()
            || command_epoch != self.publication.decision.command_epoch
        {
            self.revoked = true;
            return ContinuationStatus::Invalidated;
        }
        if self.completed {
            return ContinuationStatus::Completed;
        }
        if self.next_input < 3 {
            let expected = command(&self.token, &self.frame, self.next_input);
            return match current_tick.cmp(&expected.admission_tick) {
                std::cmp::Ordering::Less => self.active_status(),
                std::cmp::Ordering::Equal => ContinuationStatus::Submit(expected),
                std::cmp::Ordering::Greater => {
                    self.revoked = true;
                    ContinuationStatus::Invalidated
                }
            };
        }
        let renewal_tick = Tick(self.frame.publication().0.saturating_add(3));
        match current_tick.cmp(&renewal_tick) {
            std::cmp::Ordering::Less => self.active_status(),
            std::cmp::Ordering::Equal => ContinuationStatus::RenewalRequired,
            std::cmp::Ordering::Greater => {
                self.revoked = true;
                ContinuationStatus::Invalidated
            }
        }
    }

    pub(crate) fn commit_command(&mut self, accepted: ContinuationCommand) -> bool {
        if self.revoked || self.completed || self.next_input >= 3 {
            self.revoked = true;
            return false;
        }
        let expected = command(&self.token, &self.frame, self.next_input);
        if !accepted.matches(&expected) {
            self.revoked = true;
            return false;
        }
        self.next_input += 1;
        self.movement = accepted.movement;
        true
    }

    pub(crate) fn commit_grant(
        &mut self,
        prepared: PreparedRenewal,
        accepted: ContinuationCommand,
    ) -> bool {
        let expected_publication = self.publication.sequence.checked_add(1);
        if self.revoked
            || self.completed
            || self.next_input != 3
            || self.pending != Some(prepared.publication())
            || expected_publication != Some(prepared.publication().sequence)
            || !Arc::ptr_eq(&self.token, prepared.token())
            || !accepted.matches(&prepared.command())
        {
            self.revoked = true;
            return false;
        }
        self.publication = prepared.publication();
        self.pending = None;
        match prepared.kind {
            PreparedRenewalKind::Grant(frame) => {
                self.frame = frame;
                self.next_input = 1;
                self.movement = accepted.movement;
            }
            PreparedRenewalKind::Complete => {
                self.movement = MovementIntent::Stop;
                self.completed = true;
            }
        }
        true
    }

    pub(crate) fn revoke(&mut self) {
        self.revoked = true;
        self.movement = MovementIntent::Stop;
    }

    fn active_status(&self) -> ContinuationStatus {
        match self.movement {
            MovementIntent::Stop => ContinuationStatus::Wait,
            MovementIntent::Direction(_) => ContinuationStatus::Continue,
        }
    }
}

fn prepare_checked(
    certificate: &ContinuationCertificate,
    current: &Observation,
    work: &mut WorkBudget,
) -> Result<PreparedFrame, RouteFailure> {
    let mut bounded = WorkBudget::new(work.remaining().min(CONTINUATION_RENEWAL_ALLOWANCE));
    let result = (|| {
        if !super::continuation_compatibility::actor_compatible_charged(
            certificate.basis(),
            current,
            certificate.evolution(),
            &mut bounded,
        ) {
            return Err(RouteFailure::NoRoute);
        }
        let frame = certificate.prepare_frame(current, &mut bounded)?;
        if let PreparedFrame::Grant(frame) = &frame
            && !super::continuation_compatibility::rivals_allow_segments_charged(
                current,
                frame.segments(),
                &mut bounded,
            )
        {
            return Err(RouteFailure::NoRoute);
        }
        Ok(frame)
    })();
    work.try_take(bounded.spent())
        .map_err(|_| RouteFailure::WorkExhausted)?;
    result
}

fn command(token: &Arc<EpisodeToken>, frame: &PreparedGrant, index: usize) -> ContinuationCommand {
    ContinuationCommand {
        token: Arc::clone(token),
        movement: frame.movements()[index],
        admission_tick: Tick(frame.publication().0.saturating_add(index as u64)),
        input_index: index as u8,
    }
}

impl ContinuationCommand {
    fn matches(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.token, &other.token)
            && self.movement == other.movement
            && self.admission_tick == other.admission_tick
            && self.input_index == other.input_index
    }
}

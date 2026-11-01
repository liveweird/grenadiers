//! Charged renewal frames for inactive continuation execution.

use crate::cpu::forecast::TickWindow;
use crate::cpu::lair_contact::{MobContactEnvelope, teleport};
use crate::cpu::{MovementIntent, Observation, ObservedActor, WorkBudget};
use crate::game::{MovementSegment, Tick};

use super::continuation_types::{ContinuationCertificate, ContinuationCheckpoint, StopBranch};
use super::{RouteFailure, storm};

pub(crate) const CONTINUATION_RENEWAL_ALLOWANCE: u32 = 8_192;
const GRANT_INPUTS: usize = 3;
const SHORT_WINDOW_UPDATES: u64 = 14;

#[derive(Clone)]
pub(crate) struct PreparedGrant {
    publication: Tick,
    checkpoints: [ContinuationCheckpoint; GRANT_INPUTS],
}

#[derive(Clone)]
pub(crate) enum PreparedFrame {
    Grant(Box<PreparedGrant>),
    Complete,
}

impl PreparedGrant {
    pub(crate) const fn publication(&self) -> Tick {
        self.publication
    }

    pub(crate) fn movements(&self) -> [MovementIntent; GRANT_INPUTS] {
        self.checkpoints
            .each_ref()
            .map(|checkpoint| checkpoint.movement)
    }

    pub(crate) fn segments(&self) -> impl Iterator<Item = MovementSegment> + '_ {
        self.checkpoints
            .iter()
            .flat_map(|checkpoint| checkpoint.segments.iter().copied())
    }
}

impl ContinuationCertificate {
    pub(crate) const fn admission_tick(&self) -> Tick {
        self.admission_checkpoint().tick
    }
}

impl ContinuationCertificate {
    pub(crate) fn prepare_frame(
        &self,
        current: &Observation,
        work: &mut WorkBudget,
    ) -> Result<PreparedFrame, RouteFailure> {
        let publication = current.parts().tick;
        charge_index_scans(self, work)?;
        let expected = self
            .checkpoint_at(publication)
            .ok_or(RouteFailure::InvalidRequest)?;
        if publication >= self.survive_until()
            || !self
                .evolution()
                .continuation_scene_matches(self.basis(), current, work)
        {
            return Err(RouteFailure::NoRoute);
        }
        charge_len(current.parts().actors.len(), work)?;
        if !self
            .contact()
            .contains(current, self.survive_until(), work)
            .map_err(super::pulse_support::contact_failure)?
        {
            return Err(RouteFailure::NoRoute);
        }
        validate_expected(self.basis(), current, expected, work)?;
        let fresh = fresh_contact(current, work)?;
        let safe_until = Tick(
            publication
                .0
                .checked_add(SHORT_WINDOW_UPDATES)
                .ok_or(RouteFailure::InvalidRequest)?,
        );
        if safe_until > self.survive_until() {
            return Err(RouteFailure::InvalidRequest);
        }
        if !teleport::is_clear(&fresh, current, safe_until, 3, work)
            .map_err(super::pulse_support::contact_failure)?
        {
            return Err(RouteFailure::NoRoute);
        }

        let future = self.future_checkpoints(publication)?;
        if future.is_empty() {
            validate_complete_stop(self, current, &fresh, publication, work)?;
            return Ok(PreparedFrame::Complete);
        }
        validate_fresh_paths(self, current, &fresh, &future, work)?;
        let checkpoints: [ContinuationCheckpoint; GRANT_INPUTS] = future
            .into_iter()
            .take(GRANT_INPUTS)
            .cloned()
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| RouteFailure::InvalidRequest)?;
        if checkpoints
            .last()
            .is_none_or(|checkpoint| checkpoint.tick >= self.survive_until())
        {
            return Err(RouteFailure::InvalidRequest);
        }
        Ok(PreparedFrame::Grant(Box::new(PreparedGrant {
            publication,
            checkpoints,
        })))
    }

    fn checkpoint_at(&self, tick: Tick) -> Option<&ContinuationCheckpoint> {
        if tick == self.admission_checkpoint().tick {
            return Some(self.admission_checkpoint());
        }
        self.checkpoints()
            .iter()
            .find(|checkpoint| checkpoint.tick == tick)
            .or_else(|| {
                terminal_branch(self)?
                    .checkpoints
                    .iter()
                    .find(|value| value.tick == tick)
            })
    }

    fn future_checkpoints(
        &self,
        publication: Tick,
    ) -> Result<Vec<&ContinuationCheckpoint>, RouteFailure> {
        let mut values = self
            .checkpoints()
            .iter()
            .filter(|checkpoint| checkpoint.tick > publication)
            .take(GRANT_INPUTS)
            .collect::<Vec<_>>();
        if values.is_empty() {
            return Ok(values);
        }
        if values.len() < GRANT_INPUTS {
            let terminal = terminal_branch(self).ok_or(RouteFailure::InvalidRequest)?;
            for checkpoint in terminal
                .checkpoints
                .iter()
                .filter(|checkpoint| checkpoint.tick > publication)
            {
                if values.iter().all(|value| value.tick != checkpoint.tick) {
                    values.push(checkpoint);
                }
                if values.len() == GRANT_INPUTS {
                    break;
                }
            }
        }
        (values.len() == GRANT_INPUTS)
            .then_some(values)
            .ok_or(RouteFailure::InvalidRequest)
    }

    fn fallback_after(&self, tick: Tick) -> Result<&[ContinuationCheckpoint], RouteFailure> {
        if let Some(branch) = self
            .stop_branches()
            .iter()
            .find(|branch| branch.publication == tick)
        {
            return Ok(&branch.checkpoints);
        }
        let branch = terminal_branch(self).ok_or(RouteFailure::InvalidRequest)?;
        let first = branch
            .checkpoints
            .iter()
            .position(|checkpoint| checkpoint.tick > tick)
            .unwrap_or(branch.checkpoints.len());
        Ok(&branch.checkpoints[first..])
    }
}

fn terminal_branch(certificate: &ContinuationCertificate) -> Option<&StopBranch> {
    certificate.stop_branches().last()
}

fn charge_index_scans(
    certificate: &ContinuationCertificate,
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    let terminal = terminal_branch(certificate).map_or(0, |branch| branch.checkpoints.len());
    let units = certificate
        .checkpoints()
        .len()
        .checked_mul(2)
        .and_then(|value| value.checked_add(certificate.stop_branches().len().checked_mul(4)?))
        .and_then(|value| value.checked_add(terminal.checked_mul(4)?))
        .and_then(|value| value.checked_add(8))
        .ok_or(RouteFailure::Capacity)?;
    charge_len(units, work)
}

fn charge_len(units: usize, work: &mut WorkBudget) -> Result<(), RouteFailure> {
    work.try_take(u32::try_from(units).map_err(|_| RouteFailure::Capacity)?)
        .map_err(|_| RouteFailure::WorkExhausted)
}

fn validate_expected(
    basis: &Observation,
    current: &Observation,
    expected: &ContinuationCheckpoint,
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    charge_len(current.parts().actors.len(), work)?;
    let actor = current
        .parts()
        .actors
        .iter()
        .find_map(|candidate| match candidate {
            ObservedActor::Visible(actor) if actor.seat == current.parts().recipient => Some(actor),
            _ => None,
        })
        .ok_or(RouteFailure::InvalidRequest)?;
    charge_len(
        basis
            .parts()
            .actors
            .len()
            .checked_mul(2)
            .and_then(|value| value.checked_add(current.parts().actors.len()))
            .ok_or(RouteFailure::Capacity)?,
        work,
    )?;
    if (actor.position, actor.heading) != (expected.motion.position, expected.motion.heading)
        || !expected.pit_guard.matches(current, work)?
        || !storm::observation_fits(basis, current, expected.motion.storm_deadlines)
    {
        return Err(RouteFailure::NoRoute);
    }
    Ok(())
}

fn fresh_contact(
    current: &Observation,
    work: &mut WorkBudget,
) -> Result<MobContactEnvelope, RouteFailure> {
    charge_len(current.parts().actors.len(), work)?;
    MobContactEnvelope::new(current, work).map_err(super::pulse_support::contact_failure)
}

fn validate_fresh_paths(
    certificate: &ContinuationCertificate,
    observation: &Observation,
    fresh: &MobContactEnvelope,
    future: &[&ContinuationCheckpoint],
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    for checkpoint in future {
        validate_segments(
            observation,
            fresh,
            checkpoint.tick,
            &checkpoint.segments,
            work,
        )?;
    }
    // Each possible publication may be followed by a forced Stop. The complete
    // proof stores that exact branch, including turn residual and pit/storm state.
    for checkpoint in future {
        for stopped in certificate.fallback_after(checkpoint.tick)? {
            validate_segments(observation, fresh, stopped.tick, &stopped.segments, work)?;
        }
    }
    Ok(())
}

fn validate_complete_stop(
    certificate: &ContinuationCertificate,
    observation: &Observation,
    fresh: &MobContactEnvelope,
    publication: Tick,
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    for stopped in certificate.fallback_after(publication)? {
        validate_segments(observation, fresh, stopped.tick, &stopped.segments, work)?;
    }
    Ok(())
}

fn validate_segments(
    observation: &Observation,
    fresh: &MobContactEnvelope,
    tick: Tick,
    segments: &[MovementSegment],
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    if !fresh
        .sweep_is_clear(tick, segments, work)
        .map_err(super::pulse_support::contact_failure)?
    {
        return Err(RouteFailure::NoRoute);
    }
    let scans = observation
        .parts()
        .pickups
        .len()
        .checked_add(observation.parts().flames.len())
        .and_then(|count| u32::try_from(count).ok())
        .ok_or(RouteFailure::Capacity)?;
    work.try_take(scans)
        .map_err(|_| RouteFailure::WorkExhausted)?;
    let end = Tick(tick.0.checked_add(1).ok_or(RouteFailure::InvalidRequest)?);
    if !teleport::motion_is_clear(observation, TickWindow { start: tick, end }, segments, work)
        .map_err(super::pulse_support::contact_failure)?
    {
        return Err(RouteFailure::NoRoute);
    }
    Ok(())
}

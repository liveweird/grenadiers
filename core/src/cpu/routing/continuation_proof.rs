use std::collections::BTreeSet;

use crate::cpu::forecast::{HazardForecast, TickWindow};
use crate::cpu::lair_contact::MobContactEnvelope;
use crate::cpu::{MovementIntent, Observation, WorkBudget};
use crate::game::{EntityId, Tick};

use super::continuation_types::{ContinuationCheckpoint, STOP_BRANCH_UPDATES, StopBranch};
use super::motion::{Motion, advance};
use super::pulse::PulseOriginGrace;
use super::pulse_support::{contact_failure, one_tick, update_origin};
use super::route_checks::{CollisionProofs, check_stationary_all, check_transition_all};
use super::validation::Context;
use super::{RouteFailure, RouteMode};

pub(super) struct ProofState {
    pub(super) motion: Motion,
    pub(super) pits: super::spike_pits::PitTrajectory,
    pub(super) origin_active: bool,
}

pub(super) struct ProofContext<'a> {
    pub(super) context: &'a Context<'a>,
    pub(super) forecast: &'a HazardForecast,
    pub(super) contact: &'a MobContactEnvelope,
    pub(super) origin: PulseOriginGrace,
}

pub(super) fn advance_checked(
    proof: &ProofContext<'_>,
    state: &mut ProofState,
    movement: MovementIntent,
    tick: Tick,
    work: &mut WorkBudget,
    assumptions: &mut BTreeSet<EntityId>,
) -> Result<ContinuationCheckpoint, RouteFailure> {
    super::storm::charge_tick(proof.context.storm, work)?;
    let segments = advance(proof.context, &mut state.motion, movement, tick)?;
    state
        .pits
        .advance(tick, segments[0].start(), &segments, work)?;
    if !proof
        .contact
        .sweep_is_clear(tick, &segments, work)
        .map_err(contact_failure)?
    {
        return Err(RouteFailure::NoRoute);
    }
    assumptions.extend(check_transition_all(
        proof.context,
        &[proof.forecast],
        &segments,
        one_tick(tick)?,
        RouteMode::AllowStationaryAssumptions,
        CollisionProofs {
            placement_origin: state.origin_active.then_some(proof.origin.cell),
            recipient_barrier: None,
        },
        work,
    )?);
    update_origin(Some(proof.origin), &segments, &mut state.origin_active)?;
    Ok(ContinuationCheckpoint {
        tick,
        movement,
        motion: state.motion.snapshot(),
        segments,
        origin_active: state.origin_active,
        pit_guard: state.pits.guard()?,
    })
}

pub(super) fn certify_stop_branch(
    proof: &ProofContext<'_>,
    after: &ProofState,
    publication: Tick,
    work: &mut WorkBudget,
    assumptions: &mut BTreeSet<EntityId>,
) -> Result<StopBranch, RouteFailure> {
    let mut branch = ProofState {
        motion: after.motion.clone(),
        pits: after.pits.clone(),
        origin_active: after.origin_active,
    };
    let end = Tick(
        publication
            .0
            .checked_add(STOP_BRANCH_UPDATES + 1)
            .ok_or(RouteFailure::InvalidRequest)?,
    );
    if end > proof.forecast.horizon_end() {
        return Err(RouteFailure::InvalidRequest);
    }
    let mut checkpoints = Vec::with_capacity(STOP_BRANCH_UPDATES as usize);
    for value in publication.0 + 1..end.0 {
        checkpoints.push(advance_checked(
            proof,
            &mut branch,
            MovementIntent::Stop,
            Tick(value),
            work,
            assumptions,
        )?);
    }
    Ok(StopBranch {
        publication,
        checkpoints,
    })
}

pub(super) fn certify_stationary_suffix(
    proof: &ProofContext<'_>,
    state: &mut ProofState,
    start: Tick,
    end: Tick,
    work: &mut WorkBudget,
    assumptions: &mut BTreeSet<EntityId>,
) -> Result<(), RouteFailure> {
    if start >= end {
        return Ok(());
    }
    if proof.context.storm.is_some() {
        for value in start.0..end.0 {
            advance_checked(
                proof,
                state,
                MovementIntent::Stop,
                Tick(value),
                work,
                assumptions,
            )?;
        }
        return Ok(());
    }
    let last = Tick(end.0.checked_sub(1).ok_or(RouteFailure::InvalidRequest)?);
    if !proof
        .contact
        .stationary_is_clear_until(state.motion.position, last, work)
        .map_err(contact_failure)?
    {
        return Err(RouteFailure::NoRoute);
    }
    assumptions.extend(check_stationary_all(
        &[proof.forecast],
        state.motion.position,
        TickWindow { start, end },
        RouteMode::AllowStationaryAssumptions,
        CollisionProofs {
            placement_origin: state.origin_active.then_some(proof.origin.cell),
            recipient_barrier: None,
        },
        work,
    )?);
    state
        .pits
        .stationary_until(end, state.motion.position, work)?;
    state.motion.stop();
    Ok(())
}

pub(super) fn reject_initial_scene(
    observation: &Observation,
    forecast: &HazardForecast,
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    let parts = observation.parts();
    let crate::cpu::BombKnowledge::Visible(bombs) = &parts.bombs else {
        return Err(RouteFailure::Unsupported);
    };
    let scans = parts
        .pickups
        .len()
        .checked_add(bombs.len())
        .and_then(|count| u32::try_from(count.checked_add(1)?).ok())
        .ok_or(RouteFailure::Capacity)?;
    work.try_take(scans)
        .map_err(|_| RouteFailure::WorkExhausted)?;
    if parts.rules.version != 10
        || parts.rules.running_updates == 0
        || !parts.flames.is_empty()
        || !parts.oil.is_empty()
        || parts.pickups.iter().any(|pickup| {
            matches!(
                pickup.kind,
                crate::game::PickupKind::Teleporter | crate::game::PickupKind::Diarrhea
            )
        })
        || bombs.iter().any(|bomb| {
            bomb.blast != crate::game::BlastKind::Standard
                || !matches!(
                    bomb.trigger,
                    crate::cpu::ObservedBombTrigger::Timed { paused: false, .. }
                )
                || !forecast.stationary_assumptions().contains(&bomb.id)
        })
    {
        return Err(RouteFailure::Unsupported);
    }
    Ok(())
}

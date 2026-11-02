use std::collections::BTreeSet;

use crate::cpu::forecast::{HazardForecast, TickWindow};
use crate::cpu::{MovementIntent, WorkBudget};
use crate::game::{EntityId, Tick};

use super::motion::{Motion, advance};
use super::pulse::PulseOriginGrace;
use super::pulse_support::{guard_teleporter_collection, one_tick, take_segments};
use super::safety::stationary_segment;
use super::validation::Context;
use super::{PulseRequest, RouteFailure};

pub(super) struct SuffixEvidence<'a> {
    pub(super) work: &'a mut WorkBudget,
    pub(super) storm_deadlines: &'a mut [(Tick, Vec<Option<super::storm::DeadlineBounds>>)],
    pub(super) spike_pit_guards: &'a mut [(Tick, Vec<super::SpikePitGuard>)],
    pub(super) assumptions: &'a mut BTreeSet<EntityId>,
    pub(super) guard_teleport: bool,
}

pub(super) fn certify_stopped_suffix(
    context: &Context<'_>,
    forecast: &HazardForecast,
    request: PulseRequest,
    origin: Option<PulseOriginGrace>,
    scenarios: &[(Tick, Motion, bool)],
    pit_scenarios: &[super::spike_pits::PitTrajectory],
    evidence: SuffixEvidence<'_>,
) -> Result<(), RouteFailure> {
    for ((((start, motion, origin_active), pits), (_, deadlines)), (_, guards)) in scenarios
        .iter()
        .zip(pit_scenarios)
        .zip(evidence.storm_deadlines.iter_mut())
        .zip(evidence.spike_pit_guards.iter_mut())
    {
        let stopped = Tick(
            start
                .0
                .checked_add(u64::from(request.hold_ticks))
                .ok_or(RouteFailure::InvalidRequest)?,
        );
        if stopped >= request.stopped_safe_until {
            continue;
        }
        if evidence.guard_teleport {
            guard_teleporter_collection(
                context.observation,
                TickWindow {
                    start: stopped,
                    end: request.stopped_safe_until,
                },
                &stationary_segment(motion.position),
                evidence.work,
            )?;
        }
        if context.storm.is_none() {
            certify_static_suffix(
                forecast,
                request,
                origin,
                *origin_active,
                motion,
                pits,
                guards,
                &mut *evidence.work,
                &mut *evidence.assumptions,
                stopped,
            )?;
            continue;
        }
        let mut simulation = motion.clone();
        let mut pits = pits.clone();
        for tick in stopped.0..request.stopped_safe_until.0 {
            super::storm::charge_tick(context.storm, evidence.work)?;
            let tick = Tick(tick);
            let segment = advance(context, &mut simulation, MovementIntent::Stop, tick)?;
            pits.advance(tick, segment[0].start(), &segment, evidence.work)?;
            deadlines.push(simulation.storm_deadlines());
            guards.push(pits.guard()?);
            take_segments(evidence.work, &segment, forecast.dimensions())?;
            evidence
                .assumptions
                .extend(super::safety::check_segments_with_grace(
                    forecast,
                    &segment,
                    one_tick(tick)?,
                    request.mode,
                    origin.filter(|_| *origin_active).map(|value| value.cell),
                    None,
                )?);
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn certify_static_suffix(
    forecast: &HazardForecast,
    request: PulseRequest,
    origin: Option<PulseOriginGrace>,
    origin_active: bool,
    motion: &Motion,
    pits: &super::spike_pits::PitTrajectory,
    guards: &mut Vec<super::SpikePitGuard>,
    work: &mut WorkBudget,
    assumptions: &mut BTreeSet<EntityId>,
    stopped: Tick,
) -> Result<(), RouteFailure> {
    let mut pits = pits.clone();
    for value in stopped.0..request.stopped_safe_until.0 {
        let tick = Tick(value);
        let segment = stationary_segment(motion.position);
        pits.advance(tick, motion.position, &segment, work)?;
        guards.push(pits.guard()?);
    }
    let segment = stationary_segment(motion.position);
    take_segments(work, &segment, forecast.dimensions())?;
    assumptions.extend(super::safety::check_segments_with_grace(
        forecast,
        &segment,
        TickWindow {
            start: stopped,
            end: request.stopped_safe_until,
        },
        request.mode,
        origin.filter(|_| origin_active).map(|value| value.cell),
        None,
    )?);
    Ok(())
}

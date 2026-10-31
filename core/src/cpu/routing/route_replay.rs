use crate::cpu::{MovementIntent, WorkBudget};
use crate::game::Tick;

use super::motion::{Motion, advance};
use super::route_time::add_tick;
use super::route_types::StepTiming;
use super::validation::Context;
use super::{RouteFailure, RouteLane, RouteRequest, RouteStep};

pub(super) fn build_lanes(
    context: &Context<'_>,
    request: RouteRequest,
    steps: &[RouteStep],
    work: &mut WorkBudget,
) -> Result<Vec<RouteLane>, RouteFailure> {
    let lane_count = request
        .start_window
        .end
        .0
        .checked_sub(request.start_window.start.0)
        .ok_or(RouteFailure::InvalidRequest)?;
    let snapshots = request
        .survive_until
        .0
        .checked_sub(context.observation.parts().tick.0)
        .and_then(|ticks| ticks.checked_mul(lane_count))
        .and_then(|count| u32::try_from(count).ok())
        .ok_or(RouteFailure::Capacity)?;
    work.try_take(snapshots)
        .map_err(|_| RouteFailure::WorkExhausted)?;
    let mut lanes = Vec::new();
    for first in request.start_window.start.0..request.start_window.end.0 {
        lanes.push(build_lane(context, request, steps, Tick(first), work)?);
    }
    Ok(lanes)
}

fn build_lane(
    context: &Context<'_>,
    request: RouteRequest,
    steps: &[RouteStep],
    first: Tick,
    work: &mut WorkBudget,
) -> Result<RouteLane, RouteFailure> {
    let mut motion = Motion::stopped(context, first);
    let mut pits = super::spike_pits::PitTrajectory::from_observation(context.observation, work)?;
    let mut poses = vec![(
        context.observation.parts().tick,
        motion.position,
        motion.heading,
        motion.storm_deadlines(),
        pits.guard()?,
    )];
    for tick in add_tick(context.observation.parts().tick, 1)?.0..first.0 {
        let tick = Tick(tick);
        super::storm::charge_tick(context.storm, work)?;
        let segments = advance(context, &mut motion, MovementIntent::Stop, tick)?;
        pits.advance(tick, segments[0].start(), &segments, work)?;
        poses.push((
            tick,
            motion.position,
            motion.heading,
            motion.storm_deadlines(),
            pits.guard()?,
        ));
    }
    let mut timings = Vec::with_capacity(steps.len());
    let mut ready = first;
    for step in steps {
        let start = ready;
        match *step {
            RouteStep::Move { direction, updates } => {
                for offset in 0..updates {
                    let tick = add_tick(start, u64::from(offset))?;
                    super::storm::charge_tick(context.storm, work)?;
                    let segments = advance(
                        context,
                        &mut motion,
                        MovementIntent::Direction(direction),
                        tick,
                    )?;
                    pits.advance(tick, segments[0].start(), &segments, work)?;
                    poses.push((
                        tick,
                        motion.position,
                        motion.heading,
                        motion.storm_deadlines(),
                        pits.guard()?,
                    ));
                }
                let stop = add_tick(start, u64::from(updates))?;
                for offset in 0..3 {
                    let tick = add_tick(stop, offset)?;
                    super::storm::charge_tick(context.storm, work)?;
                    let segments = advance(context, &mut motion, MovementIntent::Stop, tick)?;
                    pits.advance(tick, segments[0].start(), &segments, work)?;
                    poses.push((
                        tick,
                        motion.position,
                        motion.heading,
                        motion.storm_deadlines(),
                        pits.guard()?,
                    ));
                }
                ready = add_tick(stop, 3)?;
                timings.push(StepTiming { start, stop, ready });
            }
            RouteStep::Wait { updates } => {
                for offset in 0..updates {
                    let tick = add_tick(start, u64::from(offset))?;
                    super::storm::charge_tick(context.storm, work)?;
                    let segments = advance(context, &mut motion, MovementIntent::Stop, tick)?;
                    pits.advance(tick, segments[0].start(), &segments, work)?;
                    poses.push((
                        tick,
                        motion.position,
                        motion.heading,
                        motion.storm_deadlines(),
                        pits.guard()?,
                    ));
                }
                ready = add_tick(start, u64::from(updates))?;
                timings.push(StepTiming {
                    start,
                    stop: start,
                    ready,
                });
            }
        }
    }
    for tick in ready.0..request.survive_until.0 {
        let tick = Tick(tick);
        super::storm::charge_tick(context.storm, work)?;
        let segments = advance(context, &mut motion, MovementIntent::Stop, tick)?;
        pits.advance(tick, segments[0].start(), &segments, work)?;
        poses.push((
            tick,
            motion.position,
            motion.heading,
            motion.storm_deadlines(),
            pits.guard()?,
        ));
    }
    RouteLane::new(first, timings, poses)
}

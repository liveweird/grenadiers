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
    (request.start_window.start.0..request.start_window.end.0)
        .map(|first| build_lane(context, request, steps, Tick(first)))
        .collect()
}

fn build_lane(
    context: &Context<'_>,
    request: RouteRequest,
    steps: &[RouteStep],
    first: Tick,
) -> Result<RouteLane, RouteFailure> {
    let mut motion = Motion::stopped(context, first);
    let mut poses = (context.observation.parts().tick.0..first.0)
        .map(|tick| (Tick(tick), motion.position, motion.heading))
        .collect::<Vec<_>>();
    let mut timings = Vec::with_capacity(steps.len());
    let mut ready = first;
    for step in steps {
        let start = ready;
        match *step {
            RouteStep::Move { direction, updates } => {
                for offset in 0..updates {
                    let tick = add_tick(start, u64::from(offset))?;
                    let _ = advance(
                        context,
                        &mut motion,
                        MovementIntent::Direction(direction),
                        tick,
                    )?;
                    poses.push((tick, motion.position, motion.heading));
                }
                let stop = add_tick(start, u64::from(updates))?;
                motion.stop();
                for offset in 0..3 {
                    poses.push((add_tick(stop, offset)?, motion.position, motion.heading));
                }
                ready = add_tick(stop, 3)?;
                timings.push(StepTiming { start, stop, ready });
            }
            RouteStep::Wait { updates } => {
                for offset in 0..updates {
                    poses.push((
                        add_tick(start, u64::from(offset))?,
                        motion.position,
                        motion.heading,
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
        poses.push((Tick(tick), motion.position, motion.heading));
    }
    RouteLane::new(first, timings, poses)
}

use std::collections::BTreeSet;

use crate::cpu::forecast::HazardForecast;
use crate::cpu::{MovementIntent, WorkBudget};
use crate::game::Direction;

use super::motion::advance;
use super::route::{CONDITIONAL_MOVE_UPDATES, SearchNode};
use super::route_checks::{CollisionProofs, check_stationary_all, check_transition_all, one_tick};
use super::route_replay::build_lanes;
use super::route_time::add_tick;
use super::safety::stationary_segment;
use super::validation::Context;
use super::{
    MIN_FINAL_STOP_UPDATES, RouteAssurance, RouteCertificate, RouteFailure, RouteMode,
    RouteRequest, RouteStep,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_move(
    context: &Context<'_>,
    forecasts: &[&HazardForecast],
    mode: RouteMode,
    direction: Direction,
    updates: u8,
    require_progress: bool,
    node: &mut SearchNode,
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    let starts = node
        .lanes
        .iter()
        .map(|lane| (lane.motion.position, lane.motion.heading))
        .collect::<Vec<_>>();
    for lane in &mut node.lanes {
        for offset in 0..updates {
            super::storm::charge_tick(context.storm, work)?;
            let tick = add_tick(lane.ready, u64::from(offset))?;
            let segments = advance(
                context,
                &mut lane.motion,
                MovementIntent::Direction(direction),
                tick,
            )?;
            lane.pits
                .advance(tick, segments[0].start(), &segments, work)?;
            node.assumptions.extend(check_transition_all(
                context,
                forecasts,
                &segments,
                one_tick(tick)?,
                mode,
                CollisionProofs {
                    placement_origin: lane.origin_grace,
                    recipient_barrier: lane.recipient_barrier,
                },
                work,
            )?);
            if lane.origin_grace.is_some_and(|cell| {
                !crate::game::geometry::position_overlaps_cell(lane.motion.position, cell)
            }) {
                lane.origin_grace = None;
            }
            lane.recipient_barrier = retain_recipient_barrier(lane.recipient_barrier, &segments);
        }
        let stop = add_tick(lane.ready, u64::from(updates))?;
        if context.storm.is_none() {
            lane.motion.stop();
            let segment = stationary_segment(lane.motion.position);
            node.assumptions.extend(check_transition_all(
                context,
                forecasts,
                &segment,
                crate::cpu::forecast::TickWindow {
                    start: stop,
                    end: add_tick(stop, 3)?,
                },
                mode,
                CollisionProofs {
                    placement_origin: lane.origin_grace,
                    recipient_barrier: lane.recipient_barrier,
                },
                work,
            )?);
            lane.pits
                .stationary_until(add_tick(stop, 3)?, lane.motion.position, work)?;
        }
        if context.storm.is_some() {
            for offset in 0..3 {
                super::storm::charge_tick(context.storm, work)?;
                let tick = add_tick(stop, offset)?;
                let segment = advance(context, &mut lane.motion, MovementIntent::Stop, tick)?;
                lane.pits
                    .advance(tick, segment[0].start(), &segment, work)?;
                node.assumptions.extend(check_transition_all(
                    context,
                    forecasts,
                    &segment,
                    one_tick(tick)?,
                    mode,
                    CollisionProofs {
                        placement_origin: lane.origin_grace,
                        recipient_barrier: lane.recipient_barrier,
                    },
                    work,
                )?);
            }
        }
        lane.ready = add_tick(stop, 3)?;
        lane.last_stop = Some(stop);
    }
    if (require_progress
        && starts.iter().zip(&node.lanes).all(|(start, lane)| {
            *start == (lane.motion.position, lane.motion.heading)
        }))
        || (!node.assumptions.is_empty()
        && (updates > CONDITIONAL_MOVE_UPDATES
            || node.steps.iter().any(|step| {
                matches!(step, RouteStep::Move { updates, .. } if *updates > CONDITIONAL_MOVE_UPDATES)
            })))
    {
        return Err(RouteFailure::NoRoute);
    }
    Ok(())
}

pub(super) fn retain_recipient_barrier(
    barrier: Option<super::safety::RecipientBarrier>,
    segments: &[crate::game::MovementSegment],
) -> Option<super::safety::RecipientBarrier> {
    barrier.filter(|barrier| {
        segments.iter().all(|segment| {
            crate::game::geometry::position_overlaps_cell(segment.start(), barrier.cell)
                && crate::game::geometry::position_overlaps_cell(segment.end(), barrier.cell)
        })
    })
}

pub(super) fn apply_wait(
    context: &Context<'_>,
    forecasts: &[&HazardForecast],
    mode: RouteMode,
    updates: u16,
    node: &mut SearchNode,
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    for lane in &mut node.lanes {
        let end = add_tick(lane.ready, u64::from(updates))?;
        if context.storm.is_none() {
            node.assumptions.extend(check_stationary_all(
                forecasts,
                lane.motion.position,
                crate::cpu::forecast::TickWindow {
                    start: lane.ready,
                    end,
                },
                mode,
                CollisionProofs {
                    placement_origin: lane.origin_grace,
                    recipient_barrier: lane.recipient_barrier,
                },
                work,
            )?);
            lane.motion.stop();
            lane.pits
                .stationary_until(end, lane.motion.position, work)?;
        }
        if context.storm.is_some() {
            for tick in lane.ready.0..end.0 {
                super::storm::charge_tick(context.storm, work)?;
                let tick = crate::game::Tick(tick);
                let segment = advance(context, &mut lane.motion, MovementIntent::Stop, tick)?;
                lane.pits
                    .advance(tick, segment[0].start(), &segment, work)?;
                node.assumptions.extend(check_transition_all(
                    context,
                    forecasts,
                    &segment,
                    one_tick(tick)?,
                    mode,
                    CollisionProofs {
                        placement_origin: lane.origin_grace,
                        recipient_barrier: lane.recipient_barrier,
                    },
                    work,
                )?);
            }
        }
        lane.ready = end;
    }
    Ok(())
}

pub(super) fn terminal(
    context: &Context<'_>,
    forecasts: &[&HazardForecast],
    request: RouteRequest,
    node: &mut SearchNode,
    work: &mut WorkBudget,
) -> Result<bool, RouteFailure> {
    let mut assumptions = BTreeSet::new();
    for lane in &mut node.lanes {
        if lane.ready > request.survive_until {
            return Ok(false);
        }
        if let Some(stop) = lane.last_stop
            && add_tick(stop, MIN_FINAL_STOP_UPDATES)? > request.survive_until
        {
            return Ok(false);
        }
        let mut pits = lane.pits.clone();
        if context.storm.is_none() {
            match pits.stationary_until(request.survive_until, lane.motion.position, work) {
                Ok(()) => {}
                Err(RouteFailure::NoRoute) => return Ok(false),
                Err(error) => return Err(error),
            }
            match check_stationary_all(
                forecasts,
                lane.motion.position,
                crate::cpu::forecast::TickWindow {
                    start: lane.ready,
                    end: request.survive_until,
                },
                request.mode,
                CollisionProofs {
                    placement_origin: lane.origin_grace,
                    recipient_barrier: lane.recipient_barrier,
                },
                work,
            ) {
                Ok(found) => assumptions.extend(found),
                Err(RouteFailure::NoRoute) => return Ok(false),
                Err(error) => return Err(error),
            }
            continue;
        }
        if context.observation.parts().rules.version == 10 {
            match pits.stationary_until(request.survive_until, lane.motion.position, work) {
                Ok(()) => {}
                Err(RouteFailure::NoRoute) => return Ok(false),
                Err(error) => return Err(error),
            }
            let window = crate::cpu::forecast::TickWindow {
                start: lane.ready,
                end: request.survive_until,
            };
            if window.start < window.end {
                match check_transition_all(
                    context,
                    forecasts,
                    &stationary_segment(lane.motion.position),
                    window,
                    request.mode,
                    CollisionProofs {
                        placement_origin: lane.origin_grace,
                        recipient_barrier: lane.recipient_barrier,
                    },
                    work,
                ) {
                    Ok(found) => assumptions.extend(found),
                    Err(RouteFailure::NoRoute) => return Ok(false),
                    Err(error) => return Err(error),
                }
            }
            continue;
        }
        let mut simulation = lane.motion.clone();
        for tick in lane.ready.0..request.survive_until.0 {
            super::storm::charge_tick(context.storm, work)?;
            let tick = crate::game::Tick(tick);
            let segment = advance(context, &mut simulation, MovementIntent::Stop, tick)?;
            match pits.advance(tick, segment[0].start(), &segment, work) {
                Ok(()) => {}
                Err(RouteFailure::NoRoute) => return Ok(false),
                Err(error) => return Err(error),
            }
            match check_transition_all(
                context,
                forecasts,
                &segment,
                one_tick(tick)?,
                request.mode,
                CollisionProofs {
                    placement_origin: lane.origin_grace,
                    recipient_barrier: lane.recipient_barrier,
                },
                work,
            ) {
                Ok(found) => assumptions.extend(found),
                Err(RouteFailure::NoRoute) => return Ok(false),
                Err(error) => return Err(error),
            }
        }
    }
    if !assumptions.is_empty()
        && node.steps.iter().any(|step| {
            matches!(step, RouteStep::Move { updates, .. } if *updates > CONDITIONAL_MOVE_UPDATES)
        })
    {
        return Ok(false);
    }
    node.assumptions.extend(assumptions);
    Ok(true)
}

pub(super) fn finish(
    context: &Context<'_>,
    forecast: &HazardForecast,
    request: RouteRequest,
    node: SearchNode,
    work: &mut WorkBudget,
) -> Result<RouteCertificate, RouteFailure> {
    let assurance = if node.assumptions.is_empty() {
        RouteAssurance::Certified
    } else {
        RouteAssurance::ConditionalStationary {
            bombs: node.assumptions.into_iter().collect(),
        }
    };
    let lanes = build_lanes(context, request, &node.steps, work)?;
    RouteCertificate::new(
        context.key,
        forecast.basis_tick(),
        request.start_window,
        assurance,
        request.survive_until,
        node.steps,
        lanes,
        forecast.evolution().clone(),
    )
}

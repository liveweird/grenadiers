use crate::cpu::{Observation, ObservedActor, WorkBudget};
use crate::game::geometry::MovementSegment;
use crate::game::storm_geometry::swept_hits_position;
use crate::game::types::HALF_FOOTPRINT;
use crate::game::{
    Direction, Position, STORM_CLOUD_HALF_EXTENT, STORM_CLOUD_SPEED_PER_TICK, STORM_SLOW_TICKS,
    StormCloudConfig, StormCloudView, Tick,
};

use super::RouteFailure;

#[derive(Clone, Copy)]
pub(super) struct StormContext {
    basis_tick: Tick,
    cloud: StormCloudView,
    min: Position,
    max: Position,
    initial_deadline: Tick,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DeadlineBounds {
    pub(super) lower: Tick,
    pub(super) upper: Tick,
}

#[derive(Clone, Copy)]
struct CloudBounds {
    start_min: Position,
    end_min: Position,
    end_max: Position,
    exact: bool,
}

pub(super) fn context(observation: &Observation) -> Result<Option<StormContext>, RouteFailure> {
    let Some(environment) = observation.environment() else {
        return Ok(None);
    };
    match environment.storm_cloud_config() {
        StormCloudConfig::Disabled => Ok(None),
        StormCloudConfig::Single => {
            let cloud = environment
                .storm_cloud()
                .ok_or(RouteFailure::InvalidRequest)?;
            let actor_index = observation
                .parts()
                .actors
                .iter()
                .position(|actor| actor.seat() == observation.parts().recipient)
                .ok_or(RouteFailure::InvalidRequest)?;
            let remaining = environment
                .actor_storm_slow_remaining_ticks()
                .get(actor_index)
                .copied()
                .flatten()
                .ok_or(RouteFailure::InvalidRequest)?;
            let initial_deadline = Tick(
                observation
                    .parts()
                    .tick
                    .0
                    .checked_add(u64::from(remaining))
                    .ok_or(RouteFailure::InvalidRequest)?,
            );
            Ok(Some(StormContext {
                basis_tick: observation.parts().tick,
                cloud,
                min: Position::center(crate::game::CellId::new(1, 1)),
                max: Position::center(crate::game::CellId::new(
                    observation.parts().arena.width - 2,
                    observation.parts().arena.height - 2,
                )),
                initial_deadline,
            }))
        }
    }
}

impl StormContext {
    pub(super) const fn initial_deadline(self) -> DeadlineBounds {
        DeadlineBounds {
            lower: self.initial_deadline,
            upper: self.initial_deadline,
        }
    }

    pub(super) fn apply_tick(
        self,
        tick: Tick,
        actor_start: Position,
        actor_segments: &[MovementSegment],
        deadlines: &mut DeadlineBounds,
    ) -> Result<(), RouteFailure> {
        let cloud = self.cloud_bounds(tick)?;
        let exact_contact = cloud.exact
            && (swept_hits_position(
                MovementSegment::new(cloud.start_min, cloud.end_min),
                STORM_CLOUD_HALF_EXTENT,
                actor_start,
                HALF_FOOTPRINT,
            ) || actor_segments.iter().any(|segment| {
                swept_hits_position(
                    *segment,
                    HALF_FOOTPRINT,
                    cloud.end_min,
                    STORM_CLOUD_HALF_EXTENT,
                )
            }));
        let possible_contact = exact_contact
            || swept_boxes_overlap(
                cloud.start_min,
                cloud.end_max,
                STORM_CLOUD_HALF_EXTENT,
                actor_start,
                actor_start,
                HALF_FOOTPRINT,
            )
            || actor_segments.iter().any(|segment| {
                swept_boxes_overlap(
                    segment.start(),
                    segment.end(),
                    HALF_FOOTPRINT,
                    cloud.end_min,
                    cloud.end_max,
                    STORM_CLOUD_HALF_EXTENT,
                )
            });
        if possible_contact {
            let refresh = Tick(
                tick.0
                    .checked_add(STORM_SLOW_TICKS)
                    .ok_or(RouteFailure::InvalidRequest)?,
            );
            deadlines.upper = deadlines.upper.max(refresh);
            if exact_contact {
                deadlines.lower = deadlines.lower.max(refresh);
            }
        }
        Ok(())
    }

    fn cloud_bounds(self, tick: Tick) -> Result<CloudBounds, RouteFailure> {
        let elapsed = tick
            .0
            .checked_sub(self.basis_tick.0)
            .ok_or(RouteFailure::InvalidRequest)?;
        if elapsed == 0 {
            return Ok(CloudBounds {
                start_min: self.cloud.position,
                end_min: self.cloud.position,
                end_max: self.cloud.position,
                exact: true,
            });
        }
        if elapsed <= u64::from(self.cloud.leg_remaining_ticks) {
            let known_start = translate(self.cloud.position, self.cloud.direction, elapsed - 1);
            let known_end = translate(self.cloud.position, self.cloud.direction, elapsed);
            return Ok(CloudBounds {
                start_min: known_start,
                end_min: known_end,
                end_max: known_end,
                exact: true,
            });
        }
        let endpoint = translate(
            self.cloud.position,
            self.cloud.direction,
            u64::from(self.cloud.leg_remaining_ticks),
        );
        let unknown_end = elapsed - u64::from(self.cloud.leg_remaining_ticks);
        let unknown_start = unknown_end.saturating_sub(1);
        Ok(CloudBounds {
            start_min: envelope_min(endpoint, unknown_start, self.min),
            end_min: envelope_min(endpoint, unknown_end, self.min),
            end_max: envelope_max(endpoint, unknown_end, self.max),
            exact: false,
        })
    }
}

pub(super) fn charge_tick(
    storm: Option<StormContext>,
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    if storm.is_some() {
        work.try_take(1).map_err(|_| RouteFailure::WorkExhausted)?;
    }
    Ok(())
}

pub(crate) fn observation_fits(
    basis: &Observation,
    current: &Observation,
    deadlines: Option<DeadlineBounds>,
) -> bool {
    let (Some(old), Some(new)) = (basis.environment(), current.environment()) else {
        return basis.environment().is_none() && current.environment().is_none();
    };
    if old.storm_cloud_config() != new.storm_cloud_config() {
        return false;
    }
    if old.storm_cloud_config() == StormCloudConfig::Disabled {
        return true;
    }
    let Ok(Some(storm)) = context(basis) else {
        return false;
    };
    let Some(observed_cloud) = new.storm_cloud() else {
        return false;
    };
    let Ok(bounds) = storm.cloud_bounds(current.parts().tick) else {
        return false;
    };
    if observed_cloud.position.x < bounds.end_min.x
        || observed_cloud.position.x > bounds.end_max.x
        || observed_cloud.position.y < bounds.end_min.y
        || observed_cloud.position.y > bounds.end_max.y
    {
        return false;
    }
    let elapsed = current.parts().tick.0.saturating_sub(basis.parts().tick.0);
    if elapsed <= u64::from(storm.cloud.leg_remaining_ticks)
        && (observed_cloud.direction != storm.cloud.direction
            || u64::from(observed_cloud.leg_remaining_ticks)
                != u64::from(storm.cloud.leg_remaining_ticks) - elapsed)
    {
        return false;
    }
    let Some(bounds) = deadlines else {
        return true;
    };
    let Some(index) = current
        .parts()
        .actors
        .iter()
        .position(|actor| actor.seat() == current.parts().recipient)
    else {
        return false;
    };
    let Some(remaining) = new
        .actor_storm_slow_remaining_ticks()
        .get(index)
        .copied()
        .flatten()
    else {
        return false;
    };
    let lower_remaining = bounds.lower.0.saturating_sub(current.parts().tick.0);
    let upper_remaining = bounds.upper.0.saturating_sub(current.parts().tick.0);
    (lower_remaining..=upper_remaining).contains(&u64::from(remaining))
}

pub(crate) fn stationary_deadlines_at(basis: &Observation, at: Tick) -> Option<DeadlineBounds> {
    let storm = context(basis).ok().flatten()?;
    let mut deadlines = storm.initial_deadline();
    let actor = basis.parts().actors.iter().find_map(|actor| match actor {
        ObservedActor::Visible(actor) if actor.seat == basis.parts().recipient => Some(actor),
        ObservedActor::Visible(_) | ObservedActor::Hidden { .. } => None,
    })?;
    let stationary = [MovementSegment::new(actor.position, actor.position)];
    let first = basis.parts().tick.0.checked_add(1)?;
    for tick in first..=at.0 {
        storm
            .apply_tick(Tick(tick), actor.position, &stationary, &mut deadlines)
            .ok()?;
    }
    Some(deadlines)
}

fn translate(position: Position, direction: Direction, ticks: u64) -> Position {
    let distance = i32::try_from(ticks)
        .unwrap_or(i32::MAX)
        .saturating_mul(STORM_CLOUD_SPEED_PER_TICK);
    match direction {
        Direction::North => Position {
            y: position.y.saturating_sub(distance),
            ..position
        },
        Direction::East => Position {
            x: position.x.saturating_add(distance),
            ..position
        },
        Direction::South => Position {
            y: position.y.saturating_add(distance),
            ..position
        },
        Direction::West => Position {
            x: position.x.saturating_sub(distance),
            ..position
        },
    }
}

fn envelope_min(origin: Position, ticks: u64, min: Position) -> Position {
    let reach = i32::try_from(ticks)
        .unwrap_or(i32::MAX)
        .saturating_mul(STORM_CLOUD_SPEED_PER_TICK);
    Position {
        x: origin.x.saturating_sub(reach).max(min.x),
        y: origin.y.saturating_sub(reach).max(min.y),
    }
}

fn envelope_max(origin: Position, ticks: u64, max: Position) -> Position {
    let reach = i32::try_from(ticks)
        .unwrap_or(i32::MAX)
        .saturating_mul(STORM_CLOUD_SPEED_PER_TICK);
    Position {
        x: origin.x.saturating_add(reach).min(max.x),
        y: origin.y.saturating_add(reach).min(max.y),
    }
}

fn swept_boxes_overlap(
    left_start: Position,
    left_end: Position,
    left_half: i32,
    right_start: Position,
    right_end: Position,
    right_half: i32,
) -> bool {
    let sum = i64::from(left_half) + i64::from(right_half);
    i64::from(left_start.x.min(left_end.x)) < i64::from(right_start.x.max(right_end.x)) + sum
        && i64::from(left_start.x.max(left_end.x)) > i64::from(right_start.x.min(right_end.x)) - sum
        && i64::from(left_start.y.min(left_end.y)) < i64::from(right_start.y.max(right_end.y)) + sum
        && i64::from(left_start.y.max(left_end.y)) > i64::from(right_start.y.min(right_end.y)) - sum
}

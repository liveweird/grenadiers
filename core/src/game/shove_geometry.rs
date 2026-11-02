use std::cmp::Ordering;

use super::Direction;
use crate::game::MovementSegment;
use crate::game::movement::MovementOutcome;
use crate::game::types::HALF_FOOTPRINT;

pub(crate) fn stationary_contact_entry(holder: MovementSegment, rival: super::Position) -> bool {
    let distance = segment_length(holder);
    if distance <= 0 {
        return false;
    }
    let mut holder_trace = super::MovementTrace::new(holder.start());
    holder_trace.push(holder);
    let holder = MovementOutcome {
        trace: holder_trace,
        kick: None,
        original_budget: distance,
        forward_segment: Some(holder),
        blocking_player: None,
        punch_contact: None,
    };
    let rival = MovementOutcome {
        trace: super::MovementTrace::new(rival),
        kick: None,
        original_budget: 0,
        forward_segment: None,
        blocking_player: None,
        punch_contact: None,
    };
    contact_entry(&holder, &rival).is_some()
}

/// Positive-area contact between an arbitrary axis-aligned actor sweep and a
/// stationary actor-sized footprint. Unlike shove entry timing, forced travel
/// is not bounded by one voluntary movement budget.
pub(crate) fn stationary_sweep_contact(
    segment: MovementSegment,
    position: super::Position,
) -> bool {
    let extent = i64::from(HALF_FOOTPRINT) * 2;
    let start = segment.start();
    let end = segment.end();
    if start.x == end.x {
        (i64::from(start.x) - i64::from(position.x)).abs() < extent
            && i64::from(position.y) > i64::from(start.y.min(end.y)) - extent
            && i64::from(position.y) < i64::from(start.y.max(end.y)) + extent
    } else {
        (i64::from(start.y) - i64::from(position.y)).abs() < extent
            && i64::from(position.x) > i64::from(start.x.min(end.x)) - extent
            && i64::from(position.x) < i64::from(start.x.max(end.x)) + extent
    }
}

#[derive(Clone, Copy)]
struct Velocity {
    axis: (i64, i64),
    forward: Option<MovementSegment>,
}

#[derive(Clone, Copy)]
struct Fraction {
    numerator: i64,
    denominator: i64,
}

#[derive(Clone, Copy)]
pub(super) struct ContactEntry {
    pub(super) direction: Direction,
    time: Fraction,
}

impl ContactEntry {
    pub(super) fn compare_time(self, other: Self) -> Ordering {
        self.time.compare(other.time)
    }
}

impl Fraction {
    const fn integer(value: i64) -> Self {
        Self {
            numerator: value,
            denominator: 1,
        }
    }

    fn new(numerator: i64, denominator: i64) -> Self {
        debug_assert!(denominator != 0);
        if denominator < 0 {
            Self {
                numerator: -numerator,
                denominator: -denominator,
            }
        } else {
            Self {
                numerator,
                denominator,
            }
        }
    }

    fn compare(self, other: Self) -> Ordering {
        (i128::from(self.numerator) * i128::from(other.denominator))
            .cmp(&(i128::from(other.numerator) * i128::from(self.denominator)))
    }
}

pub(super) fn contact_entry(
    holder: &MovementOutcome,
    rival: &MovementOutcome,
) -> Option<ContactEntry> {
    let forward = holder.forward_segment?;
    let direction = segment_direction(forward)?;
    if holder.original_budget <= 0 {
        return None;
    }
    let outcomes = [holder, rival];
    debug_assert!(
        outcomes
            .iter()
            .all(|outcome| (0..=86).contains(&outcome.original_budget)),
        "supported speeds produce at most 86 units per update"
    );
    let denominator =
        i64::from(holder.original_budget.max(1)) * i64::from(rival.original_budget.max(1));
    // The common denominator is at most 86²=7,396 and relative axis velocity is at most 172.
    // Arena coordinates are below 15,360, so widened i64 position products stay below 114M;
    // Fraction cross-products widen again to i128 before comparison.
    let boundaries = merged_boundaries(&outcomes, denominator);
    debug_assert!(
        boundaries.len() <= 6,
        "two paths create at most five intervals"
    );
    for pair in boundaries.windows(2) {
        let start = pair[0];
        let end = pair[1];
        let holder_position = scaled_position(holder, start, denominator);
        let rival_position = scaled_position(rival, start, denominator);
        let holder_velocity = interval_velocity(holder, start, denominator);
        let rival_velocity = interval_velocity(rival, start, denominator);
        let relative = (
            rival_position.0 - holder_position.0,
            rival_position.1 - holder_position.1,
        );
        let relative_velocity = (
            rival_velocity.axis.0 - holder_velocity.axis.0,
            rival_velocity.axis.1 - holder_velocity.axis.1,
        );
        let initially_overlapped = overlap(relative, denominator);
        let Some(entry) = overlap_window(relative, relative_velocity, denominator, end - start)
        else {
            continue;
        };
        if !initially_overlapped
            && holder_velocity.forward == Some(forward)
            && leading_face_entry(direction, relative, relative_velocity, entry, denominator)
        {
            return Some(ContactEntry {
                direction,
                time: Fraction::new(
                    start * entry.denominator + entry.numerator,
                    denominator * entry.denominator,
                ),
            });
        }
    }
    None
}

#[cfg(test)]
pub(super) fn contact_entry_direction(
    seat: super::SeatId,
    outcomes: &[MovementOutcome; 2],
) -> Option<Direction> {
    contact_entry(&outcomes[seat.index()], &outcomes[1 - seat.index()]).map(|entry| entry.direction)
}

fn merged_boundaries(outcomes: &[&MovementOutcome; 2], denominator: i64) -> Vec<i64> {
    let mut boundaries = vec![0, denominator];
    for outcome in outcomes {
        let budget = i64::from(outcome.original_budget.max(1));
        let mut elapsed = 0;
        for segment in outcome.trace.segments() {
            elapsed += i64::from(segment_length(segment)) * denominator / budget;
            boundaries.push(elapsed);
        }
    }
    boundaries.sort_unstable();
    boundaries.dedup();
    boundaries
}

fn overlap_window(
    relative: (i64, i64),
    velocity: (i64, i64),
    scale: i64,
    length: i64,
) -> Option<Fraction> {
    let mut lower = Fraction::integer(0);
    let mut upper = Fraction::integer(length);
    let limit = i64::from(2 * HALF_FOOTPRINT) * scale;
    for (position, speed) in [(relative.0, velocity.0), (relative.1, velocity.1)] {
        if speed == 0 {
            if position.abs() >= limit {
                return None;
            }
            continue;
        }
        let first = Fraction::new(-limit - position, speed);
        let second = Fraction::new(limit - position, speed);
        let axis_lower = if first.compare(second).is_lt() {
            first
        } else {
            second
        };
        let axis_upper = if first.compare(second).is_gt() {
            first
        } else {
            second
        };
        if lower.compare(axis_lower).is_lt() {
            lower = axis_lower;
        }
        if upper.compare(axis_upper).is_gt() {
            upper = axis_upper;
        }
    }
    (lower.compare(upper).is_lt()).then_some(lower)
}

fn leading_face_entry(
    direction: Direction,
    relative: (i64, i64),
    velocity: (i64, i64),
    entry: Fraction,
    scale: i64,
) -> bool {
    let value_at_entry = |position: i64, speed: i64| {
        i128::from(position) * i128::from(entry.denominator)
            + i128::from(speed) * i128::from(entry.numerator)
    };
    let leading_boundary =
        i128::from(2 * HALF_FOOTPRINT) * i128::from(scale) * i128::from(entry.denominator);
    match direction {
        Direction::East => {
            velocity.0 < 0 && value_at_entry(relative.0, velocity.0) == leading_boundary
        }
        Direction::West => {
            velocity.0 > 0 && value_at_entry(relative.0, velocity.0) == -leading_boundary
        }
        Direction::South => {
            velocity.1 < 0 && value_at_entry(relative.1, velocity.1) == leading_boundary
        }
        Direction::North => {
            velocity.1 > 0 && value_at_entry(relative.1, velocity.1) == -leading_boundary
        }
    }
}

fn scaled_position(outcome: &MovementOutcome, time: i64, denominator: i64) -> (i64, i64) {
    let budget = i64::from(outcome.original_budget.max(1));
    let mut elapsed = 0;
    let mut position = outcome.trace.start();
    for segment in outcome.trace.segments() {
        let duration = i64::from(segment_length(segment)) * denominator / budget;
        if time <= elapsed + duration && duration > 0 {
            let (vx, vy) = direction_vector(segment, budget);
            return (
                i64::from(segment.start().x) * denominator + vx * (time - elapsed),
                i64::from(segment.start().y) * denominator + vy * (time - elapsed),
            );
        }
        elapsed += duration;
        position = segment.end();
    }
    (
        i64::from(position.x) * denominator,
        i64::from(position.y) * denominator,
    )
}

fn interval_velocity(outcome: &MovementOutcome, time: i64, denominator: i64) -> Velocity {
    let budget = i64::from(outcome.original_budget.max(1));
    let mut elapsed = 0;
    for segment in outcome.trace.segments() {
        let duration = i64::from(segment_length(segment)) * denominator / budget;
        if time >= elapsed && time < elapsed + duration {
            return Velocity {
                axis: direction_vector(segment, budget),
                forward: (outcome.forward_segment == Some(segment)).then_some(segment),
            };
        }
        elapsed += duration;
    }
    Velocity {
        axis: (0, 0),
        forward: None,
    }
}

fn overlap(relative: (i64, i64), scale: i64) -> bool {
    let limit = i64::from(2 * HALF_FOOTPRINT) * scale;
    relative.0.abs() < limit && relative.1.abs() < limit
}

fn segment_length(segment: MovementSegment) -> i32 {
    (segment.end().x - segment.start().x).abs() + (segment.end().y - segment.start().y).abs()
}

fn segment_direction(segment: MovementSegment) -> Option<Direction> {
    let dx = segment.end().x - segment.start().x;
    let dy = segment.end().y - segment.start().y;
    match (dx.signum(), dy.signum()) {
        (1, 0) => Some(Direction::East),
        (-1, 0) => Some(Direction::West),
        (0, 1) => Some(Direction::South),
        (0, -1) => Some(Direction::North),
        _ => None,
    }
}

fn direction_vector(segment: MovementSegment, speed: i64) -> (i64, i64) {
    match segment_direction(segment) {
        Some(Direction::East) => (speed, 0),
        Some(Direction::West) => (-speed, 0),
        Some(Direction::South) => (0, speed),
        Some(Direction::North) => (0, -speed),
        None => (0, 0),
    }
}

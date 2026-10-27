use crate::cpu::VisibleActor;
use crate::game::types::TICKS_PER_SECOND;
use crate::game::{CellId, Direction, EffectKind, Position};

use super::MAX_HOLD_TICKS;

pub(super) const STEPS: [(Direction, i32, i32); 4] = [
    (Direction::North, 0, -1),
    (Direction::East, 1, 0),
    (Direction::South, 0, 1),
    (Direction::West, -1, 0),
];

pub(super) fn offset(cell: CellId, dc: i32, dr: i32) -> Option<CellId> {
    Some(CellId::new(
        u16::try_from(i32::from(cell.column) + dc).ok()?,
        u16::try_from(i32::from(cell.row) + dr).ok()?,
    ))
}

pub(super) fn direction(from: CellId, to: CellId) -> Option<Direction> {
    STEPS
        .iter()
        .find_map(|(direction, dc, dr)| (offset(from, *dc, *dr) == Some(to)).then_some(*direction))
}

pub(super) fn input_direction(actor: &VisibleActor, desired: Direction) -> Direction {
    if actor
        .effects
        .iter()
        .any(|effect| effect.kind == EffectKind::ReverseControls)
    {
        match desired {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    } else {
        desired
    }
}

/// Chooses a pulse whose maximum possible travel cannot pass the 64-unit
/// center window. Unknown movement residual is represented by floor/ceil bounds.
pub(super) fn hold_ticks(actor: &VisibleActor, target_cell: CellId, desired: Direction) -> u8 {
    let target = Position::center(target_cell);
    let distance = axis_distance(actor.position, target, desired);
    let (numerator, denominator) = speed_ratio(actor);
    let mut best = 1;
    for ticks in 1..=MAX_HOLD_TICKS {
        let scaled = u64::from(ticks) * u64::from(numerator);
        let maximum = scaled.div_ceil(u64::from(TICKS_PER_SECOND) * denominator);
        if maximum <= u64::try_from(distance.saturating_add(64)).unwrap_or(0) {
            best = ticks;
        }
    }
    best
}

pub(super) fn pulse_minimum_distance(actor: &VisibleActor, ticks: u8) -> u64 {
    let (numerator, denominator) = speed_ratio(actor);
    (u64::from(ticks) * u64::from(numerator)) / (u64::from(TICKS_PER_SECOND) * denominator)
}

pub(super) fn bounded_alignment_hold_ticks(
    actor: &VisibleActor,
    cell: CellId,
    direction: Direction,
    tolerance: u32,
) -> Option<u8> {
    let ticks = hold_ticks(actor, cell, direction);
    let distance = u64::try_from(axis_distance(
        actor.position,
        Position::center(cell),
        direction,
    ))
    .ok()?;
    let minimum = pulse_minimum_distance(actor, ticks);
    let (numerator, denominator) = speed_ratio(actor);
    let maximum = (u64::from(ticks) * u64::from(numerator))
        .div_ceil(u64::from(TICKS_PER_SECOND) * denominator);
    let window = u64::from(tolerance.min(64));
    let minimum_endpoint = distance.abs_diff(minimum);
    let maximum_endpoint = distance.abs_diff(maximum);
    // The correction changes only this coordinate. Ending within 64 units of its
    // center keeps it in `cell`; the unchanged perpendicular coordinate was
    // already in `cell` because `cell` is the actor's containing cell.
    (minimum_endpoint <= window
        && maximum_endpoint <= window
        && minimum_endpoint < distance
        && maximum_endpoint < distance)
        .then_some(ticks)
}

fn speed_ratio(actor: &VisibleActor) -> (u16, u64) {
    let stacks = actor
        .effects
        .iter()
        .filter(|effect| effect.kind == EffectKind::SlowFoot)
        .count();
    (actor.speed.units_per_second(), 1_u64 << stacks.min(6))
}

fn axis_distance(from: Position, to: Position, direction: Direction) -> i32 {
    match direction {
        Direction::North => from.y.saturating_sub(to.y),
        Direction::East => to.x.saturating_sub(from.x),
        Direction::South => to.y.saturating_sub(from.y),
        Direction::West => from.x.saturating_sub(to.x),
    }
    .max(0)
}

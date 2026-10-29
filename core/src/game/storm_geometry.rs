use super::{MovementSegment, Position};

pub(crate) fn swept_hits_position(
    segment: MovementSegment,
    moving_half: i32,
    stationary: Position,
    stationary_half: i32,
) -> bool {
    let sum = i64::from(moving_half) + i64::from(stationary_half);
    let left = i64::from(segment.start().x.min(segment.end().x));
    let right = i64::from(segment.start().x.max(segment.end().x));
    let top = i64::from(segment.start().y.min(segment.end().y));
    let bottom = i64::from(segment.start().y.max(segment.end().y));
    left < i64::from(stationary.x) + sum
        && right > i64::from(stationary.x) - sum
        && top < i64::from(stationary.y) + sum
        && bottom > i64::from(stationary.y) - sum
}

pub(crate) fn footprints_overlap(
    left: Position,
    left_half: i32,
    right: Position,
    right_half: i32,
) -> bool {
    let sum = i64::from(left_half) + i64::from(right_half);
    (i64::from(left.x) - i64::from(right.x)).abs() < sum
        && (i64::from(left.y) - i64::from(right.y)).abs() < sum
}

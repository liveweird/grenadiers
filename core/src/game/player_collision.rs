use super::{Direction, MovementSegment, Position, SeatId};
use crate::game::types::{HALF_FOOTPRINT, UNITS_PER_CELL};

/// One actor-owned reservation considered by solid-player movement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerObstacle {
    pub seat: SeatId,
    pub segment: MovementSegment,
}

impl PlayerObstacle {
    #[must_use]
    pub const fn stationary(seat: SeatId, position: Position) -> Self {
        Self {
            seat,
            segment: MovementSegment::new(position, position),
        }
    }
}

/// The accepted distance and optional actor face physically reached by a move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerClip {
    pub distance: i32,
    pub physical_contact: Option<SeatId>,
}

/// Whether two actor endpoints violate the solid-player position invariant.
#[must_use]
pub fn player_positions_conflict(left: Position, right: Position) -> bool {
    left.cell().is_some_and(|cell| right.cell() == Some(cell))
        || ((i64::from(left.x) - i64::from(right.x)).abs() < i64::from(2 * HALF_FOOTPRINT)
            && (i64::from(left.y) - i64::from(right.y)).abs() < i64::from(2 * HALF_FOOTPRINT))
}

/// Returns the first conflicting pair in caller-supplied deterministic order.
#[must_use]
pub fn first_player_position_conflict(
    positions: &[(SeatId, Position)],
) -> Option<(SeatId, SeatId)> {
    for (index, &(left_seat, left)) in positions.iter().enumerate() {
        for &(right_seat, right) in &positions[index + 1..] {
            if player_positions_conflict(left, right) {
                return Some((left_seat, right_seat));
            }
        }
    }
    None
}

/// Clips one axis-aligned move against prior actor footprint and center-cell reservations.
///
/// Obstacles owned by `seat` are ignored. Footprint tangency is permitted. Center-cell
/// reservation is stricter: an accepted endpoint never belongs to an entered reserved cell.
#[must_use]
pub fn clip_player_segment(
    seat: SeatId,
    start: Position,
    direction: Direction,
    requested: i32,
    obstacles: &[PlayerObstacle],
) -> PlayerClip {
    let requested = requested.max(0);
    let mut distance = requested;
    let mut physical_contact = None;
    for obstacle in obstacles.iter().filter(|obstacle| obstacle.seat != seat) {
        if let Some(candidate) = footprint_distance(start, direction, obstacle.segment) {
            let candidate = candidate.max(0).min(requested);
            if candidate < requested
                && (candidate < distance
                    || (candidate == distance
                        && physical_contact.is_none_or(|prior| obstacle.seat < prior)))
            {
                distance = candidate;
                physical_contact = Some(obstacle.seat);
            }
        }
        for_reserved_cells(obstacle.segment, |column, row| {
            let candidate = cell_entry_distance(start, direction, column, row);
            if let Some(candidate) = candidate.filter(|candidate| *candidate < distance) {
                distance = candidate.max(0);
                physical_contact = None;
            }
        });
    }
    PlayerClip {
        distance,
        physical_contact,
    }
}

fn footprint_distance(
    start: Position,
    direction: Direction,
    obstacle: MovementSegment,
) -> Option<i32> {
    let left = obstacle.start().x.min(obstacle.end().x) - HALF_FOOTPRINT;
    let right = obstacle.start().x.max(obstacle.end().x) + HALF_FOOTPRINT;
    let top = obstacle.start().y.min(obstacle.end().y) - HALF_FOOTPRINT;
    let bottom = obstacle.start().y.max(obstacle.end().y) + HALF_FOOTPRINT;
    match direction {
        Direction::East
            if start.y - HALF_FOOTPRINT < bottom
                && start.y + HALF_FOOTPRINT > top
                && start.x + HALF_FOOTPRINT <= left =>
        {
            Some(left - HALF_FOOTPRINT - start.x)
        }
        Direction::West
            if start.y - HALF_FOOTPRINT < bottom
                && start.y + HALF_FOOTPRINT > top
                && start.x - HALF_FOOTPRINT >= right =>
        {
            Some(start.x - HALF_FOOTPRINT - right)
        }
        Direction::South
            if start.x - HALF_FOOTPRINT < right
                && start.x + HALF_FOOTPRINT > left
                && start.y + HALF_FOOTPRINT <= top =>
        {
            Some(top - HALF_FOOTPRINT - start.y)
        }
        Direction::North
            if start.x - HALF_FOOTPRINT < right
                && start.x + HALF_FOOTPRINT > left
                && start.y - HALF_FOOTPRINT >= bottom =>
        {
            Some(start.y - HALF_FOOTPRINT - bottom)
        }
        _ => None,
    }
}

fn for_reserved_cells(segment: MovementSegment, mut visit: impl FnMut(i32, i32)) {
    let start_column = segment.start().x.div_euclid(UNITS_PER_CELL);
    let end_column = segment.end().x.div_euclid(UNITS_PER_CELL);
    let start_row = segment.start().y.div_euclid(UNITS_PER_CELL);
    let end_row = segment.end().y.div_euclid(UNITS_PER_CELL);
    for row in start_row.min(end_row)..=start_row.max(end_row) {
        for column in start_column.min(end_column)..=start_column.max(end_column) {
            visit(column, row);
        }
    }
}

fn cell_entry_distance(
    start: Position,
    direction: Direction,
    column: i32,
    row: i32,
) -> Option<i32> {
    let left = column.saturating_mul(UNITS_PER_CELL);
    let right = left.saturating_add(UNITS_PER_CELL);
    let top = row.saturating_mul(UNITS_PER_CELL);
    let bottom = top.saturating_add(UNITS_PER_CELL);
    match direction {
        Direction::East if (top..bottom).contains(&start.y) && start.x < left => {
            Some(left - start.x - 1)
        }
        Direction::West if (top..bottom).contains(&start.y) && start.x >= right => {
            Some(start.x - right)
        }
        Direction::South if (left..right).contains(&start.x) && start.y < top => {
            Some(top - start.y - 1)
        }
        Direction::North if (left..right).contains(&start.x) && start.y >= bottom => {
            Some(start.y - bottom)
        }
        Direction::East | Direction::West
            if (left..right).contains(&start.x) && (top..bottom).contains(&start.y) =>
        {
            Some(0)
        }
        Direction::North | Direction::South
            if (left..right).contains(&start.x) && (top..bottom).contains(&start.y) =>
        {
            Some(0)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn position(x: i32, y: i32) -> Position {
        Position { x, y }
    }

    #[test]
    fn endpoint_conflict_includes_same_cell_and_positive_overlap_but_not_tangency() {
        assert!(player_positions_conflict(
            position(100, 100),
            position(900, 900)
        ));
        assert!(player_positions_conflict(
            position(1_000, 1_000),
            position(1_639, 1_000)
        ));
        assert!(!player_positions_conflict(
            position(1_000, 1_000),
            position(1_640, 1_000)
        ));
    }

    #[test]
    fn footprint_contact_clips_and_reports_the_lowest_seat_deterministically() {
        let blockers = [
            PlayerObstacle::stationary(SeatId::Three, position(2_200, 1_500)),
            PlayerObstacle::stationary(SeatId::Two, position(2_200, 1_500)),
        ];
        assert_eq!(
            clip_player_segment(
                SeatId::One,
                position(1_500, 1_500),
                Direction::East,
                500,
                &blockers,
            ),
            PlayerClip {
                distance: 60,
                physical_contact: Some(SeatId::Two),
            }
        );
    }

    #[test]
    fn reserved_cell_clips_one_unit_before_entry_without_physical_contact() {
        let blocker = PlayerObstacle::stationary(SeatId::Two, position(2_900, 1_500));
        assert_eq!(
            clip_player_segment(
                SeatId::One,
                position(1_500, 1_500),
                Direction::East,
                700,
                &[blocker],
            ),
            PlayerClip {
                distance: 547,
                physical_contact: None,
            }
        );
    }

    #[test]
    fn prior_sweep_blocks_crossing_and_own_reservations_are_ignored() {
        let sweep = PlayerObstacle {
            seat: SeatId::Two,
            segment: MovementSegment::new(position(2_400, 1_000), position(2_400, 3_000)),
        };
        assert_eq!(
            clip_player_segment(
                SeatId::One,
                position(1_500, 2_000),
                Direction::East,
                1_500,
                &[sweep],
            )
            .distance,
            260
        );
        assert_eq!(
            clip_player_segment(
                SeatId::Two,
                position(1_500, 2_000),
                Direction::East,
                1_500,
                &[sweep],
            )
            .distance,
            1_500
        );
    }
}

use crate::cpu::policy::StrategicTarget;
use crate::game::{Position, SeatId};

pub(super) fn position_distance(left: Position, right: Position) -> u32 {
    left.x
        .abs_diff(right.x)
        .saturating_add(left.y.abs_diff(right.y))
}

pub(super) const fn target_order(target: StrategicTarget) -> (u8, u16, u16, u8) {
    match target {
        StrategicTarget::Pickup { cell, .. } => (0, cell.row, cell.column, 0),
        StrategicTarget::Block { cell } => (1, cell.row, cell.column, 0),
        StrategicTarget::Opponent { seat } => (2, 0, 0, seat_order(seat)),
        StrategicTarget::Tile { cell } => (3, cell.row, cell.column, 0),
        StrategicTarget::None => (4, 0, 0, 0),
    }
}

const fn seat_order(seat: SeatId) -> u8 {
    seat as u8
}

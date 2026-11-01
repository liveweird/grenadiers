use crate::cpu::policy::StrategicTarget;
use crate::cpu::{Observation, WorkBudget};
use crate::game::{CellId, Position, SeatId, Tick};

use super::super::{STEPS, SearchMap, offset};

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

pub(super) fn fallback(
    current: CellId,
    observation: &Observation,
    map: &SearchMap,
    work: &mut WorkBudget,
) -> Option<CellId> {
    STEPS
        .into_iter()
        .filter_map(|(_, dc, dr)| {
            work.try_take(1).ok()?;
            offset(current, dc, dr).filter(|cell| map.transitable(observation, *cell))
        })
        .min_by_key(|cell| (cell.row, cell.column))
}

pub(super) fn add_ticks(tick: Tick, amount: u64) -> Tick {
    Tick(tick.0.saturating_add(amount))
}

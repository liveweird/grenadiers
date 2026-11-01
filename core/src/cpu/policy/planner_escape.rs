use std::collections::VecDeque;

use crate::cpu::{Observation, WorkBudget};
use crate::game::CellId;
use crate::game::types::UNITS_PER_CELL;

use super::super::hazards::immediately_safe;
use super::super::movement::{STEPS, offset, pulse_minimum_distance};
use super::super::{MAX_HOLD_TICKS, MAX_ROUTE_CELLS};

const PER_PULSE_IDLE_TICKS: u64 = 10;
const CENTER_TOLERANCE: u64 = 64;

pub(super) fn escape_route(
    observation: &Observation,
    origin: CellId,
    work: &mut WorkBudget,
) -> Option<Vec<CellId>> {
    let mut queue = VecDeque::from([vec![origin]]);
    while let Some(path) = queue.pop_front() {
        work.try_take(1).ok()?;
        let cell = *path.last()?;
        if path.len() >= 3 && cell.column != origin.column && cell.row != origin.row {
            return Some(path);
        }
        if path.len() == MAX_ROUTE_CELLS {
            continue;
        }
        for (_, dc, dr) in STEPS {
            let Some(next) = offset(cell, dc, dr) else {
                continue;
            };
            if !path.contains(&next)
                && next != origin
                && immediately_safe(observation, next)
                && !observation
                    .parts()
                    .pickups
                    .iter()
                    .any(|pickup| pickup.cell == next)
            {
                let mut extended = path.clone();
                extended.push(next);
                queue.push_back(extended);
            }
        }
    }
    None
}

pub(super) fn escape_before_fuse(
    actor: &crate::cpu::VisibleActor,
    steps: usize,
    fuse: u64,
) -> bool {
    let per_pulse = pulse_minimum_distance(actor, MAX_HOLD_TICKS).max(1);
    let first =
        u64::try_from(UNITS_PER_CELL + UNITS_PER_CELL / 2).unwrap_or(u64::MAX) + CENTER_TOLERANCE;
    let later = u64::try_from(UNITS_PER_CELL).unwrap_or(u64::MAX) + CENTER_TOLERANCE;
    let distance = first.saturating_add(later.saturating_mul(steps.saturating_sub(1) as u64));
    let pulses = distance.div_ceil(per_pulse);
    pulses.saturating_mul(u64::from(MAX_HOLD_TICKS) + PER_PULSE_IDLE_TICKS) < fuse
}

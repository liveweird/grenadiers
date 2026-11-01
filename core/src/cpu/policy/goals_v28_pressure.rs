use crate::cpu::{Observation, ObservedActor, VisibleActor, WorkBudget};
use crate::game::geometry::position_overlaps_cell;
use crate::game::{BlastKind, CellId, SeatId};

use super::super::{SearchMap, cell_index};
use super::{Candidate, DistanceMap, FirstStepMap};

const MAX_PRESSURE_WORK: u32 = 4_096;
const MAX_ORIGIN_PROBES: usize = 32;

const fn pressure_allowance(remaining: u32) -> u32 {
    let share = remaining / 8;
    if share < MAX_PRESSURE_WORK {
        share
    } else {
        MAX_PRESSURE_WORK
    }
}

pub(super) fn candidates(
    observation: &Observation,
    actor: &VisibleActor,
    distance: &DistanceMap,
    first_step: &FirstStepMap,
    map: &SearchMap,
    work: &mut WorkBudget,
) -> Vec<Candidate> {
    let allowance = pressure_allowance(work.remaining());
    let mut pressure_work = WorkBudget::new(allowance);
    let Some((kind, range)) = super::super::pressure::weapon_geometry(observation, actor) else {
        return Vec::new();
    };
    let Some(geometry) =
        crate::cpu::survival::ObservedBlastGeometry::new(observation, &mut pressure_work)
    else {
        let _ = work.try_take(pressure_work.spent());
        return Vec::new();
    };
    let mut rivals = observation
        .parts()
        .actors
        .iter()
        .filter_map(|observed| match observed {
            ObservedActor::Visible(rival) if rival.seat != actor.seat && rival.alive => Some(rival),
            _ => None,
        })
        .collect::<Vec<_>>();
    rivals.sort_unstable_by_key(|rival| rival.seat as u8);
    let mut result = Vec::new();
    for (index, rival) in rivals.iter().enumerate() {
        let divisor = u32::try_from(rivals.len() - index).unwrap_or(u32::MAX);
        let mut rival_work = WorkBudget::new(pressure_work.remaining() / divisor.max(1));
        if let Some(candidate) = nearest(
            observation,
            rival.seat,
            rival.position,
            range,
            kind,
            distance,
            first_step,
            map,
            &geometry,
            &mut rival_work,
        ) {
            result.push(candidate);
        }
        let _ = pressure_work.try_take(rival_work.spent());
    }
    let _ = work.try_take(pressure_work.spent());
    result
}

pub(super) fn add_legacy_candidates(
    observation: &Observation,
    actor: &VisibleActor,
    distance: &DistanceMap,
    first_step: &FirstStepMap,
    candidates: &mut Vec<Candidate>,
) {
    for observed in &observation.parts().actors {
        if let ObservedActor::Visible(rival) = observed
            && rival.seat != actor.seat
            && rival.alive
            && let Some(cell) = rival.position.cell()
        {
            super::farm::add_firing_candidates(
                observation,
                cell,
                crate::cpu::policy::StrategicObjectiveKind::Attack,
                crate::cpu::policy::StrategicTarget::Opponent { seat: rival.seat },
                1,
                distance,
                first_step,
                candidates,
                None,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn nearest(
    observation: &Observation,
    seat: SeatId,
    position: crate::game::Position,
    range: u16,
    kind: crate::game::BlastKind,
    distance: &DistanceMap,
    first_step: &FirstStepMap,
    map: &SearchMap,
    geometry: &crate::cpu::survival::ObservedBlastGeometry<'_>,
    work: &mut WorkBudget,
) -> Option<Candidate> {
    let mut target_cells = Vec::new();
    for row in 0..observation.parts().arena.height {
        for column in 0..observation.parts().arena.width {
            work.try_take(1).ok()?;
            let cell = CellId::new(column, row);
            if position_overlaps_cell(position, cell) {
                target_cells.push(cell);
            }
        }
    }
    let mut origins = Vec::new();
    for row in 0..observation.parts().arena.height {
        for column in 0..observation.parts().arena.width {
            work.try_take(1).ok()?;
            let cell = CellId::new(column, row);
            let Some(cell_index) = cell_index(observation, cell) else {
                continue;
            };
            if let (Some(distance), Some(first_step)) =
                (distance[cell_index], first_step[cell_index])
                && map.transitable(observation, cell)
                && plausibly_threatens(cell, range, kind, &target_cells)
            {
                origins.push((distance, cell, first_step));
            }
        }
    }
    origins.sort_unstable_by_key(|(distance, cell, _)| (*distance, cell.row, cell.column));
    for (distance, approach, first_step) in origins.into_iter().take(MAX_ORIGIN_PROBES) {
        if geometry.threatens_position(approach, range, kind, position, work)? {
            return Some(Candidate {
                objective: crate::cpu::policy::StrategicObjectiveKind::Attack,
                target: crate::cpu::policy::StrategicTarget::Opponent { seat },
                approach,
                score: 4,
                distance,
                first_step,
            });
        }
    }
    None
}

fn plausibly_threatens(
    origin: CellId,
    range: u16,
    kind: BlastKind,
    target_cells: &[CellId],
) -> bool {
    target_cells.iter().any(|target| {
        let columns = origin.column.abs_diff(target.column);
        let rows = origin.row.abs_diff(target.row);
        if kind == BlastKind::Power {
            columns.saturating_add(rows) <= range
        } else {
            (columns == 0 && rows <= range) || (rows == 0 && columns <= range)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::pressure_allowance;

    #[test]
    fn pressure_share_is_capped_but_smaller_budgets_remain_available() {
        assert_eq!(pressure_allowance(65_536), 4_096);
        assert_eq!(pressure_allowance(32_000), 4_000);
        assert_eq!(pressure_allowance(3_000), 375);
    }
}

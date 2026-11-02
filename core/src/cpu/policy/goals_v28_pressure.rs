use crate::cpu::{Observation, ObservedActor, VisibleActor, WorkBudget};
use crate::game::geometry::position_overlaps_cell;
use crate::game::{BlastKind, CellId, SeatId};

use super::super::{SearchMap, cell_index};
use super::{Candidate, DistanceMap, FirstStepMap};
use crate::cpu::policy::pit_avoidance::PitMask;

const MAX_PRESSURE_WORK: u32 = 4_096;
const MAX_ORIGIN_PROBES: usize = 32;

#[derive(Clone, Copy)]
pub(super) struct PressureOrigin {
    pub distance: u16,
    pub approach: CellId,
    pub first_step: CellId,
}

pub(super) struct RivalPressure {
    pub seat: SeatId,
    pub position: crate::game::Position,
    pub baseline: Option<Candidate>,
    pub origins: Vec<PressureOrigin>,
    pub next_probe: usize,
    pub probes: usize,
    pub complete: bool,
    pub unused_slot: u32,
}

pub(super) struct PressureMaps<'a> {
    pub distance: &'a DistanceMap,
    pub first_step: &'a FirstStepMap,
    pub search: &'a SearchMap,
    pub pits: &'a PitMask,
}

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
    maps: PressureMaps<'_>,
    mode: crate::cpu::policy::PressureMode,
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
    let mut rivals_pressure = Vec::new();
    for (index, rival) in rivals.iter().enumerate() {
        let divisor = u32::try_from(rivals.len() - index).unwrap_or(u32::MAX);
        let slot = pressure_work.remaining() / divisor.max(1);
        let mut rival_work = WorkBudget::new(slot);
        let baseline = nearest(
            observation,
            rival.seat,
            rival.position,
            range,
            kind,
            maps.distance,
            maps.first_step,
            maps.search,
            maps.pits,
            &geometry,
            &mut rival_work,
        );
        let unused_slot = slot.saturating_sub(rival_work.spent());
        let _ = pressure_work.try_take(rival_work.spent());
        rivals_pressure.push(RivalPressure {
            seat: rival.seat,
            position: rival.position,
            baseline: baseline.candidate,
            origins: baseline.origins,
            next_probe: baseline.next_probe,
            probes: baseline.probes,
            complete: baseline.complete,
            unused_slot,
        });
    }
    if mode == crate::cpu::policy::PressureMode::ExitCoverage {
        super::exit_pressure::promote(
            observation,
            kind,
            range,
            &geometry,
            &mut rivals_pressure,
            &mut pressure_work,
        );
    }
    let _ = work.try_take(pressure_work.spent());
    rivals_pressure
        .into_iter()
        .filter_map(|rival| rival.baseline)
        .collect()
}

pub(super) fn add_legacy_candidates(
    observation: &Observation,
    actor: &VisibleActor,
    distance: &DistanceMap,
    first_step: &FirstStepMap,
    pit_sites: &PitMask,
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
                pit_sites,
                candidates,
                None,
            );
        }
    }
}

struct BaselineAttempt {
    candidate: Option<Candidate>,
    origins: Vec<PressureOrigin>,
    next_probe: usize,
    probes: usize,
    complete: bool,
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
    pit_sites: &PitMask,
    geometry: &crate::cpu::survival::ObservedBlastGeometry<'_>,
    work: &mut WorkBudget,
) -> BaselineAttempt {
    let mut target_cells = Vec::new();
    for row in 0..observation.parts().arena.height {
        for column in 0..observation.parts().arena.width {
            if work.try_take(1).is_err() {
                return incomplete(Vec::new(), 0);
            }
            let cell = CellId::new(column, row);
            if position_overlaps_cell(position, cell) {
                target_cells.push(cell);
            }
        }
    }
    let mut origins = Vec::new();
    for row in 0..observation.parts().arena.height {
        for column in 0..observation.parts().arena.width {
            if work.try_take(1).is_err() {
                return incomplete(origins, 0);
            }
            let cell = CellId::new(column, row);
            let Some(cell_index) = cell_index(observation, cell) else {
                continue;
            };
            if let (Some(distance), Some(first_step)) =
                (distance[cell_index], first_step[cell_index])
                && map.transitable(observation, cell)
                && !pit_sites.contains(observation, cell)
                && plausibly_threatens(cell, range, kind, &target_cells)
            {
                origins.push(PressureOrigin {
                    distance,
                    approach: cell,
                    first_step,
                });
            }
        }
    }
    origins.sort_unstable_by_key(|origin| {
        (origin.distance, origin.approach.row, origin.approach.column)
    });
    for (index, origin) in origins.iter().take(MAX_ORIGIN_PROBES).enumerate() {
        let Some(threatens) =
            geometry.threatens_position(origin.approach, range, kind, position, work)
        else {
            return incomplete(origins, index);
        };
        if threatens {
            return BaselineAttempt {
                candidate: Some(Candidate {
                    objective: crate::cpu::policy::StrategicObjectiveKind::Attack,
                    target: crate::cpu::policy::StrategicTarget::Opponent { seat },
                    approach: origin.approach,
                    score: 4,
                    distance: origin.distance,
                    first_step: origin.first_step,
                }),
                origins,
                next_probe: index + 1,
                probes: index + 1,
                complete: true,
            };
        }
    }
    let probes = origins.len().min(MAX_ORIGIN_PROBES);
    BaselineAttempt {
        candidate: None,
        origins,
        next_probe: probes,
        probes,
        complete: true,
    }
}

fn incomplete(origins: Vec<PressureOrigin>, probes: usize) -> BaselineAttempt {
    BaselineAttempt {
        candidate: None,
        origins,
        next_probe: probes,
        probes,
        complete: false,
    }
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

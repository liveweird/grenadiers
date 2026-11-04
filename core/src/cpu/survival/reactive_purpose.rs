use crate::cpu::policy::{StrategicIntent, StrategicTarget};
use crate::cpu::{Observation, ObservedActor, WorkBudget};
use crate::game::{CellId, EntityId, SeatId, Terrain};
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ReactivePlantPurpose {
    Farm {
        block: CellId,
    },
    AttackDemolition {
        opponent_seat: SeatId,
        opponent_id: EntityId,
        opponent_cell: CellId,
        block: CellId,
        terrain: Vec<Terrain>,
        lairs: Vec<CellId>,
    },
}

pub(super) struct ReactivePurposeContext {
    target: StrategicTarget,
    attack: Option<AttackGeometry>,
}

struct AttackGeometry {
    opponent_seat: SeatId,
    opponent_id: EntityId,
    opponent_cell: CellId,
    distances: Vec<Option<u16>>,
    terrain: Vec<Terrain>,
    lairs: Vec<CellId>,
}

impl ReactivePurposeContext {
    pub(super) fn new(
        observation: &Observation,
        target: StrategicTarget,
        work: &mut WorkBudget,
    ) -> Option<Self> {
        let attack = if let StrategicTarget::Opponent { seat } = target {
            let opponent = observation
                .parts()
                .actors
                .iter()
                .find_map(|actor| match actor {
                    ObservedActor::Visible(actor) if actor.seat == seat && actor.alive => {
                        Some(actor)
                    }
                    _ => None,
                })?;
            let opponent_cell = opponent.position.cell()?;
            Some(AttackGeometry {
                opponent_seat: seat,
                opponent_id: opponent.id,
                opponent_cell,
                distances: permanent_distances(observation, opponent_cell, work)?,
                terrain: observation.parts().arena.terrain.clone(),
                lairs: lair_cells(observation),
            })
        } else {
            None
        };
        Some(Self { target, attack })
    }
}

impl ReactivePlantPurpose {
    pub(super) const fn block(&self) -> CellId {
        match self {
            Self::Farm { block } | Self::AttackDemolition { block, .. } => *block,
        }
    }

    pub(super) fn select(
        observation: &Observation,
        context: &ReactivePurposeContext,
        placement: crate::cpu::routing::PlannedBomb,
        evolution: &crate::cpu::forecast::ForecastEvolution,
        work: &mut WorkBudget,
    ) -> Option<Self> {
        let geometry = super::plant_value::ObservedBlastGeometry::new(observation, work)?;
        let hit = |block, work: &mut WorkBudget| {
            !evolution.removes_block_by(block, placement.detonates_at)
                && geometry
                    .visible_block_yield(
                        placement.cell,
                        placement.range,
                        placement.blast,
                        block,
                        work,
                    )
                    .is_some_and(|value| value > 0)
        };
        match context.target {
            StrategicTarget::Block { cell } => {
                hit(cell, work).then_some(Self::Farm { block: cell })
            }
            StrategicTarget::Opponent { .. } => {
                let attack = context.attack.as_ref()?;
                let placement_distance = distance(observation, &attack.distances, placement.cell)?;
                let block = observation
                    .parts()
                    .arena
                    .blocks
                    .iter()
                    .copied()
                    .filter(|block| {
                        if work.try_take(1).is_err() {
                            return false;
                        }
                        let Some(block_distance) = distance(observation, &attack.distances, *block)
                        else {
                            return false;
                        };
                        let separation = placement.cell.column.abs_diff(block.column)
                            + placement.cell.row.abs_diff(block.row);
                        separation > 0
                            && block_distance.checked_add(separation) == Some(placement_distance)
                            && hit(*block, work)
                    })
                    .min_by_key(|block| {
                        (
                            placement.cell.column.abs_diff(block.column)
                                + placement.cell.row.abs_diff(block.row),
                            block.row,
                            block.column,
                        )
                    })?;
                Some(Self::AttackDemolition {
                    opponent_seat: attack.opponent_seat,
                    opponent_id: attack.opponent_id,
                    opponent_cell: attack.opponent_cell,
                    block,
                    terrain: attack.terrain.clone(),
                    lairs: attack.lairs.clone(),
                })
            }
            _ => None,
        }
    }

    pub(super) fn still_targets(
        &self,
        observation: &Observation,
        intent: StrategicIntent,
        placement: &crate::cpu::routing::PlannedBomb,
        work: &mut WorkBudget,
    ) -> bool {
        let block = match self {
            Self::Farm { block } => {
                if intent.target != (StrategicTarget::Block { cell: *block }) {
                    return false;
                }
                *block
            }
            Self::AttackDemolition {
                opponent_seat,
                opponent_id,
                opponent_cell,
                block,
                terrain,
                lairs,
            } => {
                if intent.target != (StrategicTarget::Opponent { seat: *opponent_seat })
                    || observation.parts().arena.terrain != *terrain || lair_cells(observation) != *lairs
                    || !observation.parts().actors.iter().any(|actor| matches!(actor,
                        ObservedActor::Visible(actor) if actor.seat == *opponent_seat && actor.id == *opponent_id
                            && actor.alive && actor.position.cell() == Some(*opponent_cell))) { return false }
                *block
            }
        };
        observation.parts().arena.blocks.contains(&block)
            && super::plant_value::useful_plant_for(
                observation,
                placement,
                StrategicTarget::Block { cell: block },
                work,
            ) == Some(true)
    }
}

fn permanent_distances(
    observation: &Observation,
    start: CellId,
    work: &mut WorkBudget,
) -> Option<Vec<Option<u16>>> {
    let arena = &observation.parts().arena;
    let lairs = lair_cells(observation);
    let mut distances: Vec<Option<u16>> = vec![None; arena.cell_count()];
    distances[arena.index(start)?] = Some(0);
    let mut queue = VecDeque::from([start]);
    while let Some(cell) = queue.pop_front() {
        work.try_take(1).ok()?;
        let next = distances[arena.index(cell)?]?.checked_add(1)?;
        for (column, row) in [
            (cell.column.wrapping_sub(1), cell.row),
            (cell.column + 1, cell.row),
            (cell.column, cell.row.wrapping_sub(1)),
            (cell.column, cell.row + 1),
        ] {
            let candidate = CellId::new(column, row);
            let Some(index) = arena.index(candidate) else {
                continue;
            };
            if distances[index].is_none()
                && arena.terrain[index] != Terrain::Wall
                && !lairs.contains(&candidate)
            {
                distances[index] = Some(next);
                queue.push_back(candidate);
            }
        }
    }
    Some(distances)
}
fn distance(observation: &Observation, distances: &[Option<u16>], cell: CellId) -> Option<u16> {
    distances
        .get(observation.parts().arena.index(cell)?)?
        .as_ref()
        .copied()
}
fn lair_cells(observation: &Observation) -> Vec<CellId> {
    observation
        .respawn_lairs()
        .into_iter()
        .flat_map(|lairs| lairs.lairs().iter().map(|lair| lair.cell))
        .collect()
}

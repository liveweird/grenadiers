use std::ops::ControlFlow;

use crate::cpu::{BombKnowledge, Observation, ObservedActor, ObservedBombTrigger, WorkBudget};
use crate::game::arena::ARENA_CELL_COUNT;
use crate::game::blast_geometry::{BlastBomb, BlastField, BlastSource, BlastVisit, visit_blast};
use crate::game::geometry::position_overlaps_cell;
use crate::game::{BlastKind, CellId, OilBarrelState, Position, Terrain};

use crate::cpu::routing::PlannedBomb;

pub(super) fn could_be_useful(observation: &Observation, work: &mut WorkBudget) -> bool {
    let Some(actor) = super::compatibility::actor(observation) else {
        return false;
    };
    let Some(origin) = actor.position.cell() else {
        return false;
    };
    let parts = observation.parts();
    let scan = parts.arena.blocks.len().saturating_add(parts.actors.len());
    if work
        .try_take(u32::try_from(scan).unwrap_or(u32::MAX))
        .is_err()
    {
        return false;
    }
    let near = |target: CellId| {
        possible_origins(origin)
            .into_iter()
            .flatten()
            .any(|candidate| {
                candidate.column.abs_diff(target.column) + candidate.row.abs_diff(target.row) <= 9
            })
    };
    parts.arena.blocks.iter().copied().any(near)
        || parts.actors.iter().any(|observed| {
            matches!(observed, ObservedActor::Visible(rival)
                if rival.id != actor.id && rival.alive
                    && rival.position.cell().is_some_and(near))
        })
}

pub(super) fn useful_plant(
    observation: &Observation,
    bomb: &PlannedBomb,
    work: &mut WorkBudget,
) -> Option<bool> {
    let field = ObservedBlastGeometry::new(observation, work)?;
    if bomb.blast == BlastKind::Power {
        work.try_take(u32::try_from(observation.parts().arena.cell_count()).ok()?)
            .ok()?;
    }
    let mut useful = false;
    let result = visit_blast(
        &field,
        BlastSource {
            cell: bomb.cell,
            range: bomb.range,
            kind: bomb.blast,
        },
        |visit| {
            if work.try_take(1).is_err() {
                return ControlFlow::Break(());
            }
            useful |= visit.soft_block || field.hits_rival(visit);
            if useful {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        },
    );
    if result.is_break() && !useful {
        None
    } else {
        Some(useful)
    }
}

pub(crate) struct ObservedBlastGeometry<'a> {
    observation: &'a Observation,
    soft_blocks: [bool; ARENA_CELL_COUNT],
    unlit_oil: [bool; ARENA_CELL_COUNT],
    bombs: [Option<BlastBomb>; ARENA_CELL_COUNT],
}

impl<'a> ObservedBlastGeometry<'a> {
    pub(crate) fn new(observation: &'a Observation, work: &mut WorkBudget) -> Option<Self> {
        work.try_take(u32::try_from(observation.parts().arena.cell_count()).ok()?)
            .ok()?;
        let mut value = Self {
            observation,
            soft_blocks: [false; ARENA_CELL_COUNT],
            unlit_oil: [false; ARENA_CELL_COUNT],
            bombs: [None; ARENA_CELL_COUNT],
        };
        for &cell in &observation.parts().arena.blocks {
            value.soft_blocks[observation.parts().arena.index(cell)?] = true;
        }
        for oil in &observation.parts().oil {
            if oil.state == OilBarrelState::Unlit {
                value.unlit_oil[observation.parts().arena.index(oil.cell)?] = true;
            }
        }
        let BombKnowledge::Visible(bombs) = &observation.parts().bombs else {
            return None;
        };
        for bomb in bombs {
            value.bombs[observation.parts().arena.index(bomb.cell)?] = Some(BlastBomb {
                id: bomb.id,
                contact: bomb.trigger == ObservedBombTrigger::Contact,
            });
        }
        Some(value)
    }

    pub(crate) fn threatens_position(
        &self,
        firing: CellId,
        range: u16,
        kind: BlastKind,
        target: Position,
        work: &mut WorkBudget,
    ) -> Option<bool> {
        if kind == BlastKind::Power {
            work.try_take(u32::try_from(self.observation.parts().arena.cell_count()).ok()?)
                .ok()?;
        }
        let mut hit = false;
        let result = visit_blast(
            self,
            BlastSource {
                cell: firing,
                range,
                kind,
            },
            |visit| {
                if work.try_take(1).is_err() {
                    return ControlFlow::Break(());
                }
                hit = position_overlaps_cell(target, visit.cell);
                if hit {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            },
        );
        if result.is_break() && !hit {
            None
        } else {
            Some(hit)
        }
    }

    fn hits_rival(&self, visit: BlastVisit) -> bool {
        let owner = self
            .observation
            .parts()
            .actors
            .iter()
            .find(|actor| actor.seat() == self.observation.parts().recipient)
            .map(ObservedActor::id);
        self.observation.parts().actors.iter().any(|observed| {
            matches!(observed, ObservedActor::Visible(rival)
                if Some(rival.id) != owner && rival.alive
                    && position_overlaps_cell(rival.position, visit.cell))
        })
    }
}

impl BlastField for ObservedBlastGeometry<'_> {
    fn dimensions(&self) -> (u16, u16) {
        (
            self.observation.parts().arena.width,
            self.observation.parts().arena.height,
        )
    }

    fn is_floor(&self, cell: CellId) -> bool {
        self.observation
            .parts()
            .arena
            .index(cell)
            .is_some_and(|index| {
                self.observation.parts().arena.terrain.get(index) == Some(&Terrain::Floor)
            })
    }

    fn has_soft_block(&self, cell: CellId) -> bool {
        self.observation
            .parts()
            .arena
            .index(cell)
            .is_some_and(|index| self.soft_blocks[index])
    }

    fn has_unlit_oil(&self, cell: CellId) -> bool {
        self.observation
            .parts()
            .arena
            .index(cell)
            .is_some_and(|index| self.unlit_oil[index])
    }

    fn bomb_at(&self, cell: CellId) -> Option<BlastBomb> {
        self.observation
            .parts()
            .arena
            .index(cell)
            .and_then(|index| self.bombs[index])
    }
}

fn possible_origins(cell: CellId) -> [Option<CellId>; 5] {
    [
        Some(cell),
        cell.row
            .checked_sub(1)
            .map(|row| CellId::new(cell.column, row)),
        cell.column
            .checked_add(1)
            .map(|column| CellId::new(column, cell.row)),
        cell.row
            .checked_add(1)
            .map(|row| CellId::new(cell.column, row)),
        cell.column
            .checked_sub(1)
            .map(|column| CellId::new(column, cell.row)),
    ]
}

#[cfg(test)]
mod tests {
    use crate::cpu::policy::tests::observation;
    use crate::cpu::{Observation, ObservedActor, ObservedOil, WorkBudget};
    use crate::game::{BlastKind, CellId, EntityId, OilBarrelState, Position, SeatId, Tick};

    use super::{ARENA_CELL_COUNT, could_be_useful, useful_plant};
    use crate::cpu::routing::PlannedBomb;

    fn with_rival_and_oil(rival: CellId, oil: Option<CellId>) -> Observation {
        let mut parts = observation(CellId::new(7, 7)).into_parts();
        let ObservedActor::Visible(mut actor) = parts.actors[1].clone() else {
            unreachable!()
        };
        actor.id = EntityId(1);
        actor.seat = SeatId::One;
        actor.position = Position::center(rival);
        parts.actors[0] = ObservedActor::Visible(actor);
        if let Some(cell) = oil {
            parts.oil.push(ObservedOil {
                cell,
                state: OilBarrelState::Unlit,
            });
        }
        Observation::new(parts).unwrap()
    }

    fn bomb(blast: BlastKind) -> PlannedBomb {
        PlannedBomb {
            placed_at: Tick(17),
            owner: EntityId(2),
            cell: CellId::new(7, 7),
            range: 3,
            blast,
            detonates_at: Tick(197),
        }
    }

    #[test]
    fn shared_geometry_values_piercing_past_oil_and_power_diagonals() {
        let piercing = with_rival_and_oil(CellId::new(9, 7), Some(CellId::new(8, 7)));
        assert_eq!(
            useful_plant(
                &piercing,
                &bomb(BlastKind::Standard),
                &mut WorkBudget::new(1_000)
            ),
            Some(false)
        );
        assert_eq!(
            useful_plant(
                &piercing,
                &bomb(BlastKind::Piercing),
                &mut WorkBudget::new(1_000)
            ),
            Some(true)
        );

        let diagonal = with_rival_and_oil(CellId::new(8, 8), None);
        assert!(could_be_useful(&diagonal, &mut WorkBudget::new(100)));
        assert_eq!(
            useful_plant(
                &diagonal,
                &bomb(BlastKind::Standard),
                &mut WorkBudget::new(1_000)
            ),
            Some(false)
        );
        assert_eq!(
            useful_plant(
                &diagonal,
                &bomb(BlastKind::Power),
                &mut WorkBudget::new(1_000)
            ),
            Some(true)
        );
    }

    #[test]
    fn power_full_arena_scan_is_charged_before_value_can_succeed() {
        let diagonal = with_rival_and_oil(CellId::new(8, 8), None);
        assert_eq!(
            useful_plant(
                &diagonal,
                &bomb(BlastKind::Power),
                &mut WorkBudget::new(ARENA_CELL_COUNT as u32)
            ),
            None
        );
    }
}

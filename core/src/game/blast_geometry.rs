use std::ops::ControlFlow;

use super::arena::{ARENA_HEIGHT, ARENA_WIDTH};
use super::{BlastKind, CellId, EntityId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BlastSource {
    pub(crate) cell: CellId,
    pub(crate) range: u16,
    pub(crate) kind: BlastKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BlastBomb {
    pub(crate) id: EntityId,
    pub(crate) contact: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BlastVisit {
    pub(crate) cell: CellId,
    pub(crate) soft_block: bool,
    pub(crate) unlit_oil: bool,
    pub(crate) bomb: Option<BlastBomb>,
}

/// Immutable occupancy queried while tracing one blast.
///
/// Callers must keep this view unchanged across every blast belonging to the
/// same explosion phase. That preserves blockers until all same-tick chains
/// have been planned.
pub(crate) trait BlastField {
    fn dimensions(&self) -> (u16, u16) {
        (ARENA_WIDTH, ARENA_HEIGHT)
    }

    fn is_floor(&self, cell: CellId) -> bool;
    fn has_soft_block(&self, cell: CellId) -> bool;
    fn has_unlit_oil(&self, cell: CellId) -> bool;
    fn bomb_at(&self, cell: CellId) -> Option<BlastBomb>;
}

/// Visits affected floor cells in deterministic order.
///
/// A visitor may stop early to enforce a logical-work bound. The geometry
/// itself decides which immutable occupants stop Standard and Piercing rays.
pub(crate) fn visit_blast<F, V, B>(field: &F, source: BlastSource, mut visitor: V) -> ControlFlow<B>
where
    F: BlastField,
    V: FnMut(BlastVisit) -> ControlFlow<B>,
{
    if source.kind == BlastKind::Power {
        return visit_power(field, source, visitor);
    }

    visitor(BlastVisit {
        cell: source.cell,
        soft_block: false,
        unlit_oil: false,
        bomb: None,
    })?;
    for direction in [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ] {
        visit_ray(field, source, direction, &mut visitor)?;
    }
    ControlFlow::Continue(())
}

fn visit_power<F, V, B>(field: &F, source: BlastSource, mut visitor: V) -> ControlFlow<B>
where
    F: BlastField,
    V: FnMut(BlastVisit) -> ControlFlow<B>,
{
    let (width, height) = field.dimensions();
    for row in 0..height {
        for column in 0..width {
            let cell = CellId::new(column, row);
            let distance = u32::from(column.abs_diff(source.cell.column))
                + u32::from(row.abs_diff(source.cell.row));
            if distance <= u32::from(source.range) && field.is_floor(cell) {
                visitor(visit(field, cell))?;
            }
        }
    }
    ControlFlow::Continue(())
}

fn visit_ray<F, V, B>(
    field: &F,
    source: BlastSource,
    direction: Direction,
    visitor: &mut V,
) -> ControlFlow<B>
where
    F: BlastField,
    V: FnMut(BlastVisit) -> ControlFlow<B>,
{
    let mut cell = source.cell;
    for _ in 0..source.range {
        let Some(next) = adjacent(cell, direction) else {
            break;
        };
        cell = next;
        if !field.is_floor(cell) {
            break;
        }
        let mut current = visit(field, cell);
        if source.kind == BlastKind::Standard && (current.soft_block || current.unlit_oil) {
            current.bomb = None;
        }
        visitor(current)?;
        if source.kind == BlastKind::Standard && (current.soft_block || current.unlit_oil) {
            break;
        }
        if current.bomb.is_some_and(|bomb| !bomb.contact) {
            break;
        }
    }
    ControlFlow::Continue(())
}

fn visit(field: &impl BlastField, cell: CellId) -> BlastVisit {
    BlastVisit {
        cell,
        soft_block: field.has_soft_block(cell),
        unlit_oil: field.has_unlit_oil(cell),
        bomb: field.bomb_at(cell),
    }
}

#[derive(Clone, Copy)]
enum Direction {
    North,
    East,
    South,
    West,
}

fn adjacent(cell: CellId, direction: Direction) -> Option<CellId> {
    match direction {
        Direction::North => cell
            .row
            .checked_sub(1)
            .map(|row| CellId::new(cell.column, row)),
        Direction::East => cell
            .column
            .checked_add(1)
            .map(|column| CellId::new(column, cell.row)),
        Direction::South => cell
            .row
            .checked_add(1)
            .map(|row| CellId::new(cell.column, row)),
        Direction::West => cell
            .column
            .checked_sub(1)
            .map(|column| CellId::new(column, cell.row)),
    }
}

#[cfg(test)]
#[path = "blast_geometry_tests.rs"]
mod tests;

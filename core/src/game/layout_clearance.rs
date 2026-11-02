use std::collections::{BTreeSet, VecDeque};

use super::{Arena, CellId, Terrain};

pub(crate) fn lair_floor_ring(
    arena: &Arena,
    lairs: impl IntoIterator<Item = CellId>,
) -> BTreeSet<CellId> {
    lairs
        .into_iter()
        .flat_map(|lair| {
            (-1_i32..=1).flat_map(move |row_delta| {
                (-1_i32..=1).filter_map(move |column_delta| {
                    if row_delta == 0 && column_delta == 0 {
                        return None;
                    }
                    let column = i32::from(lair.column) + column_delta;
                    let row = i32::from(lair.row) + row_delta;
                    let cell = CellId::new(u16::try_from(column).ok()?, u16::try_from(row).ok()?);
                    (arena.terrain(cell) == Some(Terrain::Floor)).then_some(cell)
                })
            })
        })
        .collect()
}

pub(crate) fn lair_cardinal_exits(
    arena: &Arena,
    lairs: impl IntoIterator<Item = CellId>,
) -> Vec<CellId> {
    lairs
        .into_iter()
        .flat_map(|lair| {
            [(0_i32, -1_i32), (1, 0), (0, 1), (-1, 0)]
                .into_iter()
                .filter_map(move |(column_delta, row_delta)| {
                    let column = i32::from(lair.column) + column_delta;
                    let row = i32::from(lair.row) + row_delta;
                    let cell = CellId::new(u16::try_from(column).ok()?, u16::try_from(row).ok()?);
                    (arena.terrain(cell) == Some(Terrain::Floor)).then_some(cell)
                })
        })
        .collect()
}

pub(crate) fn corner_openings(arena: &Arena) -> Vec<CellId> {
    let right = arena.width() - 2;
    let bottom = arena.height() - 2;
    [
        (CellId::new(1, 1), (1_i32, 1_i32)),
        (CellId::new(right, 1), (-1, 1)),
        (CellId::new(1, bottom), (1, -1)),
        (CellId::new(right, bottom), (-1, -1)),
    ]
    .into_iter()
    .flat_map(|(spawn, (horizontal, vertical))| {
        [
            CellId::new(
                u16::try_from(i32::from(spawn.column) + horizontal).unwrap(),
                spawn.row,
            ),
            CellId::new(
                spawn.column,
                u16::try_from(i32::from(spawn.row) + vertical).unwrap(),
            ),
        ]
    })
    .collect()
}

pub(crate) fn exits_share_corner_network(
    arena: &Arena,
    blocked: &BTreeSet<CellId>,
    lairs: &[CellId],
) -> bool {
    if lairs.is_empty() {
        return true;
    }
    let required = corner_openings(arena)
        .into_iter()
        .chain(lair_cardinal_exits(arena, lairs.iter().copied()))
        .collect::<Vec<_>>();
    let Some(&start) = required.first() else {
        return true;
    };
    if blocked.contains(&start) {
        return false;
    }
    let mut reached = BTreeSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(cell) = queue.pop_front() {
        for neighbor in floor_neighbors(arena, cell) {
            if !blocked.contains(&neighbor) && reached.insert(neighbor) {
                queue.push_back(neighbor);
            }
        }
    }
    required.into_iter().all(|cell| reached.contains(&cell))
}

pub(crate) fn floor_neighbors(arena: &Arena, cell: CellId) -> Vec<CellId> {
    [(0_i32, -1_i32), (-1, 0), (1, 0), (0, 1)]
        .into_iter()
        .filter_map(|(column_delta, row_delta)| {
            let column = i32::from(cell.column) + column_delta;
            let row = i32::from(cell.row) + row_delta;
            let candidate = CellId::new(u16::try_from(column).ok()?, u16::try_from(row).ok()?);
            (arena.terrain(candidate) == Some(Terrain::Floor)).then_some(candidate)
        })
        .collect()
}

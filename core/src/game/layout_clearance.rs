use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque};

use super::{Arena, CellId, GeneratedArena, Terrain, lair_layout_v1};
use crate::room::BoardSize;

pub(crate) fn apply_lair_clearance(generated: &mut GeneratedArena) {
    let arena = &generated.arena;
    let ring = lair_floor_ring(arena, lair_layout_v1(arena.board()));
    let connector = minimal_connector(arena, &ring);
    let mut reserved = ring;
    for cell in connector {
        reserved.extend(reflection_orbit(arena.board(), cell));
    }
    generated.arena.remove_blocks(&reserved);
    generated
        .concealed_pickups
        .retain(|cell, _| !reserved.contains(cell));
}

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

fn minimal_connector(arena: &Arena, sources: &BTreeSet<CellId>) -> Vec<CellId> {
    let targets = corner_openings(arena).into_iter().collect::<BTreeSet<_>>();
    let mut frontier = BinaryHeap::new();
    let mut costs = BTreeMap::new();
    let mut previous = BTreeMap::new();
    for &source in sources {
        frontier.push(Reverse((0_u16, 0_u16, source)));
        costs.insert(source, (0_u16, 0_u16));
    }
    let target = loop {
        let Some(Reverse((blocks, length, cell))) = frontier.pop() else {
            return Vec::new();
        };
        if costs.get(&cell) != Some(&(blocks, length)) {
            continue;
        }
        if targets.contains(&cell) {
            break cell;
        }
        for neighbor in floor_neighbors(arena, cell) {
            let next = (
                blocks + u16::from(arena.blocks().contains(&neighbor)),
                length + 1,
            );
            if costs.get(&neighbor).is_none_or(|current| next < *current) {
                costs.insert(neighbor, next);
                previous.insert(neighbor, cell);
                frontier.push(Reverse((next.0, next.1, neighbor)));
            }
        }
    };
    let mut path = vec![target];
    while let Some(&parent) = previous.get(path.last().expect("connector path is nonempty")) {
        path.push(parent);
    }
    path
}

fn corner_openings(arena: &Arena) -> Vec<CellId> {
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

fn lair_cardinal_exits(arena: &Arena, lairs: impl IntoIterator<Item = CellId>) -> Vec<CellId> {
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

fn reflection_orbit(board: BoardSize, cell: CellId) -> BTreeSet<CellId> {
    BTreeSet::from([
        cell,
        CellId::new(board.width() - 1 - cell.column, cell.row),
        CellId::new(cell.column, board.height() - 1 - cell.row),
        CellId::new(
            board.width() - 1 - cell.column,
            board.height() - 1 - cell.row,
        ),
    ])
}

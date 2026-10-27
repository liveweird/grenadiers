use std::collections::BTreeSet;

use super::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, ArenaError, Terrain};
use crate::game::types::CellId;

fn open_terrain() -> [Terrain; ARENA_CELL_COUNT] {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || column == ARENA_WIDTH - 1 || row == 0 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row * ARENA_WIDTH + column)] = Terrain::Wall;
            }
        }
    }
    terrain
}

fn spawns() -> [CellId; 2] {
    [CellId::new(1, 1), CellId::new(13, 11)]
}

#[test]
fn accepts_opposite_corner_spawns_in_either_seat_order() {
    assert!(Arena::new(open_terrain(), BTreeSet::new(), spawns()).is_ok());
    assert!(
        Arena::new(
            open_terrain(),
            BTreeSet::new(),
            [CellId::new(13, 11), CellId::new(1, 1)]
        )
        .is_ok()
    );
}

#[test]
fn accepts_three_or_four_distinct_interior_corners_with_clear_escapes() {
    let corners = [
        CellId::new(1, 1),
        CellId::new(13, 1),
        CellId::new(1, 11),
        CellId::new(13, 11),
    ];
    assert!(Arena::new(open_terrain(), BTreeSet::new(), corners[..3].to_vec()).is_ok());
    assert!(Arena::new(open_terrain(), BTreeSet::new(), corners).is_ok());

    let blocks = BTreeSet::from([CellId::new(12, 1)]);
    assert_eq!(
        Arena::new(open_terrain(), blocks, corners),
        Err(ArenaError::ReservedCellHasBlock(CellId::new(12, 1)))
    );
}

#[test]
fn terrain_lookup_is_checked_at_arena_bounds() {
    let arena = Arena::new(open_terrain(), BTreeSet::new(), spawns()).unwrap();

    assert_eq!(arena.terrain(CellId::new(14, 12)), Some(Terrain::Wall));
    assert_eq!(arena.terrain(CellId::new(15, 12)), None);
    assert_eq!(arena.terrain(CellId::new(14, 13)), None);
}

#[test]
fn rejects_a_gap_in_any_border() {
    let mut terrain = open_terrain();
    terrain[7] = Terrain::Floor;

    assert_eq!(
        Arena::new(terrain, BTreeSet::new(), spawns()),
        Err(ArenaError::BorderMustBeWall(CellId::new(7, 0)))
    );
}

#[test]
fn rejects_illegal_or_duplicate_spawns() {
    assert_eq!(
        Arena::new(
            open_terrain(),
            BTreeSet::new(),
            [CellId::new(1, 1), CellId::new(1, 1)]
        ),
        Err(ArenaError::DuplicateSpawn(CellId::new(1, 1)))
    );
    assert_eq!(
        Arena::new(
            open_terrain(),
            BTreeSet::new(),
            [CellId::new(1, 1), CellId::new(12, 11)]
        ),
        Err(ArenaError::SpawnsMustBeOppositeCorners)
    );

    let mut terrain = open_terrain();
    terrain[usize::from(ARENA_WIDTH + 1)] = Terrain::Wall;
    assert_eq!(
        Arena::new(terrain, BTreeSet::new(), spawns()),
        Err(ArenaError::SpawnMustBeFloor(CellId::new(1, 1)))
    );
}

#[test]
fn rejects_walls_or_blocks_in_reserved_spawn_openings() {
    let mut terrain = open_terrain();
    terrain[usize::from(ARENA_WIDTH + 2)] = Terrain::Wall;
    assert_eq!(
        Arena::new(terrain, BTreeSet::new(), spawns()),
        Err(ArenaError::ReservedCellMustBeFloor(CellId::new(2, 1)))
    );

    let blocks = BTreeSet::from([CellId::new(13, 10)]);
    assert_eq!(
        Arena::new(open_terrain(), blocks, spawns()),
        Err(ArenaError::ReservedCellHasBlock(CellId::new(13, 10)))
    );
}

#[test]
fn rejects_soft_blocks_outside_floor_cells() {
    assert_eq!(
        Arena::new(
            open_terrain(),
            BTreeSet::from([CellId::new(15, 1)]),
            spawns()
        ),
        Err(ArenaError::BlockOutOfBounds(CellId::new(15, 1)))
    );
    assert_eq!(
        Arena::new(
            open_terrain(),
            BTreeSet::from([CellId::new(0, 1)]),
            spawns()
        ),
        Err(ArenaError::BlockMustBeOnFloor(CellId::new(0, 1)))
    );
}

#[test]
fn soft_blocks_do_not_affect_underlying_floor_connectivity() {
    let blocks = (1..ARENA_WIDTH - 1)
        .map(|column| CellId::new(column, 6))
        .collect();

    assert!(Arena::new(open_terrain(), blocks, spawns()).is_ok());
}

#[test]
fn accepts_custom_connected_interior_walls_without_requiring_a_pillar_lattice() {
    let mut terrain = open_terrain();
    terrain[usize::from(5 * ARENA_WIDTH + 7)] = Terrain::Wall;

    assert!(Arena::new(terrain, BTreeSet::new(), spawns()).is_ok());
}

#[test]
fn rejects_disconnected_floor_regions() {
    let mut terrain = open_terrain();
    for cell in [
        CellId::new(6, 5),
        CellId::new(7, 4),
        CellId::new(8, 5),
        CellId::new(7, 6),
    ] {
        terrain[usize::from(cell.row * ARENA_WIDTH + cell.column)] = Terrain::Wall;
    }

    assert_eq!(
        Arena::new(terrain, BTreeSet::new(), spawns()),
        Err(ArenaError::FloorIsDisconnected(CellId::new(7, 5)))
    );
}

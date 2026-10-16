use std::collections::{BTreeMap, BTreeSet};

use super::plan;
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::bombs::Bomb;
use crate::game::types::{CellId, EntityId, Tick};
use crate::game::world::World;

fn world_with(walls: &[CellId], blocks: &[CellId]) -> World {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            let cell = CellId::new(column, row);
            if column == 0
                || row == 0
                || column == ARENA_WIDTH - 1
                || row == ARENA_HEIGHT - 1
                || walls.contains(&cell)
            {
                terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    World::new(
        Arena::new(
            terrain,
            BTreeSet::from_iter(blocks.iter().copied()),
            [CellId::new(1, 1), CellId::new(13, 11)],
        )
        .unwrap(),
    )
}

fn add_bomb(world: &mut World, id: u32, cell: CellId, range: u16, deadline: u64) {
    let id = EntityId(id);
    world.bombs.insert(
        id,
        Bomb {
            id,
            cell,
            owner: EntityId(100),
            range,
            placed_at: Tick(0),
            detonates_at: Tick(deadline),
        },
    );
}

#[test]
fn clear_range_two_blast_has_center_and_eight_ray_cells() {
    let mut world = world_with(&[], &[]);
    add_bomb(&mut world, 1, CellId::new(7, 6), 2, 1);

    let result = plan(&world, Tick(1));
    let expected = BTreeSet::from([
        CellId::new(7, 6),
        CellId::new(7, 5),
        CellId::new(7, 4),
        CellId::new(8, 6),
        CellId::new(9, 6),
        CellId::new(7, 7),
        CellId::new(7, 8),
        CellId::new(6, 6),
        CellId::new(5, 6),
    ]);
    assert_eq!(result.flames, expected);
    assert_eq!(result.detonated, BTreeSet::from([EntityId(1)]));
    assert!(result.destroyed_blocks.is_empty());
}

#[test]
fn hard_walls_exclude_flame_while_soft_blocks_burn_and_stop_rays() {
    let wall = CellId::new(8, 6);
    let block = CellId::new(6, 6);
    let mut world = world_with(&[wall], &[block]);
    add_bomb(&mut world, 1, CellId::new(7, 6), 3, 1);

    let result = plan(&world, Tick(1));
    assert!(!result.flames.contains(&wall));
    assert!(result.flames.contains(&block));
    assert!(!result.flames.contains(&CellId::new(5, 6)));
    assert_eq!(result.destroyed_blocks, BTreeSet::from([block]));
}

#[test]
fn three_bomb_chain_detonates_each_bomb_once_despite_repeated_triggers() {
    let mut world = world_with(&[], &[]);
    add_bomb(&mut world, 10, CellId::new(3, 6), 2, 1);
    add_bomb(&mut world, 20, CellId::new(5, 6), 2, 999);
    add_bomb(&mut world, 30, CellId::new(7, 6), 2, 1);

    let result = plan(&world, Tick(1));
    assert_eq!(
        result.detonated,
        BTreeSet::from([EntityId(10), EntityId(20), EntityId(30)])
    );
}

#[test]
fn processed_bombs_still_stop_later_rays_in_the_snapshot() {
    let mut world = world_with(&[], &[]);
    add_bomb(&mut world, 1, CellId::new(5, 6), 0, 1);
    add_bomb(&mut world, 2, CellId::new(3, 6), 4, 1);

    let result = plan(&world, Tick(1));
    assert!(result.flames.contains(&CellId::new(5, 6)));
    assert!(!result.flames.contains(&CellId::new(6, 6)));
    assert!(!result.flames.contains(&CellId::new(7, 6)));
}

#[test]
fn simultaneous_blasts_share_a_soft_block_without_removing_its_obstruction() {
    let block = CellId::new(5, 5);
    let mut world = world_with(&[], &[block]);
    add_bomb(&mut world, 1, CellId::new(3, 5), 4, 1);
    add_bomb(&mut world, 2, CellId::new(5, 3), 4, 1);

    let result = plan(&world, Tick(1));
    assert_eq!(result.destroyed_blocks, BTreeSet::from([block]));
    assert_eq!(result.detonated.len(), 2);
    for behind_block in [
        CellId::new(6, 5),
        CellId::new(7, 5),
        CellId::new(5, 6),
        CellId::new(5, 7),
    ] {
        assert!(!result.flames.contains(&behind_block));
    }
}

#[test]
fn geometry_is_unchanged_when_entity_ids_are_permuted() {
    fn planned(ids: [u32; 3]) -> (BTreeSet<CellId>, BTreeSet<CellId>) {
        let block = CellId::new(6, 5);
        let mut world = world_with(&[], &[block]);
        for (id, cell, deadline) in [
            (ids[0], CellId::new(3, 5), 1),
            (ids[1], CellId::new(5, 5), 999),
            (ids[2], CellId::new(5, 7), 1),
        ] {
            add_bomb(&mut world, id, cell, 2, deadline);
        }
        let result = plan(&world, Tick(1));
        (result.flames, result.destroyed_blocks)
    }

    assert_eq!(planned([1, 2, 3]), planned([30, 10, 20]));
}

#[test]
fn active_flame_seeds_a_bomb_without_repropagating_on_its_own() {
    let cell = CellId::new(7, 6);
    let mut empty = world_with(&[], &[]);
    empty.hazards = BTreeMap::from([(cell, Tick(10))]);
    assert_eq!(plan(&empty, Tick(1)), Default::default());

    add_bomb(&mut empty, 1, cell, 1, 999);
    assert_eq!(
        plan(&empty, Tick(1)).detonated,
        BTreeSet::from([EntityId(1)])
    );
}

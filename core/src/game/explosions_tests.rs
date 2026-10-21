use std::collections::{BTreeMap, BTreeSet};

use super::{plan, plan_with_contacts};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::bombs::{BlastKind, Bomb, BombTrigger};
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
    add_bomb_with_blast(world, id, cell, range, deadline, BlastKind::Standard);
}

fn add_bomb_with_blast(
    world: &mut World,
    id: u32,
    cell: CellId,
    range: u16,
    deadline: u64,
    blast: BlastKind,
) {
    let id = EntityId(id);
    world.bombs.insert(
        id,
        Bomb::new_with_blast(
            id,
            cell,
            EntityId(100),
            range,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(deadline),
            },
            blast,
        )
        .unwrap(),
    );
}

fn add_mine(world: &mut World, id: u32, cell: CellId, range: u16) {
    let id = EntityId(id);
    world.bombs.insert(
        id,
        Bomb::new(
            id,
            cell,
            EntityId(100),
            range,
            Tick(0),
            BombTrigger::Contact,
        )
        .unwrap(),
    );
}

#[test]
fn piercing_spends_range_while_crossing_consecutive_blocks() {
    let blocks = [CellId::new(4, 5), CellId::new(5, 5)];
    let mut world = world_with(&[], &blocks);
    add_bomb_with_blast(&mut world, 1, CellId::new(3, 5), 3, 1, BlastKind::Piercing);

    let result = plan(&world, Tick(1), [None; 2]);
    assert!(result.flames.contains(&CellId::new(6, 5)));
    assert!(!result.flames.contains(&CellId::new(7, 5)));
    assert_eq!(result.destroyed_blocks, BTreeSet::from(blocks));
}

#[test]
fn piercing_still_stops_without_fire_at_hard_walls() {
    let wall = CellId::new(5, 5);
    let mut world = world_with(&[wall], &[]);
    add_bomb_with_blast(&mut world, 1, CellId::new(3, 5), 4, 1, BlastKind::Piercing);

    let result = plan(&world, Tick(1), [None; 2]);
    assert!(result.flames.contains(&CellId::new(4, 5)));
    assert!(!result.flames.contains(&wall));
    assert!(!result.flames.contains(&CellId::new(6, 5)));
}

#[test]
fn power_radius_one_and_two_are_exact_manhattan_masks() {
    let center = CellId::new(7, 6);
    for (range, expected) in [
        (
            1,
            BTreeSet::from([
                center,
                CellId::new(7, 5),
                CellId::new(8, 6),
                CellId::new(7, 7),
                CellId::new(6, 6),
            ]),
        ),
        (
            2,
            BTreeSet::from([
                center,
                CellId::new(7, 4),
                CellId::new(6, 5),
                CellId::new(7, 5),
                CellId::new(8, 5),
                CellId::new(5, 6),
                CellId::new(6, 6),
                CellId::new(8, 6),
                CellId::new(9, 6),
                CellId::new(6, 7),
                CellId::new(7, 7),
                CellId::new(8, 7),
                CellId::new(7, 8),
            ]),
        ),
    ] {
        let mut world = world_with(&[], &[]);
        add_bomb_with_blast(&mut world, 1, center, range, 1, BlastKind::Power);
        assert_eq!(plan(&world, Tick(1), [None; 2]).flames, expected);
    }
}

#[test]
fn power_radius_eight_clips_at_the_arena_edge_and_excludes_distance_nine() {
    let mut world = world_with(&[], &[]);
    add_bomb_with_blast(&mut world, 1, CellId::new(1, 1), 8, 1, BlastKind::Power);

    let flames = plan(&world, Tick(1), [None; 2]).flames;
    assert_eq!(flames.len(), 45);
    assert!(flames.contains(&CellId::new(5, 5)));
    assert!(flames.contains(&CellId::new(9, 1)));
    assert!(!flames.contains(&CellId::new(9, 2)));
    assert!(!flames.contains(&CellId::new(0, 1)));
}

#[test]
fn power_treats_hard_walls_as_holes_and_burns_every_soft_block() {
    let wall = CellId::new(4, 5);
    let blocks = [CellId::new(5, 5), CellId::new(4, 6), CellId::new(3, 7)];
    let mut world = world_with(&[wall], &blocks);
    add_bomb_with_blast(&mut world, 1, CellId::new(3, 5), 3, 1, BlastKind::Power);

    let result = plan(&world, Tick(1), [None; 2]);
    assert!(!result.flames.contains(&wall));
    assert!(result.flames.contains(&CellId::new(6, 5)));
    assert!(result.flames.contains(&CellId::new(5, 6)));
    assert_eq!(result.destroyed_blocks, BTreeSet::from(blocks));
}

#[test]
fn power_queues_bombs_without_shielding_and_chains_use_their_own_blasts() {
    let wall = CellId::new(4, 5);
    let block = CellId::new(7, 5);
    let mut world = world_with(&[wall], &[block]);
    add_bomb_with_blast(&mut world, 30, CellId::new(3, 5), 3, 1, BlastKind::Power);
    add_bomb(&mut world, 20, CellId::new(5, 5), 1, 999);
    add_bomb_with_blast(
        &mut world,
        10,
        CellId::new(6, 5),
        2,
        999,
        BlastKind::Piercing,
    );

    let result = plan(&world, Tick(1), [None; 2]);
    assert_eq!(
        result.detonated,
        BTreeSet::from([EntityId(10), EntityId(20), EntityId(30)])
    );
    assert!(result.flames.contains(&CellId::new(5, 5)));
    assert!(result.flames.contains(&CellId::new(8, 5)));
    assert_eq!(result.destroyed_blocks, BTreeSet::from([block]));
}

#[test]
fn chained_bomb_stops_incoming_ray_and_uses_its_own_piercing_blast() {
    let blocks = [CellId::new(6, 5), CellId::new(7, 5)];
    let mut world = world_with(&[], &blocks);
    add_bomb(&mut world, 1, CellId::new(3, 5), 2, 1);
    add_bomb_with_blast(
        &mut world,
        2,
        CellId::new(5, 5),
        3,
        999,
        BlastKind::Piercing,
    );

    let result = plan(&world, Tick(1), [None; 2]);
    assert_eq!(result.detonated, BTreeSet::from([EntityId(1), EntityId(2)]));
    assert!(result.flames.contains(&CellId::new(8, 5)));
    assert_eq!(result.destroyed_blocks, BTreeSet::from(blocks));
}

#[test]
fn standard_ray_stops_at_block_already_marked_by_piercing_ray() {
    let block = CellId::new(5, 5);
    let mut world = world_with(&[], &[block]);
    add_bomb_with_blast(&mut world, 1, CellId::new(5, 3), 3, 1, BlastKind::Piercing);
    add_bomb(&mut world, 2, CellId::new(3, 5), 3, 1);

    let result = plan(&world, Tick(1), [None; 2]);
    assert!(result.flames.contains(&CellId::new(5, 6)));
    assert!(!result.flames.contains(&CellId::new(6, 5)));
    assert_eq!(result.destroyed_blocks, BTreeSet::from([block]));
}

#[test]
fn clear_range_two_blast_has_center_and_eight_ray_cells() {
    let mut world = world_with(&[], &[]);
    add_bomb(&mut world, 1, CellId::new(7, 6), 2, 1);

    let result = plan(&world, Tick(1), [None; 2]);
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

    let result = plan(&world, Tick(1), [None; 2]);
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

    let result = plan(&world, Tick(1), [None; 2]);
    assert_eq!(
        result.detonated,
        BTreeSet::from([EntityId(10), EntityId(20), EntityId(30)])
    );
}

#[test]
fn processed_bombs_still_stop_later_rays_in_the_snapshot() {
    let mut world = world_with(&[], &[]);
    add_bomb(&mut world, 1, CellId::new(5, 6), 0, 1);
    add_bomb_with_blast(&mut world, 2, CellId::new(3, 6), 4, 1, BlastKind::Piercing);

    let result = plan(&world, Tick(1), [None; 2]);
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

    let result = plan(&world, Tick(1), [None; 2]);
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
        for (id, cell, deadline, blast) in [
            (ids[0], CellId::new(3, 5), 1, BlastKind::Power),
            (ids[1], CellId::new(5, 5), 999, BlastKind::Standard),
            (ids[2], CellId::new(5, 7), 1, BlastKind::Standard),
        ] {
            add_bomb_with_blast(&mut world, id, cell, 2, deadline, blast);
        }
        let result = plan(&world, Tick(1), [None; 2]);
        (result.flames, result.destroyed_blocks)
    }

    assert_eq!(planned([1, 2, 3]), planned([30, 10, 20]));
}

#[test]
fn active_flame_seeds_a_bomb_without_repropagating_on_its_own() {
    let cell = CellId::new(7, 6);
    let mut empty = world_with(&[], &[]);
    empty.hazards = BTreeMap::from([(cell, Tick(10))]);
    assert_eq!(plan(&empty, Tick(1), [None; 2]), Default::default());

    add_bomb(&mut empty, 1, cell, 1, 999);
    assert_eq!(
        plan(&empty, Tick(1), [None; 2]).detonated,
        BTreeSet::from([EntityId(1)])
    );
}

#[test]
fn blast_destroys_unaccepted_mine_and_continues_through_its_cell() {
    let mut world = world_with(&[], &[]);
    add_bomb(&mut world, 1, CellId::new(3, 5), 4, 1);
    add_mine(&mut world, 2, CellId::new(5, 5), 4);

    let result = plan(&world, Tick(1), [None; 2]);
    assert_eq!(result.detonated, BTreeSet::from([EntityId(1)]));
    assert_eq!(result.destroyed_mines, BTreeSet::from([EntityId(2)]));
    assert!(result.flames.contains(&CellId::new(7, 5)));
}

#[test]
fn accepted_contact_wins_over_fire_and_chains_timed_bombs_without_chaining_mines() {
    let mut world = world_with(&[], &[]);
    add_mine(&mut world, 20, CellId::new(5, 5), 2);
    add_mine(&mut world, 10, CellId::new(7, 5), 2);
    add_bomb(&mut world, 30, CellId::new(3, 5), 2, 999);
    world.hazards.insert(CellId::new(5, 5), Tick(10));

    let result = plan_with_contacts(&world, Tick(1), [None; 2], &BTreeSet::from([EntityId(20)]));
    assert_eq!(
        result.detonated,
        BTreeSet::from([EntityId(20), EntityId(30)])
    );
    assert_eq!(result.destroyed_mines, BTreeSet::from([EntityId(10)]));
    assert!(result.flames.contains(&CellId::new(7, 5)));
}

#[test]
fn mine_destruction_and_contact_geometry_ignore_entity_id_order() {
    fn geometry(ids: [u32; 3]) -> (BTreeSet<EntityId>, BTreeSet<CellId>) {
        let mut world = world_with(&[], &[]);
        add_bomb(&mut world, ids[0], CellId::new(3, 5), 4, 1);
        add_mine(&mut world, ids[1], CellId::new(5, 5), 2);
        add_bomb(&mut world, ids[2], CellId::new(7, 5), 2, 999);
        let plan = plan(&world, Tick(1), [None; 2]);
        (plan.destroyed_mines, plan.flames)
    }

    let low = geometry([1, 2, 3]);
    let high = geometry([30, 10, 20]);
    assert_eq!(low.1, high.1);
    assert_eq!(low.0.len(), high.0.len());
}

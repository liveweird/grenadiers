use std::collections::BTreeSet;

use super::{ExplosionError, plan};
use crate::game::{Bomb, BombTrigger, CellId, EntityId, Tick, World, fixed_arena};

fn add_bomb(world: &mut World, id: u32, owner: u32, cell: CellId, trigger: BombTrigger) {
    let id = EntityId(id);
    world.bombs.insert(
        id,
        Bomb::new(id, cell, EntityId(owner), 2, Tick(0), trigger).unwrap(),
    );
}

#[test]
fn remote_targets_share_one_immutable_plan_with_due_fire_and_chain_seeds() {
    let mut world = World::new(fixed_arena());
    add_bomb(&mut world, 4, 100, CellId::new(3, 5), BombTrigger::Remote);
    add_bomb(
        &mut world,
        5,
        100,
        CellId::new(5, 5),
        BombTrigger::Timed {
            detonates_at: Tick(99),
        },
    );
    add_bomb(&mut world, 6, 101, CellId::new(7, 5), BombTrigger::Remote);
    world.hazards.insert(CellId::new(5, 5), Tick(2));

    let planned = plan(&world, Tick(1), [Some(EntityId(4)), Some(EntityId(4))]).unwrap();
    assert_eq!(
        planned.detonated,
        BTreeSet::from([EntityId(4), EntityId(5), EntityId(6)])
    );
    assert_eq!(world.bombs.len(), 3);
}

#[test]
fn missing_and_timed_manual_targets_cannot_create_phantom_detonations() {
    let mut world = World::new(fixed_arena());
    add_bomb(
        &mut world,
        5,
        100,
        CellId::new(5, 5),
        BombTrigger::Timed {
            detonates_at: Tick(99),
        },
    );
    assert!(
        plan(&world, Tick(1), [Some(EntityId(5)), Some(EntityId(99))])
            .unwrap()
            .detonated
            .is_empty()
    );
}

#[test]
fn planner_accepts_dormant_remote_but_rejects_contact_before_planning() {
    let mut remote = World::new(fixed_arena());
    add_bomb(&mut remote, 1, 100, CellId::new(7, 6), BombTrigger::Remote);
    assert_eq!(
        plan(&remote, Tick(1), [None; 2]).unwrap(),
        Default::default()
    );

    let mut contact = World::new(fixed_arena());
    add_bomb(
        &mut contact,
        1,
        100,
        CellId::new(7, 6),
        BombTrigger::Contact,
    );
    assert_eq!(
        plan(&contact, Tick(1), [None; 2]),
        Err(ExplosionError::UnsupportedBombTrigger(EntityId(1)))
    );
}

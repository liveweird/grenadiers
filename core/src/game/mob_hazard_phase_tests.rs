use std::collections::{BTreeMap, BTreeSet};

use super::mob_hazards::{TimedMobHazards, WorkBudget};
use super::{Arena, Bomb, BombTrigger, CellId, EntityId, OilState, Tick, World};

fn arena(blocks: BTreeSet<CellId>) -> Arena {
    Arena::new(
        super::fixtures::classic_terrain(),
        blocks,
        super::fixtures::classic_spawns(),
    )
    .unwrap()
}

fn bomb(id: u32, cell: CellId, range: u16, deadline: u64) -> Bomb {
    Bomb::new(
        EntityId(id),
        cell,
        EntityId(10),
        range,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(deadline),
        },
    )
    .unwrap()
}

fn model(mut world: World, bombs: impl IntoIterator<Item = Bomb>) -> TimedMobHazards {
    for bomb in bombs {
        world.bombs.insert(bomb.id(), bomb);
    }
    let mut budget = WorkBudget::new();
    TimedMobHazards::build(&world, Tick(1), &mut budget).unwrap()
}

#[test]
fn same_phase_intervening_bomb_shields_later_bomb_from_false_early_chain() {
    let hazards = model(
        World::new(arena(BTreeSet::new())),
        [
            bomb(1, CellId::new(1, 5), 5, 50),
            bomb(2, CellId::new(3, 5), 0, 100),
            bomb(3, CellId::new(5, 5), 3, 100),
        ],
    );

    assert_eq!(hazards.deadline(CellId::new(3, 5)), Some(Tick(50)));
    assert_eq!(hazards.deadline(CellId::new(5, 5)), Some(Tick(100)));
}

#[test]
fn unremoved_soft_block_does_not_invent_an_early_chain() {
    let hazards = model(
        World::new(arena(BTreeSet::from([CellId::new(5, 1)]))),
        [
            bomb(1, CellId::new(7, 1), 5, 50),
            bomb(2, CellId::new(3, 1), 3, 100),
        ],
    );

    assert_eq!(hazards.deadline(CellId::new(3, 1)), Some(Tick(100)));
}

#[test]
fn unremoved_unlit_oil_does_not_invent_an_early_chain() {
    let world = World::with_pickups_and_oil(
        arena(BTreeSet::new()),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(CellId::new(5, 1), OilState::Unlit)]),
    )
    .unwrap();
    let hazards = model(
        world,
        [
            bomb(1, CellId::new(7, 1), 5, 50),
            bomb(2, CellId::new(3, 1), 3, 100),
        ],
    );

    assert_eq!(hazards.deadline(CellId::new(3, 1)), Some(Tick(100)));
}

#[test]
fn blocker_removed_in_an_earlier_phase_exposes_the_later_blast() {
    let hazards = model(
        World::new(arena(BTreeSet::from([CellId::new(5, 1)]))),
        [
            bomb(1, CellId::new(5, 3), 2, 50),
            bomb(2, CellId::new(7, 1), 5, 200),
        ],
    );

    assert_eq!(hazards.deadline(CellId::new(3, 1)), Some(Tick(200)));
}

#[test]
fn non_monotonic_oil_states_use_the_conservative_potential_model() {
    for (state, blocks) in [
        (OilState::Concealed, BTreeSet::from([CellId::new(5, 1)])),
        (OilState::Pending, BTreeSet::new()),
    ] {
        let world = World::with_pickups_and_oil(
            arena(blocks),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(CellId::new(5, 1), state)]),
        )
        .unwrap();
        let mut world = world;
        for bomb in [
            bomb(1, CellId::new(7, 1), 5, 50),
            bomb(2, CellId::new(3, 1), 3, 100),
        ] {
            world.bombs.insert(bomb.id(), bomb);
        }
        let mut budget = WorkBudget::new();
        let hazards = TimedMobHazards::build(&world, Tick(1), &mut budget).unwrap();

        assert_eq!(hazards.deadline(CellId::new(3, 1)), Some(Tick(50)));
    }
}

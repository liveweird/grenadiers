use std::collections::BTreeMap;

use super::World;
use crate::game::{
    OilState,
    bombs::{Bomb, BombTrigger},
    fixtures::fixed_arena,
    generate_arena,
    pickups::{Pickup, PickupKind, PickupLayer, PickupPlacementError},
    types::{CellId, EntityId, Tick},
};

#[test]
fn a_new_world_has_independent_empty_dynamic_layers() {
    let world = World::new(fixed_arena());

    assert!(world.bombs().is_empty());
    assert!(world.exposed_pickups().is_empty());
    assert!(world.hazards().is_empty());
    assert_eq!(
        world.arena().spawns(),
        [CellId::new(1, 1), CellId::new(13, 11)]
    );
}

#[test]
fn oil_is_extracted_before_pickup_layers_and_private_debug_is_redacted() {
    let concealed = CellId::new(5, 1);
    let pending = CellId::new(3, 1);
    let world = World::with_pickups(
        fixed_arena(),
        BTreeMap::from([(concealed, PickupKind::OilBarrel)]),
        BTreeMap::from([(pending, PickupKind::OilBarrel)]),
        BTreeMap::new(),
    )
    .unwrap();
    assert!(world.pickups.iter().next().is_none());
    assert_eq!(world.oil.state(concealed), Some(OilState::Concealed));
    assert_eq!(world.oil.state(pending), Some(OilState::Pending));
    assert_eq!(
        format!("{:?}", world.oil),
        "OilBarrels { sites: \"<redacted>\" }"
    );
}

#[test]
fn oil_rejects_exposed_and_more_than_eight_lifetime_sites() {
    let exposed = CellId::new(3, 1);
    assert_eq!(
        World::with_pickups(
            fixed_arena(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(exposed, PickupKind::OilBarrel)]),
        ),
        Err(PickupPlacementError::ExposedOilBarrel(exposed))
    );
    let sites = (1..=9)
        .map(|column| (CellId::new(column, 3), OilState::Unlit))
        .collect();
    assert_eq!(
        World::with_pickups_and_oil(
            fixed_arena(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            sites,
        ),
        Err(PickupPlacementError::TooManyOilBarrels)
    );
}

#[test]
fn layers_can_share_a_floor_cell_without_becoming_the_same_state() {
    let mut world = World::new(fixed_arena());
    let cell = CellId::new(7, 5);
    let bomb = Bomb::new(
        EntityId(9),
        cell,
        EntityId(1),
        2,
        Tick(1),
        BombTrigger::Timed {
            detonates_at: Tick(181),
        },
    )
    .unwrap();

    world.bombs.insert(bomb.id(), bomb);
    world
        .pickups
        .exposed
        .insert(cell, Pickup::new(PickupKind::BlastRange));
    world.hazards.insert(cell, Tick(31));

    assert_eq!(world.bombs().get(&EntityId(9)), Some(&bomb));
    assert_eq!(
        world
            .exposed_pickups()
            .get(&cell)
            .map(|pickup| pickup.kind()),
        Some(PickupKind::BlastRange)
    );
    assert_eq!(world.hazards().get(&cell), Some(&Tick(31)));
}

#[test]
fn generated_world_keeps_replay_record_outside_the_world() {
    let generated = generate_arena([19; 32]);
    let expected_record = generated.record();
    let expected_count = generated.concealed_pickups().len();

    let (world, record) = World::from_generated(generated).unwrap();

    assert_eq!(record, expected_record);
    assert_eq!(world.pickups.concealed.len(), expected_count);
    assert!(world.pickups.pending.is_empty());
    assert!(world.exposed_pickups().is_empty());
}

#[test]
fn malformed_pickup_layers_are_rejected_without_a_partial_world() {
    let arena = fixed_arena();
    let floor = CellId::new(3, 1);
    let block = CellId::new(5, 1);
    let wall = CellId::new(0, 1);

    assert_eq!(
        World::with_pickups(
            arena.clone(),
            BTreeMap::from([(floor, PickupKind::BombCapacity)]),
            BTreeMap::new(),
            BTreeMap::new(),
        ),
        Err(PickupPlacementError::ConcealedWithoutBlock(floor))
    );
    assert_eq!(
        World::with_pickups(
            arena.clone(),
            BTreeMap::new(),
            BTreeMap::from([(block, PickupKind::BlastRange)]),
            BTreeMap::new(),
        ),
        Err(PickupPlacementError::RetainedBlock {
            cell: block,
            layer: PickupLayer::Pending,
        })
    );
    assert_eq!(
        World::with_pickups(
            arena.clone(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(wall, PickupKind::MovementSpeed)]),
        ),
        Err(PickupPlacementError::NotOnFloor {
            cell: wall,
            layer: PickupLayer::Exposed,
        })
    );
    assert_eq!(
        World::with_pickups(
            arena,
            BTreeMap::new(),
            BTreeMap::from([(floor, PickupKind::BombCapacity)]),
            BTreeMap::from([(floor, PickupKind::BlastRange)]),
        ),
        Err(PickupPlacementError::DuplicateCell(floor))
    );
}

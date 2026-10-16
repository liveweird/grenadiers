use super::World;
use crate::game::{
    bombs::Bomb,
    fixtures::fixed_arena,
    types::{CellId, EntityId, Tick},
};

#[test]
fn a_new_world_has_independent_empty_dynamic_layers() {
    let world = World::new(fixed_arena());

    assert!(world.bombs().is_empty());
    assert!(world.pickups().is_empty());
    assert!(world.hazards().is_empty());
    assert_eq!(
        world.arena().spawns(),
        [CellId::new(1, 1), CellId::new(13, 11)]
    );
}

#[test]
fn layers_can_share_a_floor_cell_without_becoming_the_same_state() {
    let mut world = World::new(fixed_arena());
    let cell = CellId::new(7, 5);
    let bomb = Bomb {
        id: EntityId(9),
        cell,
        owner: EntityId(1),
        range: 2,
        placed_at: Tick(1),
        detonates_at: Tick(181),
    };

    world.bombs.insert(bomb.id(), bomb);
    world.pickups.insert(cell);
    world.hazards.insert(cell, Tick(31));

    assert_eq!(world.bombs().get(&EntityId(9)), Some(&bomb));
    assert!(world.pickups().contains(&cell));
    assert_eq!(world.hazards().get(&cell), Some(&Tick(31)));
}

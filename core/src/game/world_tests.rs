use super::{BombObstacle, World};
use crate::game::{
    fixtures::fixed_arena,
    types::{CellId, EntityId},
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
    let bomb = BombObstacle {
        id: EntityId(9),
        cell,
    };

    world.bombs.insert(bomb.id, bomb);
    world.pickups.insert(cell);
    world.hazards.insert(cell);

    assert_eq!(world.bombs().get(&EntityId(9)), Some(&bomb));
    assert!(world.pickups().contains(&cell));
    assert!(world.hazards().contains(&cell));
}

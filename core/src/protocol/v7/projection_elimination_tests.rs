use crate::protocol::{Cell, EntityId, Seat, Tick, WireRoundResult};

use super::projection_test_support::base_world;
use super::projection_validation::validate_world;
use super::*;

#[test]
fn elimination_rejects_each_transient_state_that_is_valid_while_alive() {
    let items = ItemSet::new([
        ItemId::Heart,
        ItemId::RemoteBomb,
        ItemId::InvisibilityCloak,
        ItemId::InvisibleBombs,
    ])
    .unwrap();
    for category in 0..5 {
        let mut world = base_world(10);
        world.available_items = items;
        world.enabled_items = items;
        let actor = &mut world.actors[1];
        match category {
            0 => actor.inventory.equipment = Some(EquipmentId::Heart),
            1 => actor.inventory.weapon = Some(WeaponId::RemoteBomb),
            2 | 3 => {
                let kind = if category == 2 {
                    EffectKind::InvisibilityCloak
                } else {
                    EffectKind::InvisibleBombs
                };
                actor.effects = EffectViews::new(vec![EffectView::new(kind, 10).unwrap()]).unwrap();
            }
            4 => actor.damage_grace_remaining_ticks = DamageGraceTicks::new(60).unwrap(),
            _ => unreachable!(),
        }
        assert!(validate_world(world.clone()).is_ok(), "category {category}");
        world.actors[1].alive = false;
        world.result = Some(WireRoundResult::Winner { seat: Seat::One });
        assert_eq!(
            validate_world(world),
            Err(ProjectionError::InvalidActors),
            "category {category}"
        );
    }
}

#[test]
fn elimination_allows_bombs_to_outlive_their_owner() {
    let mut world = base_world(10);
    world.actors[1].alive = false;
    world.actors[1].live_bomb_count = 1;
    world.result = Some(WireRoundResult::Winner { seat: Seat::One });
    world.bombs.push(BombView {
        id: EntityId::new(3),
        cell: Cell::new(2, 2).unwrap(),
        owner: world.actors[1].id,
        range: 2,
        placed_at: Tick::new(9).unwrap(),
        trigger: BombTrigger::timed(10, false).unwrap(),
        blast: BlastKind::Standard,
    });
    assert!(validate_world(world).is_ok());
}

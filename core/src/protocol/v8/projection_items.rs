use super::ItemId;

pub(super) use crate::protocol::item_foundation::projection_items::{
    effect_item, equipment_item, pickup_item, push_item, weapon_item,
};

pub(super) fn validate_world_items(
    world: &super::projection_facts::WorldFacts,
) -> Result<(), super::ProjectionError> {
    if !world.enabled_items.is_subset_of(world.available_items) {
        return Err(super::ProjectionError::InvalidItems);
    }
    let mut required = Vec::new();
    for actor in &world.actors {
        if let Some(item) = actor.inventory.equipment {
            push_item(&mut required, equipment_item(item));
        }
        if let Some(item) = actor.inventory.weapon {
            push_item(&mut required, weapon_item(item));
        }
        for effect in actor.effects.as_slice() {
            push_item(&mut required, effect_item(effect.kind()));
        }
        if actor.damage_grace_remaining_ticks.get() > 0 {
            push_item(&mut required, ItemId::Heart);
        }
    }
    for bomb in &world.bombs {
        match bomb.trigger.observation() {
            super::BombTriggerObservation::Remote => {
                push_item(&mut required, ItemId::RemoteBomb);
            }
            super::BombTriggerObservation::Contact => {
                push_item(&mut required, ItemId::Landmine);
            }
            super::BombTriggerObservation::Timed { paused: true, .. } => {
                push_item(&mut required, ItemId::Stopwatch);
            }
            super::BombTriggerObservation::Timed { paused: false, .. } => {}
        }
        match bomb.blast {
            super::BlastKind::Piercing => push_item(&mut required, ItemId::PiercingBomb),
            super::BlastKind::Power => push_item(&mut required, ItemId::PowerBomb),
            super::BlastKind::Standard => {}
        }
    }
    for pickup in &world.pickups {
        if let Some(item) = pickup_item(pickup.kind) {
            push_item(&mut required, item);
        }
    }
    required
        .into_iter()
        .all(|item| world.enabled_items.contains(item))
        .then_some(())
        .ok_or(super::ProjectionError::InvalidItems)
}

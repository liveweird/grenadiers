use crate::game::RulesConfig;
use crate::protocol::item_foundation::projection_items::{
    effect_item, equipment_item, pickup_item, weapon_item,
};

use super::projection_facts::{TriggerHistory, WorldFacts};
use super::{BlastKind, ItemId, ProjectionError};

pub(crate) fn validate(world: &WorldFacts, rules: &RulesConfig) -> Result<(), ProjectionError> {
    if world.rules_version != rules.version() as u8
        || world.enabled_items.mask().bits() != rules.enabled_items().bits()
        || !world.enabled_items.is_subset_of(world.available_items)
    {
        return Err(ProjectionError::InvalidItems);
    }
    let mut required = Vec::new();
    for actor in &world.actors {
        if let Some(value) = actor.view.inventory.equipment {
            required.push(equipment_item(value));
        }
        if let Some(value) = actor.view.inventory.weapon {
            required.push(weapon_item(value));
        }
        for effect in actor.view.effects.as_slice() {
            required.push(effect_item(effect.kind()));
        }
        if actor.view.damage_grace_remaining_ticks.get() > 0 {
            required.push(ItemId::Heart);
        }
    }
    for (bomb, history) in world.bombs.iter().zip(&world.bomb_history) {
        match history.trigger {
            TriggerHistory::Remote => required.push(ItemId::RemoteBomb),
            TriggerHistory::Contact => required.push(ItemId::Landmine),
            TriggerHistory::Timed { paused: true, .. } => required.push(ItemId::Stopwatch),
            TriggerHistory::Timed { paused: false, .. } => {}
        }
        match bomb.blast {
            BlastKind::Piercing => required.push(ItemId::PiercingBomb),
            BlastKind::Power => required.push(ItemId::PowerBomb),
            BlastKind::Standard => {}
        }
        if history.motion.is_some() || history.has_been_kicked {
            required.push(ItemId::Kick);
        }
        if history.has_been_punched {
            required.push(ItemId::Punch);
        }
    }
    for pickup in &world.private_pickups {
        if let Some(item) = pickup_item(pickup.kind) {
            required.push(item);
        }
    }
    required
        .into_iter()
        .all(|item| world.enabled_items.contains(item))
        .then_some(())
        .ok_or(ProjectionError::InvalidItems)
}

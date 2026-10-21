use super::{EffectKind, EquipmentId, ItemId, PickupKind, WeaponId};

pub(super) fn equipment_item(item: EquipmentId) -> ItemId {
    match item {
        EquipmentId::Kick => ItemId::Kick,
        EquipmentId::Punch => ItemId::Punch,
        EquipmentId::Bracelet => ItemId::Bracelet,
        EquipmentId::WallPass => ItemId::WallPass,
        EquipmentId::Heart => ItemId::Heart,
    }
}

pub(super) fn weapon_item(item: WeaponId) -> ItemId {
    match item {
        WeaponId::RemoteBomb => ItemId::RemoteBomb,
        WeaponId::PiercingBomb => ItemId::PiercingBomb,
        WeaponId::PowerBomb => ItemId::PowerBomb,
        WeaponId::Landmine => ItemId::Landmine,
    }
}

pub(super) fn effect_item(kind: EffectKind) -> ItemId {
    match kind {
        EffectKind::Shield => ItemId::Shield,
        EffectKind::Stopwatch => ItemId::Stopwatch,
        EffectKind::InvisibilityCloak => ItemId::InvisibilityCloak,
        EffectKind::GoldenFlame => ItemId::GoldenFlame,
        EffectKind::ShortFuse => ItemId::ShortFuse,
        EffectKind::Diarrhea => ItemId::Diarrhea,
        EffectKind::SlowFoot => ItemId::SlowFoot,
        EffectKind::ShortFlame => ItemId::ShortFlame,
        EffectKind::LongFuse => ItemId::LongFuse,
        EffectKind::ReverseControls => ItemId::ReverseControls,
        EffectKind::InvisibleBombs => ItemId::InvisibleBombs,
    }
}

pub(super) fn pickup_item(kind: PickupKind) -> Option<ItemId> {
    match kind {
        PickupKind::BombCapacity | PickupKind::BlastRange | PickupKind::MovementSpeed => None,
        PickupKind::Kick => Some(ItemId::Kick),
        PickupKind::Punch => Some(ItemId::Punch),
        PickupKind::Bracelet => Some(ItemId::Bracelet),
        PickupKind::WallPass => Some(ItemId::WallPass),
        PickupKind::Heart => Some(ItemId::Heart),
        PickupKind::RemoteBomb => Some(ItemId::RemoteBomb),
        PickupKind::PiercingBomb => Some(ItemId::PiercingBomb),
        PickupKind::PowerBomb => Some(ItemId::PowerBomb),
        PickupKind::Landmine => Some(ItemId::Landmine),
        PickupKind::Shield => Some(ItemId::Shield),
        PickupKind::Stopwatch => Some(ItemId::Stopwatch),
        PickupKind::InvisibilityCloak => Some(ItemId::InvisibilityCloak),
        PickupKind::GoldenFlame => Some(ItemId::GoldenFlame),
        PickupKind::ShortFuse => Some(ItemId::ShortFuse),
        PickupKind::Diarrhea => Some(ItemId::Diarrhea),
        PickupKind::SlowFoot => Some(ItemId::SlowFoot),
        PickupKind::ShortFlame => Some(ItemId::ShortFlame),
        PickupKind::LongFuse => Some(ItemId::LongFuse),
        PickupKind::ReverseControls => Some(ItemId::ReverseControls),
        PickupKind::InvisibleBombs => Some(ItemId::InvisibleBombs),
        PickupKind::OilBarrel => Some(ItemId::OilBarrel),
        PickupKind::Teleporter => Some(ItemId::Teleporter),
    }
}

pub(super) fn push_item(items: &mut Vec<ItemId>, item: ItemId) {
    if !items.contains(&item) {
        items.push(item);
    }
}

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

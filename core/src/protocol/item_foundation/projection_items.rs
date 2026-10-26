use super::effects::EffectKind;
use super::items::{EquipmentId, ItemId, PickupKind, WeaponId};

pub(crate) fn equipment_item(item: EquipmentId) -> ItemId {
    match item {
        EquipmentId::Kick => ItemId::Kick,
        EquipmentId::Punch => ItemId::Punch,
        EquipmentId::Bracelet => ItemId::Bracelet,
        EquipmentId::WallPass => ItemId::WallPass,
        EquipmentId::Heart => ItemId::Heart,
    }
}

pub(crate) fn weapon_item(item: WeaponId) -> ItemId {
    match item {
        WeaponId::RemoteBomb => ItemId::RemoteBomb,
        WeaponId::PiercingBomb => ItemId::PiercingBomb,
        WeaponId::PowerBomb => ItemId::PowerBomb,
        WeaponId::Landmine => ItemId::Landmine,
    }
}

pub(crate) fn effect_item(kind: EffectKind) -> ItemId {
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

macro_rules! define_pickup_projections {
    ($(($variant:ident, $bit:literal, $constant:ident, $contains:ident),)*) => {
        pub(crate) const fn pickup_item(kind: PickupKind) -> Option<ItemId> {
            match kind {
                PickupKind::BombCapacity
                | PickupKind::BlastRange
                | PickupKind::MovementSpeed => None,
                $(PickupKind::$variant => Some(ItemId::$variant),)*
            }
        }

        pub(crate) const fn wire_pickup(kind: crate::game::PickupKind) -> PickupKind {
            match kind {
                crate::game::PickupKind::BombCapacity => PickupKind::BombCapacity,
                crate::game::PickupKind::BlastRange => PickupKind::BlastRange,
                crate::game::PickupKind::MovementSpeed => PickupKind::MovementSpeed,
                $(crate::game::PickupKind::$variant => PickupKind::$variant,)*
            }
        }
    };
}

crate::item_spec::with_item_spec!(define_pickup_projections);

pub(crate) fn push_item(items: &mut Vec<ItemId>, item: ItemId) {
    if !items.contains(&item) {
        items.push(item);
    }
}

#[cfg(test)]
mod tests {
    use super::{pickup_item, wire_pickup};
    use crate::game::{EnabledItems, PickupKind as DomainPickup};
    use crate::protocol::item_foundation::items::{ItemId, PickupKind as WirePickup};

    #[test]
    fn all_pickup_projections_and_enablement_match_the_literal_golden() {
        let base = [
            (DomainPickup::BombCapacity, WirePickup::BombCapacity),
            (DomainPickup::BlastRange, WirePickup::BlastRange),
            (DomainPickup::MovementSpeed, WirePickup::MovementSpeed),
        ];
        for (domain, wire) in base {
            assert_eq!(wire_pickup(domain), wire);
            assert_eq!(pickup_item(wire), None);
            assert!(domain.enabled_by(EnabledItems::EMPTY));
        }

        let configured = [
            (
                DomainPickup::Kick,
                WirePickup::Kick,
                ItemId::Kick,
                EnabledItems::KICK,
            ),
            (
                DomainPickup::Punch,
                WirePickup::Punch,
                ItemId::Punch,
                EnabledItems::PUNCH,
            ),
            (
                DomainPickup::Bracelet,
                WirePickup::Bracelet,
                ItemId::Bracelet,
                EnabledItems::BRACELET,
            ),
            (
                DomainPickup::WallPass,
                WirePickup::WallPass,
                ItemId::WallPass,
                EnabledItems::WALL_PASS,
            ),
            (
                DomainPickup::Heart,
                WirePickup::Heart,
                ItemId::Heart,
                EnabledItems::HEART,
            ),
            (
                DomainPickup::RemoteBomb,
                WirePickup::RemoteBomb,
                ItemId::RemoteBomb,
                EnabledItems::REMOTE_BOMB,
            ),
            (
                DomainPickup::PiercingBomb,
                WirePickup::PiercingBomb,
                ItemId::PiercingBomb,
                EnabledItems::PIERCING_BOMB,
            ),
            (
                DomainPickup::PowerBomb,
                WirePickup::PowerBomb,
                ItemId::PowerBomb,
                EnabledItems::POWER_BOMB,
            ),
            (
                DomainPickup::Landmine,
                WirePickup::Landmine,
                ItemId::Landmine,
                EnabledItems::LANDMINE,
            ),
            (
                DomainPickup::Shield,
                WirePickup::Shield,
                ItemId::Shield,
                EnabledItems::SHIELD,
            ),
            (
                DomainPickup::Stopwatch,
                WirePickup::Stopwatch,
                ItemId::Stopwatch,
                EnabledItems::STOPWATCH,
            ),
            (
                DomainPickup::InvisibilityCloak,
                WirePickup::InvisibilityCloak,
                ItemId::InvisibilityCloak,
                EnabledItems::INVISIBILITY_CLOAK,
            ),
            (
                DomainPickup::GoldenFlame,
                WirePickup::GoldenFlame,
                ItemId::GoldenFlame,
                EnabledItems::GOLDEN_FLAME,
            ),
            (
                DomainPickup::ShortFuse,
                WirePickup::ShortFuse,
                ItemId::ShortFuse,
                EnabledItems::SHORT_FUSE,
            ),
            (
                DomainPickup::Diarrhea,
                WirePickup::Diarrhea,
                ItemId::Diarrhea,
                EnabledItems::DIARRHEA,
            ),
            (
                DomainPickup::SlowFoot,
                WirePickup::SlowFoot,
                ItemId::SlowFoot,
                EnabledItems::SLOW_FOOT,
            ),
            (
                DomainPickup::ShortFlame,
                WirePickup::ShortFlame,
                ItemId::ShortFlame,
                EnabledItems::SHORT_FLAME,
            ),
            (
                DomainPickup::LongFuse,
                WirePickup::LongFuse,
                ItemId::LongFuse,
                EnabledItems::LONG_FUSE,
            ),
            (
                DomainPickup::ReverseControls,
                WirePickup::ReverseControls,
                ItemId::ReverseControls,
                EnabledItems::REVERSE_CONTROLS,
            ),
            (
                DomainPickup::InvisibleBombs,
                WirePickup::InvisibleBombs,
                ItemId::InvisibleBombs,
                EnabledItems::INVISIBLE_BOMBS,
            ),
            (
                DomainPickup::OilBarrel,
                WirePickup::OilBarrel,
                ItemId::OilBarrel,
                EnabledItems::OIL_BARREL,
            ),
            (
                DomainPickup::Teleporter,
                WirePickup::Teleporter,
                ItemId::Teleporter,
                EnabledItems::TELEPORTER,
            ),
        ];
        for (selected_index, (domain, wire, item, enabled)) in configured.iter().enumerate() {
            assert_eq!(wire_pickup(*domain), *wire);
            assert_eq!(pickup_item(*wire), Some(*item));
            for (candidate_index, (candidate, _, _, _)) in configured.iter().enumerate() {
                assert_eq!(
                    candidate.enabled_by(*enabled),
                    selected_index == candidate_index
                );
            }
        }
    }
}

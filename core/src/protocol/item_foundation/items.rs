use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

const ITEM_COUNT: usize = 22;
const ALL_ITEM_BITS: u32 = (1 << ITEM_COUNT) - 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemId {
    Kick,
    Punch,
    Bracelet,
    WallPass,
    Heart,
    RemoteBomb,
    PiercingBomb,
    PowerBomb,
    Landmine,
    Shield,
    Stopwatch,
    InvisibilityCloak,
    GoldenFlame,
    ShortFuse,
    Diarrhea,
    SlowFoot,
    ShortFlame,
    LongFuse,
    ReverseControls,
    InvisibleBombs,
    OilBarrel,
    Teleporter,
}

pub const ALL_ITEMS: [ItemId; ITEM_COUNT] = [
    ItemId::Kick,
    ItemId::Punch,
    ItemId::Bracelet,
    ItemId::WallPass,
    ItemId::Heart,
    ItemId::RemoteBomb,
    ItemId::PiercingBomb,
    ItemId::PowerBomb,
    ItemId::Landmine,
    ItemId::Shield,
    ItemId::Stopwatch,
    ItemId::InvisibilityCloak,
    ItemId::GoldenFlame,
    ItemId::ShortFuse,
    ItemId::Diarrhea,
    ItemId::SlowFoot,
    ItemId::ShortFlame,
    ItemId::LongFuse,
    ItemId::ReverseControls,
    ItemId::InvisibleBombs,
    ItemId::OilBarrel,
    ItemId::Teleporter,
];

impl ItemId {
    const fn bit(self) -> u32 {
        1 << (self as u32)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct ItemMask(u32);

impl ItemMask {
    pub const fn new(bits: u32) -> Result<Self, super::super::InvalidScalar> {
        if bits & !ALL_ITEM_BITS == 0 {
            Ok(Self(bits))
        } else {
            Err(super::super::InvalidScalar)
        }
    }
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }
    #[must_use]
    pub const fn contains(self, item: ItemId) -> bool {
        self.0 & item.bit() != 0
    }
}

impl<'de> Deserialize<'de> for ItemMask {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        super::super::scalar_input::deserialize_bounded(
            deserializer,
            u64::from(ALL_ITEM_BITS),
            "item mask contains unknown bits",
        )
        .map(|bits| Self(bits as u32))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ItemSet(ItemMask);

impl ItemSet {
    #[must_use]
    pub const fn empty() -> Self {
        Self(ItemMask(0))
    }

    pub fn new(
        items: impl IntoIterator<Item = ItemId>,
    ) -> Result<Self, super::super::InvalidScalar> {
        let mut bits = 0;
        for item in items {
            if bits & item.bit() != 0 {
                return Err(super::super::InvalidScalar);
            }
            bits |= item.bit();
        }
        Ok(Self(ItemMask(bits)))
    }
    #[must_use]
    pub const fn from_mask(mask: ItemMask) -> Self {
        Self(mask)
    }
    #[must_use]
    pub const fn mask(self) -> ItemMask {
        self.0
    }
    #[must_use]
    pub const fn contains(self, item: ItemId) -> bool {
        self.0.contains(item)
    }
    #[must_use]
    pub const fn is_subset_of(self, other: Self) -> bool {
        self.0.bits() & !other.0.bits() == 0
    }
    pub fn iter(self) -> impl Iterator<Item = ItemId> {
        ALL_ITEMS
            .into_iter()
            .filter(move |item| self.contains(*item))
    }
}

impl Serialize for ItemSet {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.iter().collect::<Vec<_>>().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ItemSet {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let items = Vec::<ItemId>::deserialize(deserializer)?;
        if items.len() > ITEM_COUNT {
            return Err(de::Error::custom("too many item ids"));
        }
        Self::new(items).map_err(|_| de::Error::custom("duplicate item id"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EquipmentId {
    Kick,
    Punch,
    Bracelet,
    WallPass,
    Heart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WeaponId {
    RemoteBomb,
    PiercingBomb,
    PowerBomb,
    Landmine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InventoryView {
    #[serde(deserialize_with = "required_equipment")]
    pub equipment: Option<EquipmentId>,
    #[serde(deserialize_with = "required_weapon")]
    pub weapon: Option<WeaponId>,
}

fn required_equipment<'de, D: Deserializer<'de>>(d: D) -> Result<Option<EquipmentId>, D::Error> {
    Option::deserialize(d)
}
fn required_weapon<'de, D: Deserializer<'de>>(d: D) -> Result<Option<WeaponId>, D::Error> {
    Option::deserialize(d)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PickupKind {
    BombCapacity,
    BlastRange,
    MovementSpeed,
    Kick,
    Punch,
    Bracelet,
    WallPass,
    Heart,
    RemoteBomb,
    PiercingBomb,
    PowerBomb,
    Landmine,
    Shield,
    Stopwatch,
    InvisibilityCloak,
    GoldenFlame,
    ShortFuse,
    Diarrhea,
    SlowFoot,
    ShortFlame,
    LongFuse,
    ReverseControls,
    InvisibleBombs,
    OilBarrel,
    Teleporter,
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{ItemId, ItemMask, ItemSet, PickupKind};
    use crate::game::EnabledItems;
    use crate::protocol::item_foundation::projection_items::pickup_item;

    const FIXTURE: [(ItemId, &str, EnabledItems, u32); 22] = [
        (ItemId::Kick, "kick", EnabledItems::KICK, 1),
        (ItemId::Punch, "punch", EnabledItems::PUNCH, 2),
        (ItemId::Bracelet, "bracelet", EnabledItems::BRACELET, 4),
        (ItemId::WallPass, "wallPass", EnabledItems::WALL_PASS, 8),
        (ItemId::Heart, "heart", EnabledItems::HEART, 16),
        (
            ItemId::RemoteBomb,
            "remoteBomb",
            EnabledItems::REMOTE_BOMB,
            32,
        ),
        (
            ItemId::PiercingBomb,
            "piercingBomb",
            EnabledItems::PIERCING_BOMB,
            64,
        ),
        (
            ItemId::PowerBomb,
            "powerBomb",
            EnabledItems::POWER_BOMB,
            128,
        ),
        (ItemId::Landmine, "landmine", EnabledItems::LANDMINE, 256),
        (ItemId::Shield, "shield", EnabledItems::SHIELD, 512),
        (
            ItemId::Stopwatch,
            "stopwatch",
            EnabledItems::STOPWATCH,
            1_024,
        ),
        (
            ItemId::InvisibilityCloak,
            "invisibilityCloak",
            EnabledItems::INVISIBILITY_CLOAK,
            2_048,
        ),
        (
            ItemId::GoldenFlame,
            "goldenFlame",
            EnabledItems::GOLDEN_FLAME,
            4_096,
        ),
        (
            ItemId::ShortFuse,
            "shortFuse",
            EnabledItems::SHORT_FUSE,
            8_192,
        ),
        (ItemId::Diarrhea, "diarrhea", EnabledItems::DIARRHEA, 16_384),
        (
            ItemId::SlowFoot,
            "slowFoot",
            EnabledItems::SLOW_FOOT,
            32_768,
        ),
        (
            ItemId::ShortFlame,
            "shortFlame",
            EnabledItems::SHORT_FLAME,
            65_536,
        ),
        (
            ItemId::LongFuse,
            "longFuse",
            EnabledItems::LONG_FUSE,
            131_072,
        ),
        (
            ItemId::ReverseControls,
            "reverseControls",
            EnabledItems::REVERSE_CONTROLS,
            262_144,
        ),
        (
            ItemId::InvisibleBombs,
            "invisibleBombs",
            EnabledItems::INVISIBLE_BOMBS,
            524_288,
        ),
        (
            ItemId::OilBarrel,
            "oilBarrel",
            EnabledItems::OIL_BARREL,
            1_048_576,
        ),
        (
            ItemId::Teleporter,
            "teleporter",
            EnabledItems::TELEPORTER,
            2_097_152,
        ),
    ];

    #[test]
    fn reserved_ids_pin_literal_names_order_and_domain_bits() {
        for (item, name, domain_item, bit) in FIXTURE {
            assert_eq!(serde_json::to_value(item).unwrap(), name);
            assert_eq!(ItemSet::new([item]).unwrap().mask().bits(), bit);
            assert_eq!(domain_item.bits(), bit);
        }
        assert_eq!(EnabledItems::ALL.bits(), 4_194_303);
        assert_eq!(ItemMask::new(4_194_303).unwrap().bits(), 4_194_303);
    }

    #[test]
    fn item_set_wire_form_is_canonical_and_rejects_ambiguous_input() {
        let shuffled = ItemSet::new([
            ItemId::Teleporter,
            ItemId::Kick,
            ItemId::OilBarrel,
            ItemId::Shield,
        ])
        .unwrap();
        assert_eq!(
            serde_json::to_value(shuffled).unwrap(),
            json!(["kick", "shield", "oilBarrel", "teleporter"])
        );
        assert!(serde_json::from_value::<ItemSet>(json!(["kick", "kick"])).is_err());
        assert!(serde_json::from_value::<ItemSet>(json!(["kick", "unknown"])).is_err());
        assert!(serde_json::from_value::<ItemMask>(json!(4_194_304)).is_err());
        for pickup in [
            PickupKind::BombCapacity,
            PickupKind::BlastRange,
            PickupKind::MovementSpeed,
        ] {
            assert_eq!(pickup_item(pickup), None);
        }
    }

    #[test]
    fn retained_v7_through_v11_schemas_reserve_the_same_ids() {
        let expected: Vec<Value> = FIXTURE
            .iter()
            .map(|(_, name, _, _)| Value::String((*name).to_owned()))
            .collect();
        for source in [
            include_str!("../../../../api/gameplay-v7.schema.json"),
            include_str!("../../../../api/gameplay-v8.schema.json"),
            include_str!("../../../../api/gameplay-v9.schema.json"),
            include_str!("../../../../api/gameplay-v10.schema.json"),
            include_str!("../../../../api/gameplay-v11.schema.json"),
        ] {
            let schema: Value = serde_json::from_str(source).unwrap();
            assert_eq!(
                schema["$defs"]["ItemId"]["enum"],
                Value::Array(expected.clone())
            );
        }
    }
}

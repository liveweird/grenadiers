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
        super::scalars::deserialize_bounded(
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

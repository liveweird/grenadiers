/// Compact set of item kinds enabled for a round.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EnabledItems(u32);

impl EnabledItems {
    pub const EMPTY: Self = Self(0);
    pub const HEART: Self = Self(1 << 4);
    pub const SHIELD: Self = Self(1 << 9);
    pub const GOLDEN_FLAME: Self = Self(1 << 12);
    pub const HEART_AND_SHIELD: Self = Self(Self::HEART.0 | Self::SHIELD.0);
    pub const HEART_AND_GOLDEN_FLAME: Self = Self(Self::HEART.0 | Self::GOLDEN_FLAME.0);
    pub const SHIELD_AND_GOLDEN_FLAME: Self = Self(Self::SHIELD.0 | Self::GOLDEN_FLAME.0);
    pub const ALL: Self = Self(Self::HEART.0 | Self::SHIELD.0 | Self::GOLDEN_FLAME.0);

    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    #[must_use]
    pub const fn from_bits(bits: u32) -> Option<Self> {
        match bits {
            0 | 16 | 512 | 528 | 4096 | 4112 | 4608 | 4624 => Some(Self(bits)),
            _ => None,
        }
    }

    #[must_use]
    pub const fn contains_heart(self) -> bool {
        self.0 & Self::HEART.0 != 0
    }

    #[must_use]
    pub const fn contains_shield(self) -> bool {
        self.0 & Self::SHIELD.0 != 0
    }

    #[must_use]
    pub const fn contains_golden_flame(self) -> bool {
        self.0 & Self::GOLDEN_FLAME.0 != 0
    }
}

#[cfg(test)]
mod tests {
    use super::EnabledItems;

    #[test]
    fn only_supported_item_combinations_are_canonical() {
        assert_eq!(EnabledItems::from_bits(0), Some(EnabledItems::EMPTY));
        assert_eq!(EnabledItems::from_bits(16), Some(EnabledItems::HEART));
        assert_eq!(EnabledItems::from_bits(512), Some(EnabledItems::SHIELD));
        assert_eq!(
            EnabledItems::from_bits(528),
            Some(EnabledItems::HEART_AND_SHIELD)
        );
        assert_eq!(
            EnabledItems::from_bits(4096),
            Some(EnabledItems::GOLDEN_FLAME)
        );
        assert_eq!(
            EnabledItems::from_bits(4112),
            Some(EnabledItems::HEART_AND_GOLDEN_FLAME)
        );
        assert_eq!(
            EnabledItems::from_bits(4608),
            Some(EnabledItems::SHIELD_AND_GOLDEN_FLAME)
        );
        assert_eq!(EnabledItems::from_bits(4624), Some(EnabledItems::ALL));
        assert_eq!(EnabledItems::from_bits(1), None);
        assert_eq!(EnabledItems::from_bits(17), None);
        assert_eq!(EnabledItems::from_bits(1 << 10), None);
        assert!(!EnabledItems::EMPTY.contains_heart());
        assert!(EnabledItems::HEART.contains_heart());
        assert!(!EnabledItems::HEART.contains_shield());
        assert!(EnabledItems::SHIELD.contains_shield());
        assert!(EnabledItems::HEART_AND_SHIELD.contains_heart());
        assert!(EnabledItems::HEART_AND_SHIELD.contains_shield());
        assert!(!EnabledItems::SHIELD.contains_golden_flame());
        assert!(EnabledItems::GOLDEN_FLAME.contains_golden_flame());
    }
}

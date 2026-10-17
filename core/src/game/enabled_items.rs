/// Compact set of item kinds enabled for a round.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EnabledItems(u32);

impl EnabledItems {
    pub const EMPTY: Self = Self(0);
    pub const HEART: Self = Self(1 << 4);

    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    #[must_use]
    pub const fn from_bits(bits: u32) -> Option<Self> {
        match bits {
            0 | 16 => Some(Self(bits)),
            _ => None,
        }
    }

    #[must_use]
    pub const fn contains_heart(self) -> bool {
        self.0 & Self::HEART.0 != 0
    }
}

#[cfg(test)]
mod tests {
    use super::EnabledItems;

    #[test]
    fn only_empty_and_heart_masks_are_canonical() {
        assert_eq!(EnabledItems::from_bits(0), Some(EnabledItems::EMPTY));
        assert_eq!(EnabledItems::from_bits(16), Some(EnabledItems::HEART));
        assert_eq!(EnabledItems::from_bits(1), None);
        assert_eq!(EnabledItems::from_bits(17), None);
        assert!(!EnabledItems::EMPTY.contains_heart());
        assert!(EnabledItems::HEART.contains_heart());
    }
}

/// Compact set of item kinds enabled for a round.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EnabledItems(u32);

impl EnabledItems {
    pub const EMPTY: Self = Self(0);
    pub const WALL_PASS: Self = Self(1 << 3);
    pub const HEART: Self = Self(1 << 4);
    pub const REMOTE_BOMB: Self = Self(1 << 5);
    pub const PIERCING_BOMB: Self = Self(1 << 6);
    pub const POWER_BOMB: Self = Self(1 << 7);
    pub const LANDMINE: Self = Self(1 << 8);
    pub const SHIELD: Self = Self(1 << 9);
    pub const STOPWATCH: Self = Self(1 << 10);
    pub const INVISIBILITY_CLOAK: Self = Self(1 << 11);
    pub const GOLDEN_FLAME: Self = Self(1 << 12);
    pub const SHORT_FUSE: Self = Self(1 << 13);
    pub const DIARRHEA: Self = Self(1 << 14);
    pub const SLOW_FOOT: Self = Self(1 << 15);
    pub const SHORT_FLAME: Self = Self(1 << 16);
    pub const LONG_FUSE: Self = Self(1 << 17);
    pub const REVERSE_CONTROLS: Self = Self(1 << 18);
    pub const INVISIBLE_BOMBS: Self = Self(1 << 19);
    // Bit 20 is reserved for Oil Barrel so catalog positions remain stable.
    pub const TELEPORTER: Self = Self(1 << 21);
    pub const HEART_AND_SHIELD: Self = Self(Self::HEART.0 | Self::SHIELD.0);
    pub const HEART_AND_GOLDEN_FLAME: Self = Self(Self::HEART.0 | Self::GOLDEN_FLAME.0);
    pub const SHIELD_AND_GOLDEN_FLAME: Self = Self(Self::SHIELD.0 | Self::GOLDEN_FLAME.0);
    pub const ALL: Self = Self(
        Self::WALL_PASS.0
            | Self::HEART.0
            | Self::REMOTE_BOMB.0
            | Self::PIERCING_BOMB.0
            | Self::POWER_BOMB.0
            | Self::LANDMINE.0
            | Self::SHIELD.0
            | Self::STOPWATCH.0
            | Self::INVISIBILITY_CLOAK.0
            | Self::GOLDEN_FLAME.0
            | Self::SHORT_FUSE.0
            | Self::DIARRHEA.0
            | Self::SLOW_FOOT.0
            | Self::SHORT_FLAME.0
            | Self::LONG_FUSE.0
            | Self::REVERSE_CONTROLS.0
            | Self::INVISIBLE_BOMBS.0
            | Self::TELEPORTER.0,
    );

    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    #[must_use]
    pub const fn from_bits(bits: u32) -> Option<Self> {
        if bits & !Self::ALL.0 == 0 {
            Some(Self(bits))
        } else {
            None
        }
    }

    #[must_use]
    pub const fn contains_heart(self) -> bool {
        self.0 & Self::HEART.0 != 0
    }

    #[must_use]
    pub const fn contains_wall_pass(self) -> bool {
        self.0 & Self::WALL_PASS.0 != 0
    }

    #[must_use]
    pub const fn contains_remote_bomb(self) -> bool {
        self.0 & Self::REMOTE_BOMB.0 != 0
    }

    #[must_use]
    pub const fn contains_piercing_bomb(self) -> bool {
        self.0 & Self::PIERCING_BOMB.0 != 0
    }

    #[must_use]
    pub const fn contains_power_bomb(self) -> bool {
        self.0 & Self::POWER_BOMB.0 != 0
    }

    #[must_use]
    pub const fn contains_landmine(self) -> bool {
        self.0 & Self::LANDMINE.0 != 0
    }

    #[must_use]
    pub const fn contains_shield(self) -> bool {
        self.0 & Self::SHIELD.0 != 0
    }

    #[must_use]
    pub const fn contains_stopwatch(self) -> bool {
        self.0 & Self::STOPWATCH.0 != 0
    }

    #[must_use]
    pub const fn contains_invisibility_cloak(self) -> bool {
        self.0 & Self::INVISIBILITY_CLOAK.0 != 0
    }

    #[must_use]
    pub const fn contains_golden_flame(self) -> bool {
        self.0 & Self::GOLDEN_FLAME.0 != 0
    }

    #[must_use]
    pub const fn contains_short_fuse(self) -> bool {
        self.0 & Self::SHORT_FUSE.0 != 0
    }

    #[must_use]
    pub const fn contains_diarrhea(self) -> bool {
        self.0 & Self::DIARRHEA.0 != 0
    }

    #[must_use]
    pub const fn contains_slow_foot(self) -> bool {
        self.0 & Self::SLOW_FOOT.0 != 0
    }

    #[must_use]
    pub const fn contains_short_flame(self) -> bool {
        self.0 & Self::SHORT_FLAME.0 != 0
    }

    #[must_use]
    pub const fn contains_long_fuse(self) -> bool {
        self.0 & Self::LONG_FUSE.0 != 0
    }

    #[must_use]
    pub const fn contains_reverse_controls(self) -> bool {
        self.0 & Self::REVERSE_CONTROLS.0 != 0
    }

    #[must_use]
    pub const fn contains_invisible_bombs(self) -> bool {
        self.0 & Self::INVISIBLE_BOMBS.0 != 0
    }

    #[must_use]
    pub const fn contains_teleporter(self) -> bool {
        self.0 & Self::TELEPORTER.0 != 0
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
        assert_eq!(EnabledItems::from_bits(4624).unwrap().bits(), 4624);
        for bits in [8192, 8208, 8704, 8720, 12288, 12304, 12800, 12816] {
            assert_eq!(EnabledItems::from_bits(bits).unwrap().bits(), bits);
        }
        assert_eq!(EnabledItems::from_bits(3_145_720), Some(EnabledItems::ALL));
        assert_eq!(EnabledItems::from_bits(8), Some(EnabledItems::WALL_PASS));
        assert_eq!(EnabledItems::from_bits(1), None);
        assert_eq!(EnabledItems::from_bits(17), None);
        assert_eq!(EnabledItems::from_bits(1 << 20), None);
        assert!(!EnabledItems::EMPTY.contains_heart());
        assert!(EnabledItems::HEART.contains_heart());
        assert!(!EnabledItems::HEART.contains_shield());
        assert!(EnabledItems::SHIELD.contains_shield());
        assert!(EnabledItems::HEART_AND_SHIELD.contains_heart());
        assert!(EnabledItems::HEART_AND_SHIELD.contains_shield());
        assert!(!EnabledItems::SHIELD.contains_golden_flame());
        assert!(EnabledItems::GOLDEN_FLAME.contains_golden_flame());
        assert!(!EnabledItems::GOLDEN_FLAME.contains_short_fuse());
        assert!(EnabledItems::SHORT_FUSE.contains_short_fuse());
        assert!(!EnabledItems::SHORT_FUSE.contains_short_flame());
        assert!(EnabledItems::SHORT_FLAME.contains_short_flame());
    }

    #[test]
    fn every_subset_of_the_eighteen_opt_in_items_is_valid() {
        let item_bits: [u32; 18] = [
            8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16_384, 32_768, 65_536, 131_072,
            262_144, 524_288, 2_097_152,
        ];
        let mut observed = 0;
        for mask in 0_u32..262_144 {
            let mut bits = 0;
            for (index, bit) in item_bits.iter().enumerate() {
                if mask & (1 << index) != 0 {
                    bits |= *bit;
                }
            }
            assert_eq!(EnabledItems::from_bits(bits).unwrap().bits(), bits);
            observed += 1;
        }
        assert_eq!(observed, 262_144);
        assert_eq!(EnabledItems::ALL.bits(), 3_145_720);
        assert_eq!(EnabledItems::default(), EnabledItems::EMPTY);
    }
}

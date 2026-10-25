macro_rules! define_enabled_items {
    ($(($variant:ident, $bit:literal, $constant:ident, $contains:ident),)*) => {
        /// Compact set of item kinds enabled for a round.
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        pub struct EnabledItems(u32);

        impl EnabledItems {
            pub const EMPTY: Self = Self(0);
            $(pub const $constant: Self = Self(1 << $bit);)*
            pub const HEART_AND_SHIELD: Self = Self(Self::HEART.0 | Self::SHIELD.0);
            pub const HEART_AND_GOLDEN_FLAME: Self =
                Self(Self::HEART.0 | Self::GOLDEN_FLAME.0);
            pub const SHIELD_AND_GOLDEN_FLAME: Self =
                Self(Self::SHIELD.0 | Self::GOLDEN_FLAME.0);
            pub const ALL: Self = Self(0 $(| Self::$constant.0)*);

            #[must_use]
            pub const fn bits(self) -> u32 {
                self.0
            }

            #[must_use]
            pub const fn from_bits(bits: u32) -> Option<Self> {
                if bits & !Self::ALL.0 == 0 { Some(Self(bits)) } else { None }
            }

            $(
                #[must_use]
                pub const fn $contains(self) -> bool {
                    self.0 & Self::$constant.0 != 0
                }
            )*
        }
    };
}

crate::item_spec::with_item_spec!(define_enabled_items);

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
        assert_eq!(EnabledItems::from_bits(4_194_303), Some(EnabledItems::ALL));
        assert_eq!(EnabledItems::from_bits(8), Some(EnabledItems::WALL_PASS));
        assert_eq!(EnabledItems::from_bits(1), Some(EnabledItems::KICK));
        assert_eq!(EnabledItems::from_bits(2), Some(EnabledItems::PUNCH));
        assert_eq!(EnabledItems::from_bits(17).unwrap().bits(), 17);
        assert_eq!(
            EnabledItems::from_bits(1 << 20),
            Some(EnabledItems::OIL_BARREL)
        );
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
        assert!(!EnabledItems::KICK.contains_punch());
        assert!(EnabledItems::PUNCH.contains_punch());
    }

    #[test]
    fn every_subset_of_the_twenty_two_opt_in_items_is_valid() {
        let item_bits: [u32; 22] = [
            1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16_384, 32_768, 65_536,
            131_072, 262_144, 524_288, 1_048_576, 2_097_152,
        ];
        let mut observed = 0;
        for mask in 0_u32..4_194_304 {
            let mut bits = 0;
            for (index, bit) in item_bits.iter().enumerate() {
                if mask & (1 << index) != 0 {
                    bits |= *bit;
                }
            }
            assert_eq!(EnabledItems::from_bits(bits).unwrap().bits(), bits);
            observed += 1;
        }
        assert_eq!(observed, 4_194_304);
        assert_eq!(EnabledItems::ALL.bits(), 4_194_303);
        assert_eq!(EnabledItems::default(), EnabledItems::EMPTY);
    }

    #[test]
    fn each_single_item_bit_enables_only_its_matching_predicate() {
        type ItemCheck = (EnabledItems, fn(EnabledItems) -> bool);
        let items: [ItemCheck; 22] = [
            (EnabledItems::KICK, EnabledItems::contains_kick),
            (EnabledItems::PUNCH, EnabledItems::contains_punch),
            (EnabledItems::BRACELET, EnabledItems::contains_bracelet),
            (EnabledItems::WALL_PASS, EnabledItems::contains_wall_pass),
            (EnabledItems::HEART, EnabledItems::contains_heart),
            (
                EnabledItems::REMOTE_BOMB,
                EnabledItems::contains_remote_bomb,
            ),
            (
                EnabledItems::PIERCING_BOMB,
                EnabledItems::contains_piercing_bomb,
            ),
            (EnabledItems::POWER_BOMB, EnabledItems::contains_power_bomb),
            (EnabledItems::LANDMINE, EnabledItems::contains_landmine),
            (EnabledItems::SHIELD, EnabledItems::contains_shield),
            (EnabledItems::STOPWATCH, EnabledItems::contains_stopwatch),
            (
                EnabledItems::INVISIBILITY_CLOAK,
                EnabledItems::contains_invisibility_cloak,
            ),
            (
                EnabledItems::GOLDEN_FLAME,
                EnabledItems::contains_golden_flame,
            ),
            (EnabledItems::SHORT_FUSE, EnabledItems::contains_short_fuse),
            (EnabledItems::DIARRHEA, EnabledItems::contains_diarrhea),
            (EnabledItems::SLOW_FOOT, EnabledItems::contains_slow_foot),
            (
                EnabledItems::SHORT_FLAME,
                EnabledItems::contains_short_flame,
            ),
            (EnabledItems::LONG_FUSE, EnabledItems::contains_long_fuse),
            (
                EnabledItems::REVERSE_CONTROLS,
                EnabledItems::contains_reverse_controls,
            ),
            (
                EnabledItems::INVISIBLE_BOMBS,
                EnabledItems::contains_invisible_bombs,
            ),
            (EnabledItems::OIL_BARREL, EnabledItems::contains_oil_barrel),
            (EnabledItems::TELEPORTER, EnabledItems::contains_teleporter),
        ];
        for (selected_index, (selected, _)) in items.iter().enumerate() {
            let enabled = EnabledItems::from_bits(selected.bits()).unwrap();
            for (candidate_index, (_, contains)) in items.iter().enumerate() {
                assert_eq!(contains(enabled), selected_index == candidate_index);
            }
        }
        for (_, contains) in items {
            assert!(!contains(EnabledItems::EMPTY));
        }
        assert_eq!(EnabledItems::from_bits(1 << 22), None);
    }
}

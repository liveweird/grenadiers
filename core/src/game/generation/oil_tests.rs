use std::collections::VecDeque;

use super::*;

struct Sequence(VecDeque<u32>);

impl RandomWords for Sequence {
    fn next_u32(&mut self) -> Option<u32> {
        self.0.pop_front()
    }
}

#[test]
fn oil_has_the_exact_appended_pool_interval() {
    let mut sole = Sequence([99, 100, 114].into());
    assert_eq!(
        sample_pickup(&mut sole, EnabledItems::OIL_BARREL),
        Some(PickupKind::MovementSpeed)
    );
    assert_eq!(
        sample_pickup(&mut sole, EnabledItems::OIL_BARREL),
        Some(PickupKind::OilBarrel)
    );
    assert_eq!(
        sample_pickup(&mut sole, EnabledItems::OIL_BARREL),
        Some(PickupKind::OilBarrel)
    );

    let mut all = Sequence([424, 425, 439].into());
    assert_eq!(
        sample_pickup(&mut all, EnabledItems::ALL),
        Some(PickupKind::Bracelet)
    );
    assert_eq!(
        sample_pickup(&mut all, EnabledItems::ALL),
        Some(PickupKind::OilBarrel)
    );
    assert_eq!(
        sample_pickup(&mut all, EnabledItems::ALL),
        Some(PickupKind::OilBarrel)
    );
}

#[test]
fn oil_preserves_every_value_mapping_for_all_prior_twenty_one_item_masks() {
    let old_bits = [
        1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1_024, 2_048, 4_096, 8_192, 16_384, 32_768, 65_536,
        131_072, 262_144, 524_288, 2_097_152,
    ];
    let mut observed = 0;
    for mask in 0_u32..2_097_152 {
        let raw = old_bits
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1 << index) != 0)
            .fold(0, |bits, (_, bit)| bits | *bit);
        let before = EnabledItems::from_bits(raw).unwrap();
        let after = EnabledItems::from_bits(raw | EnabledItems::OIL_BARREL.bits()).unwrap();
        let heart = u32::from(before.contains_heart());
        let bound = 100 + heart * 25 + (before.bits().count_ones() - heart) * 15;
        assert_eq!(
            sample_pickup(&mut Sequence([bound - 1].into()), before),
            sample_pickup(&mut Sequence([bound - 1].into()), after),
            "old mapping changed for {raw}"
        );
        assert_eq!(
            sample_pickup(&mut Sequence([bound].into()), after),
            Some(PickupKind::OilBarrel)
        );
        assert_eq!(
            sample_pickup(&mut Sequence([bound + 14].into()), after),
            Some(PickupKind::OilBarrel)
        );
        observed += 1;
    }
    assert_eq!(observed, 2_097_152);
}

#[test]
fn generation_caps_oil_sites_without_removing_later_blocks() {
    let words = (0..eligible_pairs().len())
        .flat_map(|_| [0, 0, 100])
        .collect::<VecDeque<_>>();
    let (blocks, concealed) =
        generate_layers(&mut Sequence(words), EnabledItems::OIL_BARREL).unwrap();
    assert_eq!(concealed.len(), crate::game::MAX_OIL_BARRELS);
    assert!(
        concealed
            .values()
            .all(|kind| *kind == PickupKind::OilBarrel)
    );
    assert!(blocks.len() > concealed.len());
    for cell in concealed.keys() {
        assert_eq!(concealed.get(&rotate(*cell)), Some(&PickupKind::OilBarrel));
    }
}

#[test]
fn seed_31_places_the_sole_oil_pool_on_both_sides() {
    let generated = generate_expanded_arena([31; 32], EnabledItems::OIL_BARREL);
    for cell in [CellId::new(3, 1), CellId::new(11, 11)] {
        assert_eq!(
            generated.concealed_pickups().get(&cell),
            Some(&PickupKind::OilBarrel)
        );
    }
    assert!(
        generated
            .concealed_pickups()
            .values()
            .filter(|kind| **kind == PickupKind::OilBarrel)
            .count()
            <= crate::game::MAX_OIL_BARRELS
    );
}

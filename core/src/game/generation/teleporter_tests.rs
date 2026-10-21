use std::collections::{BTreeSet, VecDeque};

use super::*;

struct Sequence(VecDeque<u32>);

impl RandomWords for Sequence {
    fn next_u32(&mut self) -> Option<u32> {
        self.0.pop_front()
    }
}

#[test]
fn teleporter_weight_is_appended_after_invisible_bombs() {
    let previous_all = EnabledItems::from_bits(1_048_560).unwrap();
    assert_eq!(
        sample_pickup(&mut Sequence([349].into()), previous_all),
        Some(PickupKind::InvisibleBombs)
    );
    let mut all = Sequence([349, 350, 364].into());
    assert_eq!(
        sample_pickup(&mut all, EnabledItems::ALL),
        Some(PickupKind::InvisibleBombs)
    );
    assert_eq!(
        sample_pickup(&mut all, EnabledItems::ALL),
        Some(PickupKind::Teleporter)
    );
    assert_eq!(
        sample_pickup(&mut all, EnabledItems::ALL),
        Some(PickupKind::Teleporter)
    );
}

#[test]
fn every_pre_teleporter_mask_keeps_its_exact_pool_tail() {
    let bits = [
        16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16_384, 32_768, 65_536, 131_072,
        262_144, 524_288,
    ];
    let mut mappings = BTreeSet::new();
    for mask in 0_u32..65_536 {
        let raw = bits
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1 << index) != 0)
            .fold(0, |enabled, (_, bit)| enabled | *bit);
        let enabled = EnabledItems::from_bits(raw).unwrap();
        let heart = u32::from(enabled.contains_heart());
        let bound = 100 + heart * 25 + (enabled.bits().count_ones() - heart) * 15;
        assert_ne!(
            sample_pickup(&mut Sequence([bound - 1].into()), enabled),
            Some(PickupKind::Teleporter)
        );
        mappings.insert((raw, bound));
    }
    assert_eq!(mappings.len(), 65_536);
}

#[test]
fn seed_31_places_the_single_item_pool_teleporter_on_both_sides() {
    let generated = generate_expanded_arena([31; 32], EnabledItems::TELEPORTER);
    for cell in [CellId::new(3, 1), CellId::new(11, 11)] {
        assert_eq!(
            generated.concealed_pickups().get(&cell),
            Some(&PickupKind::Teleporter)
        );
    }
}

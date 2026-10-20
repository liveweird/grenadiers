use std::collections::{BTreeSet, VecDeque};

use super::*;

struct Sequence(VecDeque<u32>);

impl RandomWords for Sequence {
    fn next_u32(&mut self) -> Option<u32> {
        self.0.pop_front()
    }
}

struct One(Option<u32>);

impl RandomWords for One {
    fn next_u32(&mut self) -> Option<u32> {
        self.0.take()
    }
}

fn prior_pool_bound(enabled: EnabledItems) -> u32 {
    let heart = u32::from(enabled.contains_heart());
    100 + heart * 25 + (enabled.bits().count_ones() - heart) * 15
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
        let bound = prior_pool_bound(enabled);
        assert_ne!(
            sample_pickup(&mut Sequence([bound - 1].into()), enabled),
            Some(PickupKind::Teleporter)
        );
        mappings.insert((raw, bound));
    }
    assert_eq!(mappings.len(), 65_536);
}

#[test]
fn wall_pass_append_preserves_every_value_mapping_for_all_prior_seventeen_item_masks() {
    let bits = [
        16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16_384, 32_768, 65_536, 131_072,
        262_144, 524_288, 2_097_152,
    ];
    let mut mappings = BTreeSet::new();
    for mask in 0_u32..131_072 {
        let raw = bits
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1 << index) != 0)
            .fold(0, |enabled, (_, bit)| enabled | *bit);
        let before = EnabledItems::from_bits(raw).unwrap();
        let after = EnabledItems::from_bits(raw | EnabledItems::WALL_PASS.bits()).unwrap();
        let bound = prior_pool_bound(before);
        for value in 0..bound {
            assert_eq!(
                sample_pickup(&mut One(Some(value)), before),
                sample_pickup(&mut One(Some(value)), after),
                "prior mapping changed for mask {raw} at value {value}"
            );
        }
        assert_eq!(
            sample_pickup(&mut One(Some(bound)), after),
            Some(PickupKind::WallPass)
        );
        assert_eq!(
            sample_pickup(&mut One(Some(bound + 14)), after),
            Some(PickupKind::WallPass)
        );
        mappings.insert((raw, bound));
    }
    assert_eq!(mappings.len(), 131_072);
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

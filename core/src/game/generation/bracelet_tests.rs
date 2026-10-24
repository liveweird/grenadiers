use std::collections::VecDeque;

use super::super::*;

struct Sequence(VecDeque<u32>);

impl RandomWords for Sequence {
    fn next_u32(&mut self) -> Option<u32> {
        self.0.pop_front()
    }
}

fn prior_pool_bound(enabled: EnabledItems) -> u32 {
    let heart = u32::from(enabled.contains_heart());
    100 + heart * 25 + (enabled.bits().count_ones() - heart) * 15
}

#[test]
fn bracelet_has_the_exact_sole_item_pool_interval() {
    let mut source = Sequence([99, 100, 114].into());
    assert_eq!(
        sample_pickup(&mut source, EnabledItems::BRACELET),
        Some(PickupKind::MovementSpeed)
    );
    assert_eq!(
        sample_pickup(&mut source, EnabledItems::BRACELET),
        Some(PickupKind::Bracelet)
    );
    assert_eq!(
        sample_pickup(&mut source, EnabledItems::BRACELET),
        Some(PickupKind::Bracelet)
    );
}

#[test]
fn bracelet_appends_after_punch_in_the_all_item_pool() {
    let mut source = Sequence([409, 410, 424].into());
    assert_eq!(
        sample_pickup(&mut source, EnabledItems::ALL),
        Some(PickupKind::Punch)
    );
    assert_eq!(
        sample_pickup(&mut source, EnabledItems::ALL),
        Some(PickupKind::Bracelet)
    );
    assert_eq!(
        sample_pickup(&mut source, EnabledItems::ALL),
        Some(PickupKind::Bracelet)
    );
}

#[test]
fn bracelet_preserves_the_tail_for_every_historical_twenty_item_mask() {
    let historical_bits = [
        1, 2, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16_384, 32_768, 65_536,
        131_072, 262_144, 524_288, 2_097_152,
    ];
    let mut observed = 0;
    for mask in 0_u32..1_048_576 {
        let raw = historical_bits
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1 << index) != 0)
            .fold(0, |enabled, (_, bit)| enabled | *bit);
        let before = EnabledItems::from_bits(raw).unwrap();
        let after = EnabledItems::from_bits(raw | EnabledItems::BRACELET.bits()).unwrap();
        let bound = prior_pool_bound(before);
        assert_eq!(
            sample_pickup(&mut Sequence([bound - 1].into()), before),
            sample_pickup(&mut Sequence([bound - 1].into()), after),
            "historical tail changed for mask {raw}"
        );
        assert_eq!(
            sample_pickup(&mut Sequence([bound].into()), after),
            Some(PickupKind::Bracelet)
        );
        assert_eq!(
            sample_pickup(&mut Sequence([bound + 14].into()), after),
            Some(PickupKind::Bracelet)
        );
        observed += 1;
    }
    assert_eq!(observed, 1_048_576);
}

#[test]
fn seed_31_places_the_sole_bracelet_pool_on_both_sides() {
    let generated = generate_expanded_arena([31; 32], EnabledItems::BRACELET);
    for cell in [CellId::new(3, 1), CellId::new(11, 11)] {
        assert_eq!(
            generated.concealed_pickups().get(&cell),
            Some(&PickupKind::Bracelet)
        );
    }
}

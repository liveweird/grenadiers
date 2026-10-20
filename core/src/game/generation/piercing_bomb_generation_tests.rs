use std::collections::VecDeque;

use super::*;

struct Sequence(VecDeque<u32>);

impl RandomWords for Sequence {
    fn next_u32(&mut self) -> Option<u32> {
        self.0.pop_front()
    }
}

#[test]
fn piercing_weight_is_appended_after_slow_foot() {
    let mut only = Sequence([99, 100, 114].into());
    assert_eq!(
        sample_pickup(&mut only, EnabledItems::PIERCING_BOMB),
        Some(PickupKind::MovementSpeed)
    );
    assert_eq!(
        sample_pickup(&mut only, EnabledItems::PIERCING_BOMB),
        Some(PickupKind::PiercingBomb)
    );
    assert_eq!(
        sample_pickup(&mut only, EnabledItems::PIERCING_BOMB),
        Some(PickupKind::PiercingBomb)
    );

    let old_all = EnabledItems::from_bits(520_720).unwrap();
    let mut old_tail = Sequence([244].into());
    assert_eq!(
        sample_pickup(&mut old_tail, old_all),
        Some(PickupKind::SlowFoot)
    );
    let mut all = Sequence([244, 245, 259].into());
    assert_eq!(
        sample_pickup(&mut all, EnabledItems::ALL),
        Some(PickupKind::SlowFoot)
    );
    assert_eq!(
        sample_pickup(&mut all, EnabledItems::ALL),
        Some(PickupKind::PiercingBomb)
    );
    assert_eq!(
        sample_pickup(&mut all, EnabledItems::ALL),
        Some(PickupKind::PiercingBomb)
    );
}

#[test]
fn seed_31_pins_pickup_and_two_piercing_targets_in_one_fixture() {
    let generated = generate_expanded_arena([31; 32], EnabledItems::PIERCING_BOMB);
    assert_eq!(
        generated.concealed_pickups().get(&CellId::new(3, 1)),
        Some(&PickupKind::PiercingBomb)
    );
    assert!(generated.arena().blocks().contains(&CellId::new(4, 1)));
    assert!(generated.arena().blocks().contains(&CellId::new(5, 1)));
}

#[test]
fn every_old_mask_keeps_its_exact_pool_bound_and_tail() {
    let bits = [
        16, 512, 4096, 8192, 16_384, 32_768, 65_536, 131_072, 262_144,
    ];
    for mask in 0_u32..512 {
        let raw = bits
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1 << index) != 0)
            .fold(0, |enabled, (_, bit)| enabled | *bit);
        let enabled = EnabledItems::from_bits(raw).unwrap();
        let bound = 100
            + u32::from(enabled.contains_heart()) * 25
            + u32::from(enabled.contains_shield()) * 15
            + u32::from(enabled.contains_golden_flame()) * 15
            + u32::from(enabled.contains_short_fuse()) * 15
            + u32::from(enabled.contains_short_flame()) * 15
            + u32::from(enabled.contains_long_fuse()) * 15
            + u32::from(enabled.contains_reverse_controls()) * 15
            + u32::from(enabled.contains_diarrhea()) * 15
            + u32::from(enabled.contains_slow_foot()) * 15;
        let expected = if enabled.contains_slow_foot() {
            PickupKind::SlowFoot
        } else if enabled.contains_diarrhea() {
            PickupKind::Diarrhea
        } else if enabled.contains_reverse_controls() {
            PickupKind::ReverseControls
        } else if enabled.contains_long_fuse() {
            PickupKind::LongFuse
        } else if enabled.contains_short_flame() {
            PickupKind::ShortFlame
        } else if enabled.contains_short_fuse() {
            PickupKind::ShortFuse
        } else if enabled.contains_golden_flame() {
            PickupKind::GoldenFlame
        } else if enabled.contains_shield() {
            PickupKind::Shield
        } else if enabled.contains_heart() {
            PickupKind::Heart
        } else {
            PickupKind::MovementSpeed
        };
        let mut source = Sequence([bound - 1].into());
        assert_eq!(sample_pickup(&mut source, enabled), Some(expected));
    }
}

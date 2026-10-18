use std::collections::VecDeque;

use super::*;

struct Sequence(VecDeque<u32>);

impl RandomWords for Sequence {
    fn next_u32(&mut self) -> Option<u32> {
        self.0.pop_front()
    }
}

#[test]
fn reverse_controls_weight_is_appended_after_long_fuse() {
    let mut only = Sequence([99, 100, 114].into());
    assert_eq!(
        sample_pickup(&mut only, EnabledItems::REVERSE_CONTROLS),
        Some(PickupKind::MovementSpeed)
    );
    for _ in 0..2 {
        assert_eq!(
            sample_pickup(&mut only, EnabledItems::REVERSE_CONTROLS),
            Some(PickupKind::ReverseControls)
        );
    }

    let old_all = EnabledItems::from_bits(209_424).unwrap();
    let mut boundary = Sequence([199].into());
    assert_eq!(
        sample_pickup(&mut boundary, old_all),
        Some(PickupKind::LongFuse)
    );
    let mut all = Sequence([199, 200, 214].into());
    assert_eq!(
        sample_pickup(&mut all, EnabledItems::ALL),
        Some(PickupKind::LongFuse)
    );
    assert_eq!(
        sample_pickup(&mut all, EnabledItems::ALL),
        Some(PickupKind::ReverseControls)
    );
    assert_eq!(
        sample_pickup(&mut all, EnabledItems::ALL),
        Some(PickupKind::ReverseControls)
    );
}

#[test]
fn repeating_byte_seed_31_generates_reverse_controls_at_three_one() {
    let generated = generate_expanded_arena([31; 32], EnabledItems::REVERSE_CONTROLS);
    assert_eq!(
        generated.concealed_pickups().get(&CellId::new(3, 1)),
        Some(&PickupKind::ReverseControls)
    );
}

#[test]
fn every_previous_mask_keeps_its_exact_pool_bound_and_tail() {
    let item_bits = [16, 512, 4096, 8192, 65_536, 131_072];
    for mask in 0_u32..64 {
        let bits = item_bits
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1 << index) != 0)
            .fold(0, |enabled, (_, bit)| enabled | *bit);
        let enabled = EnabledItems::from_bits(bits).unwrap();
        let bound = 100
            + u32::from(enabled.contains_heart()) * 25
            + u32::from(enabled.contains_shield()) * 15
            + u32::from(enabled.contains_golden_flame()) * 15
            + u32::from(enabled.contains_short_fuse()) * 15
            + u32::from(enabled.contains_short_flame()) * 15
            + u32::from(enabled.contains_long_fuse()) * 15;
        let expected = if enabled.contains_long_fuse() {
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

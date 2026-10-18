use std::collections::VecDeque;

use super::*;

struct Sequence(VecDeque<u32>);

impl RandomWords for Sequence {
    fn next_u32(&mut self) -> Option<u32> {
        self.0.pop_front()
    }
}

#[test]
fn short_flame_weight_is_appended_after_short_fuse() {
    let mut short = Sequence([99, 100, 114].into());
    assert_eq!(
        sample_pickup(&mut short, EnabledItems::SHORT_FLAME),
        Some(PickupKind::MovementSpeed)
    );
    for _ in 0..2 {
        assert_eq!(
            sample_pickup(&mut short, EnabledItems::SHORT_FLAME),
            Some(PickupKind::ShortFlame)
        );
    }

    let mut all = Sequence([154, 155, 169, 170, 184].into());
    for expected in [
        PickupKind::GoldenFlame,
        PickupKind::ShortFuse,
        PickupKind::ShortFuse,
        PickupKind::ShortFlame,
        PickupKind::ShortFlame,
    ] {
        assert_eq!(sample_pickup(&mut all, EnabledItems::ALL), Some(expected));
    }
}

#[test]
fn every_old_mask_retains_its_exact_sampling_boundaries() {
    for bits in [
        0, 16, 512, 528, 4096, 4112, 4608, 4624, 8192, 8208, 8704, 8720, 12288, 12304, 12800, 12816,
    ] {
        let enabled = EnabledItems::from_bits(bits).unwrap();
        let old_bound = 100
            + u32::from(enabled.contains_heart()) * 25
            + u32::from(enabled.contains_shield()) * 15
            + u32::from(enabled.contains_golden_flame()) * 15
            + u32::from(enabled.contains_short_fuse()) * 15;
        let mut source = Sequence([old_bound - 1].into());
        assert_ne!(
            sample_pickup(&mut source, enabled),
            Some(PickupKind::ShortFlame)
        );
    }

    let old_all = EnabledItems::from_bits(12_816).unwrap();
    let mut source = Sequence([139, 140, 154, 155, 169].into());
    for expected in [
        PickupKind::Shield,
        PickupKind::GoldenFlame,
        PickupKind::GoldenFlame,
        PickupKind::ShortFuse,
        PickupKind::ShortFuse,
    ] {
        assert_eq!(sample_pickup(&mut source, old_all), Some(expected));
    }
}

#[test]
fn repeating_byte_seed_31_generates_short_flame_at_three_one() {
    let generated = generate_expanded_arena([31; 32], EnabledItems::SHORT_FLAME);
    assert_eq!(
        generated.concealed_pickups().get(&CellId::new(3, 1)),
        Some(&PickupKind::ShortFlame)
    );
}

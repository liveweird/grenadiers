use std::collections::VecDeque;

use super::*;

struct Sequence(VecDeque<u32>);

impl RandomWords for Sequence {
    fn next_u32(&mut self) -> Option<u32> {
        self.0.pop_front()
    }
}

#[test]
fn remote_weight_is_appended_after_power_without_changing_the_old_tail() {
    let old_all = EnabledItems::from_bits(520_912).unwrap();
    let mut old_tail = Sequence([274].into());
    assert_eq!(
        sample_pickup(&mut old_tail, old_all),
        Some(PickupKind::PowerBomb)
    );
    let mut all = Sequence([274, 275, 289].into());
    assert_eq!(
        sample_pickup(&mut all, EnabledItems::ALL),
        Some(PickupKind::PowerBomb)
    );
    assert_eq!(
        sample_pickup(&mut all, EnabledItems::ALL),
        Some(PickupKind::RemoteBomb)
    );
    assert_eq!(
        sample_pickup(&mut all, EnabledItems::ALL),
        Some(PickupKind::RemoteBomb)
    );
}

#[test]
fn seed_31_pins_the_remote_pickup_and_native_route_geometry() {
    let generated = generate_expanded_arena([31; 32], EnabledItems::REMOTE_BOMB);
    assert_eq!(
        generated.concealed_pickups().get(&CellId::new(3, 1)),
        Some(&PickupKind::RemoteBomb)
    );
    assert!(generated.arena().blocks().contains(&CellId::new(3, 1)));
    assert_eq!(
        generated.arena().terrain(CellId::new(2, 1)),
        Some(Terrain::Floor)
    );
    assert_eq!(
        generated.arena().terrain(CellId::new(1, 2)),
        Some(Terrain::Floor)
    );
    assert!(!generated.arena().blocks().contains(&CellId::new(2, 1)));
    assert!(!generated.arena().blocks().contains(&CellId::new(1, 2)));
}

#[test]
fn every_previous_mask_keeps_its_exact_pool_tail() {
    let bits = [
        16, 64, 128, 512, 4096, 8192, 16_384, 32_768, 65_536, 131_072, 262_144,
    ];
    for mask in 0_u32..2048 {
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
            + u32::from(enabled.contains_slow_foot()) * 15
            + u32::from(enabled.contains_piercing_bomb()) * 15
            + u32::from(enabled.contains_power_bomb()) * 15;
        let expected = if enabled.contains_power_bomb() {
            PickupKind::PowerBomb
        } else if enabled.contains_piercing_bomb() {
            PickupKind::PiercingBomb
        } else if enabled.contains_slow_foot() {
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

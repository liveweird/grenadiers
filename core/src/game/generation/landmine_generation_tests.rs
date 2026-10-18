use std::collections::VecDeque;

use super::*;

struct Sequence(VecDeque<u32>);

impl RandomWords for Sequence {
    fn next_u32(&mut self) -> Option<u32> {
        self.0.pop_front()
    }
}

#[test]
fn landmine_weight_is_appended_after_remote_without_changing_the_old_tail() {
    let old_all = EnabledItems::from_bits(520_944).unwrap();
    let mut old_tail = Sequence([289].into());
    assert_eq!(
        sample_pickup(&mut old_tail, old_all),
        Some(PickupKind::RemoteBomb)
    );
    let mut all = Sequence([289, 290, 304].into());
    assert_eq!(
        sample_pickup(&mut all, EnabledItems::ALL),
        Some(PickupKind::RemoteBomb)
    );
    assert_eq!(
        sample_pickup(&mut all, EnabledItems::ALL),
        Some(PickupKind::Landmine)
    );
    assert_eq!(
        sample_pickup(&mut all, EnabledItems::ALL),
        Some(PickupKind::Landmine)
    );
}

#[test]
fn seed_31_pins_the_landmine_pickup_and_native_route_geometry() {
    let generated = generate_expanded_arena([31; 32], EnabledItems::LANDMINE);
    assert_eq!(
        generated.concealed_pickups().get(&CellId::new(3, 1)),
        Some(&PickupKind::Landmine)
    );
    assert!(generated.arena().blocks().contains(&CellId::new(3, 1)));
    for cell in [CellId::new(2, 1), CellId::new(1, 2)] {
        assert_eq!(generated.arena().terrain(cell), Some(Terrain::Floor));
        assert!(!generated.arena().blocks().contains(&cell));
    }
}

#[test]
fn every_previous_mask_keeps_its_exact_pool_tail() {
    let bits = [
        16, 32, 64, 128, 512, 4096, 8192, 16_384, 32_768, 65_536, 131_072, 262_144,
    ];
    for mask in 0_u32..4096 {
        let raw = bits
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1 << index) != 0)
            .fold(0, |enabled, (_, bit)| enabled | *bit);
        let enabled = EnabledItems::from_bits(raw).unwrap();
        let bound = 100
            + u32::from(enabled.contains_heart()) * 25
            + bits[1..].iter().filter(|bit| raw & **bit != 0).count() as u32 * 15;
        let expected = sample_pickup(&mut Sequence([bound - 1].into()), enabled).unwrap();
        assert_ne!(expected, PickupKind::Landmine);
    }
}

use std::collections::{BTreeSet, VecDeque};

use super::*;

struct Sequence(VecDeque<u32>);

impl RandomWords for Sequence {
    fn next_u32(&mut self) -> Option<u32> {
        self.0.pop_front()
    }
}

#[test]
fn invisible_bombs_weight_is_appended_after_stopwatch_without_changing_the_old_tail() {
    let previous_all = EnabledItems::from_bits(524_272).unwrap();
    assert_eq!(
        sample_pickup(&mut Sequence([334].into()), previous_all),
        Some(PickupKind::Stopwatch)
    );
    let mut all = Sequence([334, 335, 349].into());
    assert_eq!(
        sample_pickup(&mut all, EnabledItems::ALL),
        Some(PickupKind::Stopwatch)
    );
    assert_eq!(
        sample_pickup(&mut all, EnabledItems::ALL),
        Some(PickupKind::InvisibleBombs)
    );
    assert_eq!(
        sample_pickup(&mut all, EnabledItems::ALL),
        Some(PickupKind::InvisibleBombs)
    );
}

#[test]
fn seed_31_pins_both_invisible_bombs_pickups_and_escape_routes() {
    let generated = generate_expanded_arena([31; 32], EnabledItems::INVISIBLE_BOMBS);
    for cell in [CellId::new(3, 1), CellId::new(11, 11)] {
        assert_eq!(
            generated.concealed_pickups().get(&cell),
            Some(&PickupKind::InvisibleBombs)
        );
        assert!(generated.arena().blocks().contains(&cell));
    }
    for cell in [
        CellId::new(2, 1),
        CellId::new(1, 2),
        CellId::new(12, 11),
        CellId::new(13, 10),
    ] {
        assert_eq!(generated.arena().terrain(cell), Some(Terrain::Floor));
        assert!(!generated.arena().blocks().contains(&cell));
    }
}

#[test]
fn every_pre_invisible_bombs_mask_keeps_its_exact_pool_tail() {
    let bits = [
        16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16_384, 32_768, 65_536, 131_072, 262_144,
    ];
    assert_eq!(bits.len(), 15);
    let mut mappings = BTreeSet::new();
    for mask in 0_u32..32_768 {
        let raw = bits
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1 << index) != 0)
            .fold(0, |enabled, (_, bit)| enabled | *bit);
        let enabled = EnabledItems::from_bits(raw).unwrap();
        let bound = 100
            + u32::from(enabled.contains_heart()) * 25
            + bits[1..].iter().filter(|bit| raw & **bit != 0).count() as u32 * 15;
        assert_ne!(
            sample_pickup(&mut Sequence([bound - 1].into()), enabled),
            Some(PickupKind::InvisibleBombs)
        );
        mappings.insert((raw, bound));
    }
    assert_eq!(mappings.len(), 32_768);
}

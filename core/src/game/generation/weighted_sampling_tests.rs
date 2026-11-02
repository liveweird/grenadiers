use std::collections::VecDeque;

use super::*;

struct Sequence {
    words: VecDeque<u32>,
    draws: usize,
}

impl Sequence {
    fn new(words: impl IntoIterator<Item = u32>) -> Self {
        Self {
            words: words.into_iter().collect(),
            draws: 0,
        }
    }
}

impl RandomWords for Sequence {
    fn next_u32(&mut self) -> Option<u32> {
        self.draws += 1;
        self.words.pop_front()
    }
}

fn weighted(
    words: impl IntoIterator<Item = u32>,
    enabled: EnabledItems,
    multiplier: u64,
) -> (Option<PickupKind>, usize) {
    let mut source = Sequence::new(words);
    let result = sample_pickup_with_policy(
        &mut source,
        enabled,
        PickupSelectionPolicy::WeightedBombCapacityV2(
            BombCapacityDropMultiplier::new(multiplier).unwrap(),
        ),
    );
    (result, source.draws)
}

#[test]
fn exact_boundaries_cover_mandatory_and_optional_intervals() {
    let enabled = EnabledItems::HEART_AND_SHIELD;
    for (value, expected) in [
        (0, PickupKind::BombCapacity),
        (199, PickupKind::BombCapacity),
        (200, PickupKind::BlastRange),
        (239, PickupKind::BlastRange),
        (240, PickupKind::MovementSpeed),
        (259, PickupKind::MovementSpeed),
        (260, PickupKind::Heart),
        (284, PickupKind::Heart),
        (285, PickupKind::Shield),
        (299, PickupKind::Shield),
    ] {
        assert_eq!(weighted([value], enabled, 5).0, Some(expected));
    }
}

#[test]
fn multiplier_extrema_use_the_expected_normalized_pool() {
    assert_eq!(
        weighted([99], EnabledItems::EMPTY, 1).0,
        Some(PickupKind::MovementSpeed)
    );
    assert_eq!(
        weighted([399], EnabledItems::EMPTY, 10).0,
        Some(PickupKind::BombCapacity)
    );
    assert_eq!(
        weighted([400], EnabledItems::EMPTY, 10).0,
        Some(PickupKind::BlastRange)
    );
    assert_eq!(
        weighted([459], EnabledItems::EMPTY, 10).0,
        Some(PickupKind::MovementSpeed)
    );
    assert_eq!(
        weighted([799], EnabledItems::ALL, 10).0,
        Some(PickupKind::OilBarrel)
    );
}

#[test]
fn multiplier_one_uses_the_historical_percent_rejection_path() {
    let rejected = u32::MAX;
    let mut legacy = Sequence::new([rejected, 99]);
    let mut weighted_source = Sequence::new([rejected, 99]);
    let expected = sample_pickup(&mut legacy, EnabledItems::EMPTY);
    let actual = sample_pickup_with_policy(
        &mut weighted_source,
        EnabledItems::EMPTY,
        PickupSelectionPolicy::WeightedBombCapacityV2(BombCapacityDropMultiplier::LEGACY),
    );
    assert_eq!(actual, expected);
    assert_eq!(weighted_source.draws, legacy.draws);
}

#[test]
fn weighted_rejection_is_bounded_to_sixteen_draws() {
    let rejected = u32::MAX;
    let (result, draws) = weighted([rejected; 16], EnabledItems::ALL, 10);
    assert_eq!(result, None);
    assert_eq!(draws, 16);
}

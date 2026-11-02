use crate::game::{EnabledItems, PickupKind};
use crate::room::BombCapacityDropMultiplier;

use super::random_words::RandomWords;
use super::sampling::{pickup_for_value, sample_below, sample_pickup};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PickupSelectionPolicy {
    LegacyV1,
    WeightedBombCapacityV2(BombCapacityDropMultiplier),
}

pub(super) fn sample_pickup_with_policy(
    source: &mut impl RandomWords,
    enabled: EnabledItems,
    policy: PickupSelectionPolicy,
) -> Option<PickupKind> {
    let PickupSelectionPolicy::WeightedBombCapacityV2(multiplier) = policy else {
        return sample_pickup(source, enabled);
    };
    if multiplier == BombCapacityDropMultiplier::LEGACY {
        return sample_pickup(source, enabled);
    }

    let bomb_weight = 40 * u32::try_from(multiplier.value()).ok()?;
    let heart = u32::from(enabled.contains_heart());
    let bound = bomb_weight + 60 + heart * 25 + (enabled.bits().count_ones() - heart) * 15;
    let value = sample_below(source, bound)?;
    if value < bomb_weight {
        Some(PickupKind::BombCapacity)
    } else {
        Some(pickup_for_value(value - (bomb_weight - 40), enabled))
    }
}

#[cfg(test)]
#[path = "weighted_sampling_tests.rs"]
mod tests;

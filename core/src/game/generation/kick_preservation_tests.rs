use std::collections::VecDeque;

use super::super::*;

const HISTORICAL_CATALOG: [(u32, EnabledItems); 18] = [
    (1 << 3, EnabledItems::WALL_PASS),
    (1 << 4, EnabledItems::HEART),
    (1 << 5, EnabledItems::REMOTE_BOMB),
    (1 << 6, EnabledItems::PIERCING_BOMB),
    (1 << 7, EnabledItems::POWER_BOMB),
    (1 << 8, EnabledItems::LANDMINE),
    (1 << 9, EnabledItems::SHIELD),
    (1 << 10, EnabledItems::STOPWATCH),
    (1 << 11, EnabledItems::INVISIBILITY_CLOAK),
    (1 << 12, EnabledItems::GOLDEN_FLAME),
    (1 << 13, EnabledItems::SHORT_FUSE),
    (1 << 14, EnabledItems::DIARRHEA),
    (1 << 15, EnabledItems::SLOW_FOOT),
    (1 << 16, EnabledItems::SHORT_FLAME),
    (1 << 17, EnabledItems::LONG_FUSE),
    (1 << 18, EnabledItems::REVERSE_CONTROLS),
    (1 << 19, EnabledItems::INVISIBLE_BOMBS),
    (1 << 21, EnabledItems::TELEPORTER),
];

const HISTORICAL_GENERATION_ORDER: [(EnabledItems, PickupKind, u32); 18] = [
    (EnabledItems::HEART, PickupKind::Heart, 25),
    (EnabledItems::SHIELD, PickupKind::Shield, 15),
    (EnabledItems::GOLDEN_FLAME, PickupKind::GoldenFlame, 15),
    (EnabledItems::SHORT_FUSE, PickupKind::ShortFuse, 15),
    (EnabledItems::SHORT_FLAME, PickupKind::ShortFlame, 15),
    (EnabledItems::LONG_FUSE, PickupKind::LongFuse, 15),
    (
        EnabledItems::REVERSE_CONTROLS,
        PickupKind::ReverseControls,
        15,
    ),
    (EnabledItems::DIARRHEA, PickupKind::Diarrhea, 15),
    (EnabledItems::SLOW_FOOT, PickupKind::SlowFoot, 15),
    (EnabledItems::PIERCING_BOMB, PickupKind::PiercingBomb, 15),
    (EnabledItems::POWER_BOMB, PickupKind::PowerBomb, 15),
    (EnabledItems::REMOTE_BOMB, PickupKind::RemoteBomb, 15),
    (EnabledItems::LANDMINE, PickupKind::Landmine, 15),
    (
        EnabledItems::INVISIBILITY_CLOAK,
        PickupKind::InvisibilityCloak,
        15,
    ),
    (EnabledItems::STOPWATCH, PickupKind::Stopwatch, 15),
    (
        EnabledItems::INVISIBLE_BOMBS,
        PickupKind::InvisibleBombs,
        15,
    ),
    (EnabledItems::TELEPORTER, PickupKind::Teleporter, 15),
    (EnabledItems::WALL_PASS, PickupKind::WallPass, 15),
];

struct CountingSequence {
    words: VecDeque<u32>,
    draws: usize,
}

impl CountingSequence {
    fn one(value: u32) -> Self {
        Self {
            words: [value].into(),
            draws: 0,
        }
    }
}

impl RandomWords for CountingSequence {
    fn next_u32(&mut self) -> Option<u32> {
        self.draws += 1;
        self.words.pop_front()
    }
}

#[test]
fn kick_preserves_every_prior_mask_mapping_and_appends_one_interval() {
    for compact_mask in 0_u32..(1 << HISTORICAL_CATALOG.len()) {
        let historical_bits = HISTORICAL_CATALOG
            .iter()
            .enumerate()
            .filter(|(index, _)| compact_mask & (1 << index) != 0)
            .fold(0, |bits, (_, (historical_bit, item))| {
                assert_eq!(*historical_bit, item.bits());
                bits | *historical_bit
            });
        let historical = EnabledItems::from_bits(historical_bits).unwrap();
        assert!(!historical.contains_kick());

        let mut partitions = vec![
            (0, 40, PickupKind::BombCapacity),
            (40, 80, PickupKind::BlastRange),
            (80, 100, PickupKind::MovementSpeed),
        ];
        let mut historical_bound = 100;
        for (item, kind, weight) in HISTORICAL_GENERATION_ORDER {
            if historical.bits() & item.bits() != 0 {
                partitions.push((historical_bound, historical_bound + weight, kind));
                historical_bound += weight;
            }
        }
        assert_eq!(partitions.last().unwrap().1, historical_bound);

        let with_kick =
            EnabledItems::from_bits(historical_bits | EnabledItems::KICK.bits()).unwrap();
        for (start, end, expected) in partitions {
            for value in [start, end - 1] {
                let mut source = CountingSequence::one(value);
                assert_eq!(sample_pickup(&mut source, historical), Some(expected));
                assert_eq!(source.draws, 1);

                let mut source = CountingSequence::one(value);
                assert_eq!(sample_pickup(&mut source, with_kick), Some(expected));
                assert_eq!(source.draws, 1);
            }
        }
        for value in [historical_bound, historical_bound + 14] {
            let mut source = CountingSequence::one(value);
            assert_eq!(
                sample_pickup(&mut source, with_kick),
                Some(PickupKind::Kick)
            );
            assert_eq!(source.draws, 1);
        }
    }
}

#[test]
fn complete_catalog_pool_is_395_with_kick_at_380_through_394() {
    assert_eq!(HISTORICAL_CATALOG.len(), 18);
    let historical_all = HISTORICAL_CATALOG
        .iter()
        .fold(0, |bits, (historical_bit, _)| bits | *historical_bit);
    assert_eq!(historical_all, 3_145_720);
    assert_eq!(EnabledItems::ALL.bits(), 3_145_721);
    assert_eq!(EnabledItems::ALL.bits().count_ones(), 19);

    for (value, expected) in [
        (379, PickupKind::WallPass),
        (380, PickupKind::Kick),
        (394, PickupKind::Kick),
    ] {
        let mut source = CountingSequence::one(value);
        assert_eq!(
            sample_pickup(&mut source, EnabledItems::ALL),
            Some(expected)
        );
        assert_eq!(source.draws, 1);
    }
}

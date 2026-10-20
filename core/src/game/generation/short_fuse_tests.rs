use std::collections::VecDeque;

use super::*;

struct Sequence(VecDeque<u32>);

impl RandomWords for Sequence {
    fn next_u32(&mut self) -> Option<u32> {
        self.0.pop_front()
    }
}

#[test]
fn short_fuse_weight_is_appended_after_every_preexisting_item() {
    let mut short = Sequence([99, 100, 114].into());
    assert_eq!(
        sample_pickup(&mut short, EnabledItems::SHORT_FUSE),
        Some(PickupKind::MovementSpeed)
    );
    for _ in 0..2 {
        assert_eq!(
            sample_pickup(&mut short, EnabledItems::SHORT_FUSE),
            Some(PickupKind::ShortFuse)
        );
    }

    let old_all = EnabledItems::from_bits(12_816).unwrap();
    let mut all = Sequence([99, 100, 124, 125, 139, 140, 154, 155, 169].into());
    for expected in [
        PickupKind::MovementSpeed,
        PickupKind::Heart,
        PickupKind::Heart,
        PickupKind::Shield,
        PickupKind::Shield,
        PickupKind::GoldenFlame,
        PickupKind::GoldenFlame,
        PickupKind::ShortFuse,
        PickupKind::ShortFuse,
    ] {
        assert_eq!(sample_pickup(&mut all, old_all), Some(expected));
    }
}

#[test]
fn all_eight_old_masks_retain_their_item_domain() {
    for bits in [0, 16, 512, 528, 4096, 4112, 4608, 4624] {
        let enabled = EnabledItems::from_bits(bits).unwrap();
        for seed in [[0; 32], [1; 32], [37; 32]] {
            assert!(
                generate_expanded_arena(seed, enabled)
                    .concealed_pickups()
                    .values()
                    .all(|kind| *kind != PickupKind::ShortFuse)
            );
        }
    }
}

#[test]
fn seeded_output_and_repeating_byte_seed_pin_short_fuse_generation() {
    assert_eq!(
        signature(&generate_expanded_arena([0; 32], EnabledItems::SHORT_FUSE)),
        "..##.##C.#.#./.X#X#X#X#X.X./.#.#.##C#.R##/RX#XFX#X#X.X#/####.##C..#RR/RXCX.X.X.XCXR/RR#..C##.####/#X.X#X#XFX#XR/##R.#C##.#.#./.X.X#X#X#X#X./.#.#.C##.##.."
    );
    let byte = (0..=u8::MAX)
        .find(|byte| {
            generate_expanded_arena([*byte; 32], EnabledItems::SHORT_FUSE)
                .concealed_pickups()
                .get(&CellId::new(3, 1))
                == Some(&PickupKind::ShortFuse)
        })
        .expect("one-byte seed search must find the ordinary Short Fuse route");
    assert_eq!(byte, 31);
}

fn signature(generated: &GeneratedArena) -> String {
    let mut rows = Vec::new();
    for row in 1..ARENA_HEIGHT - 1 {
        let mut encoded = String::new();
        for column in 1..ARENA_WIDTH - 1 {
            let cell = CellId::new(column, row);
            encoded.push(match generated.concealed_pickups().get(&cell) {
                Some(PickupKind::BombCapacity) => 'C',
                Some(PickupKind::BlastRange) => 'R',
                Some(PickupKind::MovementSpeed) => 'S',
                Some(PickupKind::WallPass) => 'A',
                Some(PickupKind::Heart) => 'H',
                Some(PickupKind::PiercingBomb) => 'P',
                Some(PickupKind::PowerBomb) => 'W',
                Some(PickupKind::RemoteBomb) => 'M',
                Some(PickupKind::Landmine) => 'N',
                Some(PickupKind::Shield) => 'D',
                Some(PickupKind::GoldenFlame) => 'G',
                Some(PickupKind::ShortFuse) => 'F',
                Some(PickupKind::ShortFlame) => 'L',
                Some(PickupKind::LongFuse) => 'U',
                Some(PickupKind::ReverseControls) => 'V',
                Some(PickupKind::Diarrhea) => 'D',
                Some(PickupKind::SlowFoot) => 'W',
                None if generated.arena().blocks().contains(&cell) => '#',
                None if generated.arena().terrain(cell) == Some(Terrain::Wall) => 'X',
                Some(PickupKind::InvisibilityCloak) => 'C',
                Some(PickupKind::Stopwatch) => 'T',
                Some(PickupKind::InvisibleBombs) => 'I',
                Some(PickupKind::Teleporter) => 'E',
                None => '.',
            });
        }
        rows.push(encoded);
    }
    rows.join("/")
}

use std::collections::VecDeque;

use super::*;

struct Sequence(VecDeque<u32>);

impl Sequence {
    fn new(words: impl IntoIterator<Item = u32>) -> Self {
        Self(words.into_iter().collect())
    }
}

impl RandomWords for Sequence {
    fn next_u32(&mut self) -> Option<u32> {
        self.0.pop_front()
    }
}

#[test]
fn golden_flame_weight_is_appended_after_every_preexisting_item() {
    let cases = [
        (
            EnabledItems::GOLDEN_FLAME,
            vec![
                (99, PickupKind::MovementSpeed),
                (100, PickupKind::GoldenFlame),
                (114, PickupKind::GoldenFlame),
            ],
        ),
        (
            EnabledItems::HEART_AND_GOLDEN_FLAME,
            vec![
                (99, PickupKind::MovementSpeed),
                (100, PickupKind::Heart),
                (124, PickupKind::Heart),
                (125, PickupKind::GoldenFlame),
                (139, PickupKind::GoldenFlame),
            ],
        ),
        (
            EnabledItems::SHIELD_AND_GOLDEN_FLAME,
            vec![
                (99, PickupKind::MovementSpeed),
                (100, PickupKind::Shield),
                (114, PickupKind::Shield),
                (115, PickupKind::GoldenFlame),
                (129, PickupKind::GoldenFlame),
            ],
        ),
        (
            EnabledItems::from_bits(4624).unwrap(),
            vec![
                (99, PickupKind::MovementSpeed),
                (100, PickupKind::Heart),
                (124, PickupKind::Heart),
                (125, PickupKind::Shield),
                (139, PickupKind::Shield),
                (140, PickupKind::GoldenFlame),
                (154, PickupKind::GoldenFlame),
            ],
        ),
    ];

    for (enabled, boundaries) in cases {
        let mut source = Sequence::new(boundaries.iter().map(|(word, _)| *word));
        for (_, expected) in boundaries {
            assert_eq!(sample_pickup(&mut source, enabled), Some(expected));
        }
    }
}

#[test]
fn golden_enabled_generation_is_seeded_and_places_only_enabled_special_items() {
    for enabled in [
        EnabledItems::GOLDEN_FLAME,
        EnabledItems::HEART_AND_GOLDEN_FLAME,
        EnabledItems::SHIELD_AND_GOLDEN_FLAME,
        EnabledItems::from_bits(4624).unwrap(),
    ] {
        let first = generate_expanded_arena([0; 32], enabled);
        let second = generate_expanded_arena([0; 32], enabled);
        assert_eq!(first, second);
        assert!(first.concealed_pickups().values().all(|kind| match kind {
            PickupKind::Heart => enabled.contains_heart(),
            PickupKind::Shield => enabled.contains_shield(),
            PickupKind::GoldenFlame => enabled.contains_golden_flame(),
            PickupKind::ShortFuse => enabled.contains_short_fuse(),
            PickupKind::ShortFlame => enabled.contains_short_flame(),
            PickupKind::LongFuse => enabled.contains_long_fuse(),
            PickupKind::ReverseControls => enabled.contains_reverse_controls(),
            PickupKind::Diarrhea => enabled.contains_diarrhea(),
            PickupKind::SlowFoot => enabled.contains_slow_foot(),
            _ => true,
        }));
    }
}

#[test]
fn seeded_outputs_match_version_two_golden_compatible_goldens() {
    assert_eq!(
        signature(&generate_expanded_arena(
            [0; 32],
            EnabledItems::GOLDEN_FLAME,
        )),
        "..##.##C.#.#./.X#X#X#X#X.X./.#.#.##C#.R##/RX#XGX#X#X.X#/####.##C..#RR/RXCX.X.X.XCXR/RR#..C##.####/#X.X#X#XGX#XR/##R.#C##.#.#./.X.X#X#X#X#X./.#.#.C##.##.."
    );
    assert_eq!(
        signature(&generate_expanded_arena(
            [0; 32],
            EnabledItems::HEART_AND_GOLDEN_FLAME,
        )),
        "..##.##H.#.#./.X#X#X#X#X.X./.#.#.##H#.R##/CX#XSX#X#X.X#/####.##H..#RR/HXCX.X.X.XCXH/RR#..H##.####/#X.X#X#XSX#XC/##R.#H##.#.#./.X.X#X#X#X#X./.#.#.H##.##.."
    );
}

#[test]
fn repeating_byte_seed_places_golden_flame_at_the_first_eligible_cell() {
    let byte = (0..=u8::MAX)
        .find(|byte| {
            generate_expanded_arena([*byte; 32], EnabledItems::GOLDEN_FLAME)
                .concealed_pickups()
                .get(&CellId::new(3, 1))
                == Some(&PickupKind::GoldenFlame)
        })
        .expect("one-byte seed search must find the ordinary Golden Flame route");
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
                Some(PickupKind::Kick) => 'K',
                Some(PickupKind::Punch) => 'U',
                Some(PickupKind::Bracelet) => 'B',
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
                Some(PickupKind::OilBarrel) => 'O',
                None => '.',
            });
        }
        rows.push(encoded);
    }
    rows.join("/")
}

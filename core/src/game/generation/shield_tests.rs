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

#[test]
fn shield_weights_have_exact_boundaries_for_both_new_masks() {
    let mut shield = Sequence::new([99, 100, 114]);
    assert_eq!(
        sample_pickup(&mut shield, EnabledItems::SHIELD),
        Some(PickupKind::MovementSpeed)
    );
    assert_eq!(
        sample_pickup(&mut shield, EnabledItems::SHIELD),
        Some(PickupKind::Shield)
    );
    assert_eq!(
        sample_pickup(&mut shield, EnabledItems::SHIELD),
        Some(PickupKind::Shield)
    );

    let mut both = Sequence::new([99, 100, 124, 125, 139]);
    assert_eq!(
        sample_pickup(&mut both, EnabledItems::HEART_AND_SHIELD),
        Some(PickupKind::MovementSpeed)
    );
    for _ in 0..2 {
        assert_eq!(
            sample_pickup(&mut both, EnabledItems::HEART_AND_SHIELD),
            Some(PickupKind::Heart)
        );
    }
    for _ in 0..2 {
        assert_eq!(
            sample_pickup(&mut both, EnabledItems::HEART_AND_SHIELD),
            Some(PickupKind::Shield)
        );
    }
}

#[test]
fn seeded_outputs_match_version_two_shield_goldens() {
    assert_eq!(
        signature(&generate_expanded_arena([0; 32], EnabledItems::SHIELD)),
        "..##.##C.#.#./.X#X#X#X#X.X./.#.#.##C#.R##/RX#XDX#X#X.X#/####.##C..#RR/RXCX.X.X.XCXR/RR#..C##.####/#X.X#X#XDX#XR/##R.#C##.#.#./.X.X#X#X#X#X./.#.#.C##.##.."
    );
    assert_eq!(
        signature(&generate_expanded_arena(
            [0; 32],
            EnabledItems::HEART_AND_SHIELD,
        )),
        "..##.##H.#.#./.X#X#X#X#X.X./.#.#.##H#.R##/CX#XSX#X#X.X#/####.##H..#RR/HXCX.X.X.XCXH/RR#..H##.####/#X.X#X#XSX#XC/##R.#H##.#.#./.X.X#X#X#X#X./.#.#.H##.##.."
    );
}

#[test]
fn repeating_byte_seed_places_a_shield_at_the_first_eligible_cell() {
    let byte = (0..=u8::MAX)
        .find(|byte| {
            generate_expanded_arena([*byte; 32], EnabledItems::SHIELD)
                .concealed_pickups()
                .get(&CellId::new(3, 1))
                == Some(&PickupKind::Shield)
        })
        .expect("one-byte seed search must find the ordinary shield route");
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
                Some(PickupKind::Heart) => 'H',
                Some(PickupKind::PiercingBomb) => 'P',
                Some(PickupKind::PowerBomb) => 'W',
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
                None => '.',
            });
        }
        rows.push(encoded);
    }
    rows.join("/")
}

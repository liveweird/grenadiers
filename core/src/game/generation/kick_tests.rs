use std::collections::VecDeque;

use super::super::*;

struct Sequence(VecDeque<u32>);

impl RandomWords for Sequence {
    fn next_u32(&mut self) -> Option<u32> {
        self.0.pop_front()
    }
}

#[test]
fn kick_has_the_exact_sole_item_pool_interval() {
    let mut source = Sequence([99, 100, 114].into());
    assert_eq!(
        sample_pickup(&mut source, EnabledItems::KICK),
        Some(PickupKind::MovementSpeed)
    );
    assert_eq!(
        sample_pickup(&mut source, EnabledItems::KICK),
        Some(PickupKind::Kick)
    );
    assert_eq!(
        sample_pickup(&mut source, EnabledItems::KICK),
        Some(PickupKind::Kick)
    );
}

#[test]
fn kick_appends_after_wall_pass_in_the_all_item_pool() {
    let mut source = Sequence([379, 380, 394].into());
    assert_eq!(
        sample_pickup(&mut source, EnabledItems::ALL),
        Some(PickupKind::WallPass)
    );
    assert_eq!(
        sample_pickup(&mut source, EnabledItems::ALL),
        Some(PickupKind::Kick)
    );
    assert_eq!(
        sample_pickup(&mut source, EnabledItems::ALL),
        Some(PickupKind::Kick)
    );
}

#[test]
fn kick_mixed_item_intervals_do_not_require_wall_pass() {
    let cases = [
        (
            EnabledItems::from_bits(EnabledItems::KICK.bits() | EnabledItems::HEART.bits())
                .unwrap(),
            [
                (99, PickupKind::MovementSpeed),
                (100, PickupKind::Heart),
                (124, PickupKind::Heart),
                (125, PickupKind::Kick),
                (139, PickupKind::Kick),
            ],
        ),
        (
            EnabledItems::from_bits(EnabledItems::KICK.bits() | EnabledItems::TELEPORTER.bits())
                .unwrap(),
            [
                (99, PickupKind::MovementSpeed),
                (100, PickupKind::Teleporter),
                (114, PickupKind::Teleporter),
                (115, PickupKind::Kick),
                (129, PickupKind::Kick),
            ],
        ),
    ];

    for (enabled, boundaries) in cases {
        for (value, expected) in boundaries {
            let mut source = Sequence([value].into());
            assert_eq!(sample_pickup(&mut source, enabled), Some(expected));
        }
    }
}

#[test]
fn seed_31_places_the_sole_kick_pool_on_both_sides() {
    let generated = generate_expanded_arena([31; 32], EnabledItems::KICK);
    for cell in [CellId::new(3, 1), CellId::new(11, 11)] {
        assert_eq!(
            generated.concealed_pickups().get(&cell),
            Some(&PickupKind::Kick)
        );
    }
}

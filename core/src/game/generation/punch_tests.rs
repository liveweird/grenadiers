use std::collections::VecDeque;

use super::super::*;

struct Sequence(VecDeque<u32>);

impl RandomWords for Sequence {
    fn next_u32(&mut self) -> Option<u32> {
        self.0.pop_front()
    }
}

#[test]
fn punch_has_the_exact_sole_item_pool_interval() {
    let mut source = Sequence([99, 100, 114].into());
    assert_eq!(
        sample_pickup(&mut source, EnabledItems::PUNCH),
        Some(PickupKind::MovementSpeed)
    );
    assert_eq!(
        sample_pickup(&mut source, EnabledItems::PUNCH),
        Some(PickupKind::Punch)
    );
    assert_eq!(
        sample_pickup(&mut source, EnabledItems::PUNCH),
        Some(PickupKind::Punch)
    );
}

#[test]
fn punch_appends_after_kick_in_the_all_item_pool() {
    let mut source = Sequence([394, 395, 409].into());
    assert_eq!(
        sample_pickup(&mut source, EnabledItems::ALL),
        Some(PickupKind::Kick)
    );
    assert_eq!(
        sample_pickup(&mut source, EnabledItems::ALL),
        Some(PickupKind::Punch)
    );
    assert_eq!(
        sample_pickup(&mut source, EnabledItems::ALL),
        Some(PickupKind::Punch)
    );
}

#[test]
fn seed_31_places_the_sole_punch_pool_on_both_sides() {
    let generated = generate_expanded_arena([31; 32], EnabledItems::PUNCH);
    for cell in [CellId::new(3, 1), CellId::new(11, 11)] {
        assert_eq!(
            generated.concealed_pickups().get(&cell),
            Some(&PickupKind::Punch)
        );
    }
}

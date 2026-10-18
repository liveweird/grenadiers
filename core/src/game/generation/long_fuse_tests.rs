use super::*;
use std::collections::VecDeque;
struct Sequence(VecDeque<u32>);
impl RandomWords for Sequence {
    fn next_u32(&mut self) -> Option<u32> {
        self.0.pop_front()
    }
}

#[test]
fn long_fuse_weight_is_appended_and_old_all_boundary_is_unchanged() {
    let mut only = Sequence([99, 100, 114].into());
    assert_eq!(
        sample_pickup(&mut only, EnabledItems::LONG_FUSE),
        Some(PickupKind::MovementSpeed)
    );
    assert_eq!(
        sample_pickup(&mut only, EnabledItems::LONG_FUSE),
        Some(PickupKind::LongFuse)
    );
    assert_eq!(
        sample_pickup(&mut only, EnabledItems::LONG_FUSE),
        Some(PickupKind::LongFuse)
    );
    let old = EnabledItems::from_bits(78_352).unwrap();
    let mut boundary = Sequence([184].into());
    assert_eq!(
        sample_pickup(&mut boundary, old),
        Some(PickupKind::ShortFlame)
    );
}

#[test]
fn repeating_byte_seed_31_generates_long_fuse_at_three_one() {
    let generated = generate_expanded_arena([31; 32], EnabledItems::LONG_FUSE);
    assert_eq!(
        generated.concealed_pickups().get(&CellId::new(3, 1)),
        Some(&PickupKind::LongFuse)
    );
}

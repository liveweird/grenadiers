use std::num::NonZeroU32;

use super::RoundAssignment;
use crate::game::{CellId, SeatId, fixed_arena};

#[test]
fn odd_and_even_rounds_swap_spawns_and_collection_priority() {
    let arena = fixed_arena();
    let odd = RoundAssignment::new(NonZeroU32::new(3).unwrap());
    let even = RoundAssignment::new(NonZeroU32::new(4).unwrap());

    assert_eq!(odd.priority_seat(), SeatId::One);
    assert_eq!(odd.spawn_for(&arena, SeatId::One), CellId::new(1, 1));
    assert_eq!(odd.spawn_for(&arena, SeatId::Two), CellId::new(13, 11));
    assert_eq!(even.priority_seat(), SeatId::Two);
    assert_eq!(even.spawn_for(&arena, SeatId::One), CellId::new(13, 11));
    assert_eq!(even.spawn_for(&arena, SeatId::Two), CellId::new(1, 1));
}

#[test]
fn mapping_uses_cell_order_even_when_arena_spawn_metadata_is_reversed() {
    let original = fixed_arena();
    let arena = crate::game::Arena::new(
        std::array::from_fn(|index| {
            let column = (index % usize::from(crate::game::arena::ARENA_WIDTH)) as u16;
            let row = (index / usize::from(crate::game::arena::ARENA_WIDTH)) as u16;
            original.terrain(CellId::new(column, row)).unwrap()
        }),
        original.blocks().clone(),
        [CellId::new(13, 11), CellId::new(1, 1)],
    )
    .unwrap();

    assert_eq!(RoundAssignment::first().spawns(&arena), original.spawns());
}

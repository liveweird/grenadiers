use std::num::NonZeroU32;

use super::RoundAssignment;
use crate::game::{CellId, PlayerCount, SeatId, fixed_arena};

#[test]
fn odd_and_even_rounds_swap_spawns_and_collection_priority() {
    let arena = fixed_arena();
    let odd = RoundAssignment::new(NonZeroU32::new(3).unwrap());
    let even = RoundAssignment::new(NonZeroU32::new(4).unwrap());

    assert_eq!(odd.priority_seat(), SeatId::One);
    assert_eq!(odd.spawn_for(&arena, SeatId::One), Some(CellId::new(1, 1)));
    assert_eq!(
        odd.spawn_for(&arena, SeatId::Two),
        Some(CellId::new(13, 11))
    );
    assert_eq!(even.priority_seat(), SeatId::Two);
    assert_eq!(
        even.spawn_for(&arena, SeatId::One),
        Some(CellId::new(13, 11))
    );
    assert_eq!(even.spawn_for(&arena, SeatId::Two), Some(CellId::new(1, 1)));
    assert_eq!(odd.spawn_for(&arena, SeatId::Three), None);
    assert_eq!(odd.spawn_for(&arena, SeatId::Four), None);
}

#[test]
fn multiplayer_priority_and_spawn_rotation_cover_every_seat() {
    let original = fixed_arena();
    let corners = [
        CellId::new(1, 1),
        CellId::new(13, 1),
        CellId::new(1, 11),
        CellId::new(13, 11),
    ];
    let arena = crate::game::Arena::new(
        std::array::from_fn(|index| {
            let column = (index % usize::from(crate::game::arena::ARENA_WIDTH)) as u16;
            let row = (index / usize::from(crate::game::arena::ARENA_WIDTH)) as u16;
            original.terrain(CellId::new(column, row)).unwrap()
        }),
        std::collections::BTreeSet::new(),
        corners,
    )
    .unwrap();
    let round = RoundAssignment::new(NonZeroU32::new(4).unwrap());

    assert_eq!(round.priority_seat_for(PlayerCount::Four), SeatId::Four);
    assert_eq!(
        round.priority_order(PlayerCount::Four).collect::<Vec<_>>(),
        [SeatId::Four, SeatId::One, SeatId::Two, SeatId::Three]
    );
    assert_eq!(
        round.spawns(&arena),
        [corners[3], corners[0], corners[1], corners[2]]
    );
    for &seat in PlayerCount::Four.seats() {
        assert_eq!(
            round.spawn_for(&arena, seat),
            Some(round.spawns(&arena)[seat.index()])
        );
    }

    let three = crate::game::Arena::new(
        std::array::from_fn(|index| {
            let column = (index % usize::from(crate::game::arena::ARENA_WIDTH)) as u16;
            let row = (index / usize::from(crate::game::arena::ARENA_WIDTH)) as u16;
            original.terrain(CellId::new(column, row)).unwrap()
        }),
        std::collections::BTreeSet::new(),
        corners[..3].to_vec(),
    )
    .unwrap();
    for &seat in PlayerCount::Three.seats() {
        assert!(round.spawn_for(&three, seat).is_some());
    }
    assert_eq!(round.spawn_for(&three, SeatId::Four), None);
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

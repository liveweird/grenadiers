use super::{CellId, PlayerCount, Position, SeatId, SpeedLevel, UNITS_PER_CELL};

#[test]
fn cell_ids_sort_in_row_major_order() {
    let mut cells = [
        CellId::new(0, 2),
        CellId::new(4, 0),
        CellId::new(1, 1),
        CellId::new(2, 1),
    ];

    cells.sort();

    assert_eq!(
        cells,
        [
            CellId::new(4, 0),
            CellId::new(1, 1),
            CellId::new(2, 1),
            CellId::new(0, 2),
        ]
    );
}

#[test]
fn seats_have_stable_indexes() {
    assert_eq!(SeatId::DUEL, [SeatId::One, SeatId::Two]);
    assert_eq!(
        SeatId::ALL,
        [SeatId::One, SeatId::Two, SeatId::Three, SeatId::Four]
    );
    assert_eq!(SeatId::One.index(), 0);
    assert_eq!(SeatId::Two.index(), 1);
    assert_eq!(SeatId::Three.index(), 2);
    assert_eq!(SeatId::Four.index(), 3);
    assert_eq!(PlayerCount::try_from(3), Ok(PlayerCount::Three));
    assert_eq!(PlayerCount::try_from(1), Err(()));
    assert_eq!(PlayerCount::Four.seats(), SeatId::ALL);
}

#[test]
fn centers_round_trip_to_cells() {
    let cell = CellId::new(13, 11);
    let position = Position::center(cell);

    assert_eq!(position.x, 13 * UNITS_PER_CELL + UNITS_PER_CELL / 2);
    assert_eq!(position.y, 11 * UNITS_PER_CELL + UNITS_PER_CELL / 2);
    assert_eq!(position.cell(), Some(cell));
}

#[test]
fn invalid_positions_do_not_silently_wrap_to_cell_ids() {
    assert_eq!(Position { x: -1, y: 0 }.cell(), None);
    assert_eq!(Position { x: 0, y: -1 }.cell(), None);
    assert_eq!(
        Position {
            x: i32::MAX,
            y: i32::MAX,
        }
        .cell(),
        None
    );
}

#[test]
fn speed_levels_use_exact_fixed_point_values() {
    assert_eq!(SpeedLevel::Base.units_per_second(), 3_072);
    assert_eq!(SpeedLevel::One.units_per_second(), 3_584);
    assert_eq!(SpeedLevel::Two.units_per_second(), 4_096);
    assert_eq!(SpeedLevel::Three.units_per_second(), 4_608);
    assert_eq!(SpeedLevel::Four.units_per_second(), 5_120);
    assert_eq!(SpeedLevel::Base.next(), SpeedLevel::One);
    assert_eq!(SpeedLevel::One.next(), SpeedLevel::Two);
    assert_eq!(SpeedLevel::Two.next(), SpeedLevel::Three);
    assert_eq!(SpeedLevel::Three.next(), SpeedLevel::Four);
    assert_eq!(SpeedLevel::Four.next(), SpeedLevel::Four);
}

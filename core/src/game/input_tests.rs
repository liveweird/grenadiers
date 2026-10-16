use super::{Direction, HeldDirections};

#[test]
fn defaults_to_no_active_direction() {
    assert_eq!(HeldDirections::default().active(), None);
}

#[test]
fn newest_opposite_or_perpendicular_direction_wins() {
    let mut held = HeldDirections::default();

    held.press(Direction::North);
    held.press(Direction::South);
    assert_eq!(held.active(), Some(Direction::South));

    held.press(Direction::East);
    assert_eq!(held.active(), Some(Direction::East));
}

#[test]
fn duplicate_press_does_not_reprioritize_an_older_direction() {
    let mut held = HeldDirections::default();
    held.press(Direction::North);
    held.press(Direction::East);

    held.press(Direction::North);

    assert_eq!(held.active(), Some(Direction::East));
    held.release(Direction::East);
    assert_eq!(held.active(), Some(Direction::North));
}

#[test]
fn releasing_active_direction_resumes_previous_held_direction() {
    let mut held = HeldDirections::default();
    held.press(Direction::West);
    held.press(Direction::North);
    held.press(Direction::East);

    held.release(Direction::East);
    assert_eq!(held.active(), Some(Direction::North));
    held.release(Direction::North);
    assert_eq!(held.active(), Some(Direction::West));
}

#[test]
fn releasing_inactive_or_unknown_direction_preserves_priority() {
    let mut held = HeldDirections::default();
    held.press(Direction::North);
    held.press(Direction::East);

    held.release(Direction::North);
    assert_eq!(held.active(), Some(Direction::East));

    held.release(Direction::South);
    assert_eq!(held.active(), Some(Direction::East));
}

#[test]
fn all_four_directions_resume_in_reverse_press_order() {
    let mut held = HeldDirections::default();
    held.press(Direction::North);
    held.press(Direction::East);
    held.press(Direction::South);
    held.press(Direction::West);

    assert_eq!(held.active(), Some(Direction::West));
    held.release(Direction::West);
    assert_eq!(held.active(), Some(Direction::South));
    held.release(Direction::South);
    assert_eq!(held.active(), Some(Direction::East));
    held.release(Direction::East);
    assert_eq!(held.active(), Some(Direction::North));
    held.release(Direction::North);
    assert_eq!(held.active(), None);
}

#[test]
fn clear_stops_movement_and_accepts_fresh_input() {
    let mut held = HeldDirections::default();
    held.press(Direction::North);
    held.press(Direction::East);

    held.clear();
    assert_eq!(held.active(), None);

    held.release(Direction::East);
    held.press(Direction::South);
    assert_eq!(held.active(), Some(Direction::South));
}

#[test]
fn equivalent_event_histories_produce_equal_state() {
    let mut direct = HeldDirections::default();
    direct.press(Direction::North);
    direct.press(Direction::East);

    let mut with_idempotent_events = HeldDirections::default();
    with_idempotent_events.press(Direction::North);
    with_idempotent_events.press(Direction::North);
    with_idempotent_events.press(Direction::West);
    with_idempotent_events.release(Direction::West);
    with_idempotent_events.release(Direction::South);
    with_idempotent_events.press(Direction::East);

    assert_eq!(direct, with_idempotent_events);
}

#[test]
fn same_active_direction_can_retain_distinct_fallback_state() {
    let mut north_then_east = HeldDirections::default();
    north_then_east.press(Direction::North);
    north_then_east.press(Direction::East);

    let mut south_then_east = HeldDirections::default();
    south_then_east.press(Direction::South);
    south_then_east.press(Direction::East);

    assert_eq!(north_then_east.active(), south_then_east.active());
    assert_ne!(north_then_east, south_then_east);

    north_then_east.release(Direction::East);
    south_then_east.release(Direction::East);
    assert_eq!(north_then_east.active(), Some(Direction::North));
    assert_eq!(south_then_east.active(), Some(Direction::South));
}

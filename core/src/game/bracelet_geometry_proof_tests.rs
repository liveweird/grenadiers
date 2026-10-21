use std::collections::BTreeSet;

use super::movement::MovementOutcome;
use super::{Direction, MovementTrace, Position, SeatId};
use crate::game::{
    BombTrigger, CellId, EffectKind, EnabledItems, EntityId, Equipment, MovementSegment, Tick,
};

use super::bracelet_test_support::{press, simple_state, state};

fn outcome(
    start: Position,
    budget: i32,
    legs: &[(Direction, i32)],
    forward_leg: Option<usize>,
) -> MovementOutcome {
    let mut trace = MovementTrace::new(start);
    let mut cursor = start;
    let mut forward_segment = None;
    for (index, &(direction, distance)) in legs.iter().enumerate() {
        let end = match direction {
            Direction::East => Position {
                x: cursor.x + distance,
                y: cursor.y,
            },
            Direction::West => Position {
                x: cursor.x - distance,
                y: cursor.y,
            },
            Direction::South => Position {
                x: cursor.x,
                y: cursor.y + distance,
            },
            Direction::North => Position {
                x: cursor.x,
                y: cursor.y - distance,
            },
        };
        let segment = MovementSegment::new(cursor, end);
        trace.push(segment);
        if forward_leg == Some(index) {
            forward_segment = Some(segment);
        }
        cursor = end;
    }
    MovementOutcome {
        trace,
        kick: None,
        original_budget: budget,
        forward_segment,
    }
}

fn entry(holder: MovementOutcome, rival: MovementOutcome) -> Option<Direction> {
    super::shove_geometry::contact_entry_direction(SeatId::One, &[holder, rival])
}

#[test]
fn tangent_motion_needs_positive_area_but_one_unit_of_entry_qualifies() {
    let holder = Position { x: 4_000, y: 4_000 };
    let tangent = Position { x: 4_691, y: 4_640 };
    assert_eq!(
        entry(
            outcome(holder, 51, &[(Direction::East, 51)], Some(0)),
            outcome(tangent, 0, &[], None),
        ),
        None
    );

    let one_unit_inside = Position { x: 4_690, y: 4_639 };
    assert_eq!(
        entry(
            outcome(holder, 51, &[(Direction::East, 51)], Some(0)),
            outcome(one_unit_inside, 0, &[], None),
        ),
        Some(Direction::East)
    );
}

#[test]
fn exact_time_geometry_handles_different_budgets_parallel_and_opposing_motion() {
    let holder = Position { x: 4_000, y: 4_000 };
    let rival = Position { x: 4_641, y: 4_000 };
    assert_eq!(
        entry(
            outcome(holder, 86, &[(Direction::East, 86)], Some(0)),
            outcome(rival, 51, &[(Direction::East, 51)], None),
        ),
        Some(Direction::East),
        "the faster parallel path closes the one-unit gap"
    );
    assert_eq!(
        entry(
            outcome(holder, 86, &[(Direction::East, 86)], Some(0)),
            outcome(rival, 51, &[(Direction::West, 51)], None),
        ),
        Some(Direction::East),
        "opposing paths close the same leading face sooner"
    );
}

#[test]
fn correction_then_forward_and_blocked_stationary_remainder_use_actual_timing() {
    let corrected = outcome(
        Position { x: 4_000, y: 4_020 },
        51,
        &[(Direction::North, 20), (Direction::East, 31)],
        Some(1),
    );
    assert_eq!(
        entry(
            corrected,
            outcome(Position { x: 4_650, y: 4_000 }, 0, &[], None)
        ),
        Some(Direction::East)
    );

    let short_forward = outcome(
        Position { x: 4_000, y: 4_000 },
        51,
        &[(Direction::East, 10)],
        Some(0),
    );
    assert_eq!(
        entry(
            short_forward,
            outcome(Position { x: 4_649, y: 4_000 }, 0, &[], None)
        ),
        Some(Direction::East),
        "entry occurs at the forward endpoint before the blocked stationary remainder"
    );
}

#[test]
fn initial_overlap_must_actually_separate_before_reentry() {
    let holder = outcome(
        Position { x: 4_000, y: 4_000 },
        86,
        &[(Direction::North, 51), (Direction::East, 35)],
        Some(1),
    );
    let rival = outcome(
        Position { x: 4_600, y: 4_600 },
        86,
        &[(Direction::East, 51), (Direction::North, 35)],
        None,
    );
    assert_eq!(entry(holder, rival), Some(Direction::East));

    let never_separates = outcome(
        Position { x: 4_000, y: 4_000 },
        86,
        &[(Direction::North, 20), (Direction::East, 66)],
        Some(1),
    );
    assert_eq!(
        entry(
            never_separates,
            outcome(Position { x: 4_600, y: 4_600 }, 0, &[], None)
        ),
        None
    );
}

#[test]
fn priority_winner_blocked_at_zero_does_not_fall_back_to_viable_loser() {
    let mut game = simple_state(
        EnabledItems::BRACELET,
        Position { x: 4_211, y: 4_096 },
        Position { x: 4_851, y: 4_096 },
    );
    game.replace_actor_equipment(SeatId::Two, Equipment::Bracelet);
    game.insert_test_bomb(
        EntityId(30),
        CellId::new(5, 4),
        EntityId(2),
        1,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
    )
    .unwrap();
    game.actors[1].bomb_grace.insert(EntityId(30));

    let report = game
        .step(&[
            press(SeatId::Two, Direction::West),
            press(SeatId::One, Direction::East),
        ])
        .unwrap();

    assert!(report.forced_movement.is_none());
    assert_eq!(game.actors[0].position, Position { x: 4_262, y: 4_096 });
    assert_eq!(game.actors[1].position, Position { x: 4_800, y: 4_096 });
    assert_eq!(game.actors[0].last_successful_shove_tick(), None);
    assert_eq!(game.actors[1].last_successful_shove_tick(), None);
}

fn bracelet_and(item: EnabledItems) -> EnabledItems {
    EnabledItems::from_bits(EnabledItems::BRACELET.bits() | item.bits()).unwrap()
}

#[test]
fn victim_wall_pass_not_initiator_grace_controls_soft_wall_clamp() {
    let block = CellId::new(6, 4);
    let world =
        crate::game::World::new(super::bracelet_test_support::arena(BTreeSet::from([block])));
    let one = Position { x: 4_220, y: 4_608 };
    let two = Position { x: 4_860, y: 4_608 };

    let mut victim_capable = state(
        bracelet_and(EnabledItems::WALL_PASS),
        world.clone(),
        one,
        two,
        Tick(0),
    );
    victim_capable.replace_actor_equipment(SeatId::Two, Equipment::WallPass);
    let full = victim_capable
        .step(&[press(SeatId::One, Direction::East)])
        .unwrap();
    assert_eq!(full.forced_movement.unwrap().segment.end().x, two.x + 1_024);

    let mut initiator_grace = state(
        bracelet_and(EnabledItems::WALL_PASS),
        world,
        one,
        two,
        Tick(0),
    );
    initiator_grace.actors[0].soft_wall_grace.insert(block);
    let clamped = initiator_grace
        .step(&[press(SeatId::One, Direction::East)])
        .unwrap();
    let end = clamped.forced_movement.unwrap().segment.end().x;
    assert!(end > two.x && end < two.x + 1_024);
}

#[test]
fn victim_bomb_grace_not_initiator_grace_controls_bomb_clamp() {
    let make = || {
        let mut game = simple_state(
            EnabledItems::BRACELET,
            Position { x: 4_220, y: 4_608 },
            Position { x: 4_860, y: 4_608 },
        );
        game.insert_test_bomb(
            EntityId(30),
            CellId::new(6, 4),
            EntityId(2),
            1,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(100),
            },
        )
        .unwrap();
        game
    };
    let mut victim_grace = make();
    victim_grace.actors[1].bomb_grace.insert(EntityId(30));
    assert_eq!(
        victim_grace
            .step(&[press(SeatId::One, Direction::East)])
            .unwrap()
            .forced_movement
            .unwrap()
            .segment
            .end()
            .x,
        5_884
    );

    let mut initiator_grace = make();
    initiator_grace.actors[0].bomb_grace.insert(EntityId(30));
    let end = initiator_grace
        .step(&[press(SeatId::One, Direction::East)])
        .unwrap()
        .forced_movement
        .unwrap()
        .segment
        .end()
        .x;
    assert!(end > 4_860 && end < 5_884);
}

#[test]
fn initial_pusher_overlap_allows_departure_and_frozen_roles_remain_distinct() {
    let one = Position { x: 4_096, y: 4_096 };
    let two = Position { x: 4_736, y: 4_096 };
    let mut departure = simple_state(EnabledItems::BRACELET, one, two);
    let forced = departure
        .step(&[press(SeatId::One, Direction::East)])
        .unwrap()
        .forced_movement
        .unwrap();
    assert_eq!(forced.segment.end().x, two.x + 1_024);

    let enabled = bracelet_and(EnabledItems::STOPWATCH);
    let mut frozen_victim = simple_state(enabled, one, two);
    frozen_victim.apply_test_effect(SeatId::One, EffectKind::Stopwatch, Tick(0));
    assert!(
        frozen_victim
            .step(&[press(SeatId::One, Direction::East)])
            .unwrap()
            .forced_movement
            .is_some()
    );

    let mut frozen_holder = simple_state(enabled, one, two);
    frozen_holder.apply_test_effect(SeatId::Two, EffectKind::Stopwatch, Tick(0));
    assert!(
        frozen_holder
            .step(&[press(SeatId::One, Direction::East)])
            .unwrap()
            .forced_movement
            .is_none()
    );
}

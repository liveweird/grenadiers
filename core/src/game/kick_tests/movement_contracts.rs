use super::*;
use crate::game::EffectKind;
use crate::game::types::{HALF_FOOTPRINT, UNITS_PER_CELL};

fn tangent(cell: CellId, direction: Direction) -> Position {
    let center = Position::center(cell);
    match direction {
        Direction::North => Position {
            y: (i32::from(cell.row) + 1) * UNITS_PER_CELL + HALF_FOOTPRINT,
            ..center
        },
        Direction::East => Position {
            x: i32::from(cell.column) * UNITS_PER_CELL - HALF_FOOTPRINT,
            ..center
        },
        Direction::South => Position {
            y: i32::from(cell.row) * UNITS_PER_CELL - HALF_FOOTPRINT,
            ..center
        },
        Direction::West => Position {
            x: (i32::from(cell.column) + 1) * UNITS_PER_CELL + HALF_FOOTPRINT,
            ..center
        },
    }
}

fn add_stationary(state: &mut State, id: u32, cell: CellId) {
    add_test_bomb_with_trigger(
        state,
        EntityId(id),
        cell,
        EntityId(1),
        2,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
    )
    .unwrap();
}

fn press(direction: Direction) -> SeatInput {
    SeatInput {
        seat: SeatId::One,
        action: InputAction::Press(direction),
    }
}

#[test]
fn every_forward_direction_and_reversed_input_arm_the_effective_direction() {
    let cases = [
        (Direction::North, Direction::North, false),
        (Direction::East, Direction::East, false),
        (Direction::South, Direction::South, false),
        (Direction::West, Direction::West, false),
        (Direction::North, Direction::South, true),
    ];
    for (effective, raw, reversed) in cases {
        let cell = CellId::new(6, 6);
        let mut state = game(tangent(cell, effective), BTreeSet::new());
        state.actors[0].heading = effective;
        if reversed {
            state.actors[0]
                .effects
                .apply(EffectKind::ReverseControls, Tick(0))
                .unwrap();
        }
        add_stationary(&mut state, 9, cell);
        state.replace_actor_equipment(SeatId::One, Equipment::Kick);

        state.step(&[press(raw)]).unwrap();

        assert_eq!(
            state.world.bombs[&EntityId(9)]
                .motion()
                .unwrap()
                .direction(),
            effective
        );
    }
}

#[test]
fn lane_correction_does_not_kick_sideways_then_forward_remainder_can_kick() {
    let cell = CellId::new(5, 5);
    let mut position = tangent(cell, Direction::East);
    position.y += 100;
    let mut state = game(position, BTreeSet::new());
    state.actors[0].heading = Direction::North;
    add_stationary(&mut state, 9, cell);
    state.replace_actor_equipment(SeatId::One, Equipment::Kick);

    state.step(&[press(Direction::East)]).unwrap();
    assert_eq!(state.world.bombs[&EntityId(9)].motion(), None);
    assert!(state.actors[0].pending_turn.is_some());

    state.step(&[]).unwrap();
    assert_eq!(
        state.world.bombs[&EntityId(9)]
            .motion()
            .unwrap()
            .direction(),
        Direction::East
    );
}

#[test]
fn zero_budget_from_slow_foot_and_stopwatch_freeze_cannot_request_kicks() {
    let cell = CellId::new(5, 5);
    let mut slow = game(tangent(cell, Direction::East), BTreeSet::new());
    add_stationary(&mut slow, 9, cell);
    slow.replace_actor_equipment(SeatId::One, Equipment::Kick);
    for _ in 0..6 {
        slow.actors[0]
            .effects
            .apply(EffectKind::SlowFoot, Tick(0))
            .unwrap();
    }
    slow.step(&[press(Direction::East)]).unwrap();
    assert_eq!(slow.world.bombs[&EntityId(9)].motion(), None);

    let mut frozen = game(tangent(cell, Direction::East), BTreeSet::new());
    add_stationary(&mut frozen, 9, cell);
    frozen.replace_actor_equipment(SeatId::One, Equipment::Kick);
    frozen.actors[1]
        .effects
        .apply(EffectKind::Stopwatch, Tick(0))
        .unwrap();
    let before = frozen.actors[0].position;
    frozen.step(&[press(Direction::East)]).unwrap();
    assert_eq!(frozen.actors[0].position, before);
    assert_eq!(frozen.world.bombs[&EntityId(9)].motion(), None);
}

#[test]
fn leaving_an_owned_grace_bomb_never_creates_a_kick_request() {
    let cell = CellId::new(5, 5);
    let mut state = game(Position::center(cell), BTreeSet::new());
    add_stationary(&mut state, 9, cell);
    state.actors[0].bomb_grace.insert(EntityId(9));
    state.replace_actor_equipment(SeatId::One, Equipment::Kick);

    state.step(&[press(Direction::East)]).unwrap();
    for _ in 0..30 {
        state.step(&[]).unwrap();
    }

    assert!(!state.actors[0].bomb_grace.contains(&EntityId(9)));
    assert!(!state.world.bombs[&EntityId(9)].has_been_kicked());
}

#[test]
fn unique_nearest_physical_bomb_wins_while_equal_bombs_or_nonbombs_reject() {
    let near = CellId::new(5, 5);
    let mut nearest = game(tangent(near, Direction::East), BTreeSet::new());
    add_stationary(&mut nearest, 9, near);
    add_stationary(&mut nearest, 10, CellId::new(7, 5));
    nearest.replace_actor_equipment(SeatId::One, Equipment::Kick);
    nearest.step(&[press(Direction::East)]).unwrap();
    assert!(nearest.world.bombs[&EntityId(9)].motion().is_some());
    assert_eq!(nearest.world.bombs[&EntityId(10)].motion(), None);

    let split = Position {
        y: 5 * UNITS_PER_CELL,
        ..tangent(near, Direction::East)
    };
    let mut equal_bombs = game(split, BTreeSet::new());
    add_stationary(&mut equal_bombs, 9, CellId::new(5, 4));
    add_stationary(&mut equal_bombs, 10, CellId::new(5, 5));
    equal_bombs.replace_actor_equipment(SeatId::One, Equipment::Kick);
    equal_bombs.step(&[press(Direction::East)]).unwrap();
    assert!(
        equal_bombs
            .world
            .bombs
            .values()
            .all(|bomb| bomb.motion().is_none())
    );

    let mut equal_nonbomb = game(split, BTreeSet::from([CellId::new(5, 5)]));
    add_stationary(&mut equal_nonbomb, 9, CellId::new(5, 4));
    equal_nonbomb.replace_actor_equipment(SeatId::One, Equipment::Kick);
    equal_nonbomb.step(&[press(Direction::East)]).unwrap();
    assert_eq!(equal_nonbomb.world.bombs[&EntityId(9)].motion(), None);
}

#[test]
fn contact_is_physically_walked_into_and_detonates_without_kick_history() {
    let cell = CellId::new(5, 5);
    let start = tangent(cell, Direction::East);
    let mut state = game(start, BTreeSet::new());
    add_test_bomb_with_trigger(
        &mut state,
        EntityId(9),
        cell,
        EntityId(1),
        2,
        Tick(0),
        BombTrigger::Contact,
    )
    .unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::Kick);

    let report = state.step(&[press(Direction::East)]).unwrap();

    assert!(state.actors[0].position.x > start.x);
    assert!(report.events.iter().any(|event| matches!(
        event.kind,
        crate::game::StepEventKind::BombDetonated(bomb)
            if bomb.id == EntityId(9) && !event.bomb_has_been_kicked
    )));
}

#[test]
fn ending_exactly_tangent_does_not_arm_until_a_positive_penetration_attempt() {
    let cell = CellId::new(5, 5);
    let exact = tangent(cell, Direction::East);
    let mut start = exact;
    start.x -= 51;
    let mut state = game(start, BTreeSet::new());
    add_stationary(&mut state, 9, cell);
    state.replace_actor_equipment(SeatId::One, Equipment::Kick);

    state.step(&[press(Direction::East)]).unwrap();
    assert_eq!(state.actors[0].position, exact);
    assert_eq!(state.world.bombs[&EntityId(9)].motion(), None);

    state.step(&[]).unwrap();
    assert!(state.world.bombs[&EntityId(9)].motion().is_some());
}

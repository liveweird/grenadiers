use std::collections::BTreeSet;

use super::{ActorInit, InputAction, MAX_INPUTS_PER_TICK, SeatInput, State, StateError, StepError};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::input::Direction;
use crate::game::rules::RulesConfig;
use crate::game::test_support::add_test_bomb;
use crate::game::types::{
    CellId, EntityId, HALF_FOOTPRINT, Position, SeatId, SpeedLevel, Tick, UNITS_PER_CELL,
};
use crate::game::world::World;

#[path = "simulation/kick_route_tests.rs"]
mod kick_route_tests;
#[path = "kick_tests.rs"]
mod kick_tests;
#[path = "simulation/punch_route_tests.rs"]
mod punch_route_tests;

fn open_world() -> World {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    World::new(
        Arena::new(
            terrain,
            BTreeSet::new(),
            [CellId::new(1, 1), CellId::new(13, 11)],
        )
        .unwrap(),
    )
}

fn init(
    id: u32,
    seat: SeatId,
    position: Position,
    heading: Direction,
    speed: SpeedLevel,
) -> ActorInit {
    ActorInit {
        id: EntityId(id),
        seat,
        position,
        heading,
        speed,
    }
}

fn state_at(position: Position, heading: Direction, speed: SpeedLevel) -> State {
    State::new(
        RulesConfig::classic_v1(),
        open_world(),
        [
            init(10, SeatId::One, position, heading, speed),
            init(
                20,
                SeatId::Two,
                Position::center(CellId::new(13, 11)),
                Direction::West,
                SpeedLevel::Base,
            ),
        ],
    )
    .unwrap()
}

fn input(seat: SeatId, action: InputAction) -> SeatInput {
    SeatInput { seat, action }
}

fn step_n(state: &mut State, count: usize) {
    for _ in 0..count {
        state.step(&[]).unwrap();
    }
}

#[test]
fn constructor_canonicalizes_seats_and_allows_actor_overlap() {
    let position = Position::center(CellId::new(3, 3));
    let one = init(1, SeatId::One, position, Direction::East, SpeedLevel::Base);
    let two = init(2, SeatId::Two, position, Direction::West, SpeedLevel::Base);
    let state = State::new(RulesConfig::classic_v1(), open_world(), [two, one]).unwrap();

    assert_eq!(state.actors()[0].seat(), SeatId::One);
    assert_eq!(state.actors()[1].seat(), SeatId::Two);
    assert_eq!(state.actors()[0].position(), state.actors()[1].position());
}

#[test]
fn constructor_rejects_invalid_identity_and_geometry() {
    let valid = Position::center(CellId::new(2, 2));
    let duplicate = State::new(
        RulesConfig::classic_v1(),
        open_world(),
        [
            init(1, SeatId::One, valid, Direction::East, SpeedLevel::Base),
            init(1, SeatId::Two, valid, Direction::West, SpeedLevel::Base),
        ],
    );
    assert_eq!(duplicate, Err(StateError::DuplicateActorId(EntityId(1))));

    let overflow = Position {
        x: i32::MAX,
        y: i32::MAX,
    };
    let invalid = State::new(
        RulesConfig::classic_v1(),
        open_world(),
        [
            init(1, SeatId::One, overflow, Direction::East, SpeedLevel::Base),
            init(2, SeatId::Two, valid, Direction::West, SpeedLevel::Base),
        ],
    );
    assert_eq!(invalid, Err(StateError::IllegalActorPosition(EntityId(1))));

    for extreme in [
        Position {
            x: i32::MIN,
            y: valid.y,
        },
        Position {
            x: valid.x,
            y: i32::MIN,
        },
        Position {
            x: i32::MAX,
            y: valid.y,
        },
        Position {
            x: valid.x,
            y: i32::MAX,
        },
    ] {
        let result = State::new(
            RulesConfig::classic_v1(),
            open_world(),
            [
                init(1, SeatId::One, extreme, Direction::East, SpeedLevel::Base),
                init(2, SeatId::Two, valid, Direction::West, SpeedLevel::Base),
            ],
        );
        assert_eq!(result, Err(StateError::IllegalActorPosition(EntityId(1))));
    }
}

#[test]
fn every_speed_moves_its_exact_sixty_tick_distance() {
    for speed in [
        SpeedLevel::Base,
        SpeedLevel::One,
        SpeedLevel::Two,
        SpeedLevel::Three,
        SpeedLevel::Four,
    ] {
        let start = Position::center(CellId::new(2, 5));
        let mut state = state_at(start, Direction::East, speed);
        state
            .step(&[input(SeatId::One, InputAction::Press(Direction::East))])
            .unwrap();
        step_n(&mut state, 59);
        assert_eq!(
            state.actors()[0].position().x - start.x,
            i32::from(speed.units_per_second())
        );
    }
}

#[test]
fn residual_progresses_fractionally_and_stop_clears_it() {
    let start = Position::center(CellId::new(2, 5));
    let mut state = state_at(start, Direction::East, SpeedLevel::Base);
    state
        .step(&[input(SeatId::One, InputAction::Press(Direction::East))])
        .unwrap();
    assert_eq!(state.actors()[0].position().x, start.x + 51);
    step_n(&mut state, 4);
    assert_eq!(state.actors()[0].position().x, start.x + 256);

    state
        .step(&[input(SeatId::One, InputAction::Clear)])
        .unwrap();
    let stopped = state.actors()[0].position().x;
    state
        .step(&[input(SeatId::One, InputAction::Press(Direction::East))])
        .unwrap();
    assert_eq!(state.actors()[0].position().x, stopped + 51);
}

#[test]
fn clear_and_repress_in_one_batch_discards_fractional_progress() {
    let start = Position::center(CellId::new(2, 5));
    let mut uninterrupted = state_at(start, Direction::East, SpeedLevel::Base);
    uninterrupted
        .step(&[input(SeatId::One, InputAction::Press(Direction::East))])
        .unwrap();
    step_n(&mut uninterrupted, 3);
    let mut restarted = uninterrupted.clone();

    uninterrupted.step(&[]).unwrap();
    restarted
        .step(&[
            input(SeatId::One, InputAction::Clear),
            input(SeatId::One, InputAction::Press(Direction::East)),
        ])
        .unwrap();
    assert_eq!(
        uninterrupted.actors()[0].position().x,
        restarted.actors()[0].position().x + 1
    );
}

#[test]
fn reversal_is_immediate_and_players_do_not_block_each_other() {
    let position = Position::center(CellId::new(5, 5));
    let mut state = State::new(
        RulesConfig::classic_v1(),
        open_world(),
        [
            init(1, SeatId::One, position, Direction::East, SpeedLevel::Four),
            init(2, SeatId::Two, position, Direction::West, SpeedLevel::Four),
        ],
    )
    .unwrap();
    state
        .step(&[
            input(SeatId::One, InputAction::Press(Direction::West)),
            input(SeatId::Two, InputAction::Press(Direction::East)),
        ])
        .unwrap();
    assert!(state.actors()[0].position().x < position.x);
    assert!(state.actors()[1].position().x > position.x);
    assert_eq!(state.actors()[0].heading(), Direction::West);
}

#[test]
fn turn_assist_accepts_128_but_never_bypasses_129() {
    let center = Position::center(CellId::new(3, 5));
    let mut eligible = state_at(
        Position {
            x: center.x + 128,
            y: center.y,
        },
        Direction::East,
        SpeedLevel::Base,
    );
    let first = eligible
        .step(&[input(SeatId::One, InputAction::Press(Direction::North))])
        .unwrap();
    assert_eq!(
        eligible.actors()[0].position(),
        Position {
            x: center.x + 77,
            y: center.y
        }
    );
    assert_eq!(eligible.actors()[0].heading(), Direction::East);
    assert_eq!(first.traces[0].len(), 1);
    let third = eligible.step(&[]).and_then(|_| eligible.step(&[])).unwrap();
    assert_eq!(
        eligible.actors()[0].position(),
        Position {
            x: center.x,
            y: center.y - 25
        }
    );
    assert_eq!(eligible.actors()[0].heading(), Direction::North);
    assert_eq!(third.traces[0].len(), 2);

    let outside = Position {
        x: center.x + 129,
        y: center.y,
    };
    let mut ineligible = state_at(outside, Direction::East, SpeedLevel::Four);
    ineligible
        .step(&[input(SeatId::One, InputAction::Press(Direction::North))])
        .unwrap();
    step_n(&mut ineligible, 200);
    assert_eq!(ineligible.actors()[0].position(), outside);
    assert_eq!(ineligible.actors()[0].heading(), Direction::East);
}

#[test]
fn changing_intent_cancels_partial_turn_and_replans() {
    let center = Position::center(CellId::new(3, 5));
    let start = Position {
        x: center.x + 128,
        y: center.y,
    };
    let mut state = state_at(start, Direction::East, SpeedLevel::Base);
    state
        .step(&[input(SeatId::One, InputAction::Press(Direction::North))])
        .unwrap();
    let after_partial = state.actors()[0].position();
    state
        .step(&[input(SeatId::One, InputAction::Clear)])
        .unwrap();
    assert_eq!(state.actors()[0].position(), after_partial);
    state
        .step(&[input(SeatId::One, InputAction::Press(Direction::West))])
        .unwrap();
    assert!(state.actors()[0].position().x < after_partial.x);
    assert_eq!(state.actors()[0].heading(), Direction::West);
}

#[test]
fn bombs_block_sweeps_and_grace_ends_after_leaving() {
    let start = Position::center(CellId::new(2, 5));
    let mut blocked = state_at(start, Direction::East, SpeedLevel::Four);
    add_test_bomb(&mut blocked, EntityId(90), CellId::new(3, 5), EntityId(10)).unwrap();
    blocked
        .step(&[input(SeatId::One, InputAction::Press(Direction::East))])
        .unwrap();
    step_n(&mut blocked, 20);
    assert_eq!(
        blocked.actors()[0].position().x,
        3 * UNITS_PER_CELL - HALF_FOOTPRINT
    );

    let mut grace = State::new(
        RulesConfig::classic_v1(),
        open_world(),
        [
            init(1, SeatId::One, start, Direction::West, SpeedLevel::Four),
            init(2, SeatId::Two, start, Direction::West, SpeedLevel::Four),
        ],
    )
    .unwrap();
    add_test_bomb(&mut grace, EntityId(91), CellId::new(2, 5), EntityId(1)).unwrap();
    grace
        .step(&[
            input(SeatId::One, InputAction::Press(Direction::West)),
            input(SeatId::Two, InputAction::Press(Direction::West)),
        ])
        .unwrap();
    step_n(&mut grace, 10);
    grace
        .step(&[
            input(SeatId::One, InputAction::Press(Direction::East)),
            input(SeatId::Two, InputAction::Press(Direction::East)),
        ])
        .unwrap();
    step_n(&mut grace, 20);
    for actor in state_positions(&grace) {
        assert_eq!(actor.x, 2 * UNITS_PER_CELL - HALF_FOOTPRINT);
    }
}

fn state_positions(state: &State) -> impl Iterator<Item = Position> + '_ {
    state.actors().iter().map(|actor| actor.position())
}

#[test]
fn invalid_steps_are_atomic_and_repetition_is_deterministic() {
    let start = Position::center(CellId::new(2, 5));
    let mut state = state_at(start, Direction::East, SpeedLevel::Base);
    let before = state.clone();
    let too_many = vec![input(SeatId::One, InputAction::Clear); MAX_INPUTS_PER_TICK + 1];
    assert_eq!(
        state.step(&too_many),
        Err(StepError::TooManyInputs {
            count: 65,
            maximum: 64
        })
    );
    assert_eq!(state, before);

    state.tick = Tick(u64::MAX);
    let before_overflow = state.clone();
    assert_eq!(state.step(&[]), Err(StepError::TickOverflow));
    assert_eq!(state, before_overflow);

    let one = init(1, SeatId::One, start, Direction::East, SpeedLevel::Base);
    let two = init(
        2,
        SeatId::Two,
        Position::center(CellId::new(12, 5)),
        Direction::West,
        SpeedLevel::Base,
    );
    let mut left = State::new(RulesConfig::classic_v1(), open_world(), [one, two]).unwrap();
    let mut right = State::new(RulesConfig::classic_v1(), open_world(), [two, one]).unwrap();
    let events = [
        input(SeatId::One, InputAction::Press(Direction::East)),
        input(SeatId::Two, InputAction::Press(Direction::West)),
    ];
    left.step(&events).unwrap();
    right.step(&events).unwrap();
    step_n(&mut left, 30);
    step_n(&mut right, 30);
    assert_eq!(left, right);
}

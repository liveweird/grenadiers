use std::collections::BTreeSet;

use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::fixtures::fixed_arena;
use crate::game::input::Direction;
use crate::game::rules::RulesConfig;
use crate::game::simulation::{ActorInit, InputAction, SeatInput, State, StateError};
use crate::game::types::{
    CellId, EntityId, HALF_FOOTPRINT, Position, SeatId, SpeedLevel, UNITS_PER_CELL,
};
use crate::game::world::World;

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

fn state(position: Position, heading: Direction) -> State {
    state_in_world(open_world(), position, heading)
}

fn state_in_world(world: World, position: Position, heading: Direction) -> State {
    State::new(
        RulesConfig::classic_v1(),
        world,
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position,
                heading,
                speed: SpeedLevel::Four,
            },
            ActorInit {
                id: EntityId(2),
                seat: SeatId::Two,
                position: Position::center(CellId::new(13, 11)),
                heading: Direction::West,
                speed: SpeedLevel::Base,
            },
        ],
    )
    .unwrap()
}

fn move_for(state: &mut State, direction: Direction, ticks: usize) {
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(direction),
        }])
        .unwrap();
    for _ in 1..ticks {
        state.step(&[]).unwrap();
    }
}

#[test]
fn maximum_speed_clamps_to_bomb_boundaries_on_every_axis() {
    let cases = [
        (
            Direction::East,
            CellId::new(4, 5),
            CellId::new(5, 5),
            5 * UNITS_PER_CELL - HALF_FOOTPRINT,
            true,
        ),
        (
            Direction::West,
            CellId::new(4, 5),
            CellId::new(3, 5),
            4 * UNITS_PER_CELL + HALF_FOOTPRINT,
            true,
        ),
        (
            Direction::South,
            CellId::new(4, 5),
            CellId::new(4, 6),
            6 * UNITS_PER_CELL - HALF_FOOTPRINT,
            false,
        ),
        (
            Direction::North,
            CellId::new(4, 5),
            CellId::new(4, 4),
            5 * UNITS_PER_CELL + HALF_FOOTPRINT,
            false,
        ),
    ];

    for (direction, start, obstacle, expected, horizontal) in cases {
        let mut state = state(Position::center(start), direction);
        state.add_bomb_obstacle(EntityId(90), obstacle).unwrap();
        state
            .step(&[SeatInput {
                seat: SeatId::One,
                action: InputAction::Press(direction),
            }])
            .unwrap();
        for _ in 0..4 {
            state.step(&[]).unwrap();
        }
        let position = state.actors()[0].position();
        assert_eq!(if horizontal { position.x } else { position.y }, expected);
    }
}

#[test]
fn stopped_partial_turn_replans_and_pending_axis_can_reverse() {
    let center = Position::center(CellId::new(4, 5));
    let start = Position {
        x: center.x + 128,
        y: center.y,
    };
    let mut resumed = state(start, Direction::East);
    resumed
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::North),
        }])
        .unwrap();
    let partial = resumed.actors()[0].position();
    resumed
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Clear,
        }])
        .unwrap();
    assert_eq!(resumed.actors()[0].position(), partial);
    resumed
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::North),
        }])
        .unwrap();
    resumed.step(&[]).unwrap();
    assert_eq!(resumed.actors()[0].heading(), Direction::North);
    assert!(resumed.actors()[0].position().y < center.y);

    let mut reversed = state(start, Direction::East);
    reversed
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::North),
        }])
        .unwrap();
    reversed
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::South),
        }])
        .unwrap();
    reversed.step(&[]).unwrap();
    assert_eq!(reversed.actors()[0].heading(), Direction::South);
    assert!(reversed.actors()[0].position().y > center.y);
}

#[test]
fn bomb_and_actor_identity_errors_leave_state_unchanged() {
    let mut original = state(Position::center(CellId::new(2, 5)), Direction::East);
    original
        .add_bomb_obstacle(EntityId(90), CellId::new(8, 5))
        .unwrap();
    let before = original.clone();
    assert_eq!(
        original.add_bomb_obstacle(EntityId(90), CellId::new(9, 5)),
        Err(StateError::DuplicateEntityId(EntityId(90)))
    );
    assert_eq!(original, before);

    let rebuilt = State::new(
        RulesConfig::classic_v1(),
        original.world().clone(),
        [
            ActorInit {
                id: EntityId(90),
                seat: SeatId::One,
                position: Position::center(CellId::new(2, 5)),
                heading: Direction::East,
                speed: SpeedLevel::Base,
            },
            ActorInit {
                id: EntityId(2),
                seat: SeatId::Two,
                position: Position::center(CellId::new(13, 11)),
                heading: Direction::West,
                speed: SpeedLevel::Base,
            },
        ],
    );
    assert_eq!(rebuilt, Err(StateError::DuplicateEntityId(EntityId(90))));
}

#[test]
fn hard_walls_and_soft_blocks_clamp_on_exact_footprint_edges() {
    let mut hard = state_in_world(
        World::new(fixed_arena()),
        Position::center(CellId::new(1, 2)),
        Direction::East,
    );
    move_for(&mut hard, Direction::East, 4);
    assert_eq!(
        hard.actors()[0].position().x,
        2 * UNITS_PER_CELL - HALF_FOOTPRINT
    );

    let mut soft = state_in_world(
        World::new(fixed_arena()),
        Position::center(CellId::new(3, 1)),
        Direction::South,
    );
    move_for(&mut soft, Direction::South, 4);
    assert_eq!(
        soft.actors()[0].position().y,
        2 * UNITS_PER_CELL - HALF_FOOTPRINT
    );
}

#[test]
fn touching_a_corner_glides_past_without_clipping_into_the_wall() {
    let start = Position {
        x: Position::center(CellId::new(1, 1)).x,
        y: 2 * UNITS_PER_CELL - HALF_FOOTPRINT,
    };
    let mut state = state_in_world(World::new(fixed_arena()), start, Direction::East);
    move_for(&mut state, Direction::East, 12);

    assert_eq!(state.actors()[0].position().x, start.x + UNITS_PER_CELL);
    assert_eq!(state.actors()[0].position().y, start.y);
}

#[test]
fn pickups_and_hazards_do_not_block_movement() {
    let mut world = open_world();
    let destination = CellId::new(3, 5);
    world.pickups.insert(destination);
    world.hazards.insert(destination);
    let start = Position::center(CellId::new(2, 5));
    let mut state = state_in_world(world, start, Direction::East);
    move_for(&mut state, Direction::East, 12);

    assert_eq!(state.actors()[0].position(), Position::center(destination));
}

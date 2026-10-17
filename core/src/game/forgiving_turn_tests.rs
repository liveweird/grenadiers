use std::collections::BTreeSet;

use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::input::Direction;
use crate::game::rules::RulesConfig;
use crate::game::simulation::{ActorInit, InputAction, SeatInput, State};
use crate::game::types::{
    CellId, EntityId, HALF_FOOTPRINT, Position, SeatId, SpeedLevel, Tick, UNITS_PER_CELL,
};
use crate::game::world::World;

fn open_arena(blocks: BTreeSet<CellId>) -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    Arena::new(terrain, blocks, [CellId::new(1, 1), CellId::new(13, 11)]).unwrap()
}

fn state(
    world: World,
    position: Position,
    heading: Direction,
    speed: SpeedLevel,
    rules: RulesConfig,
) -> State {
    State::new(
        rules,
        world,
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position,
                heading,
                speed,
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

fn press_and_step(state: &mut State, direction: Direction, ticks: usize) {
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
fn inclusive_quarter_tile_turns_on_every_axis_at_every_speed() {
    let center = Position::center(CellId::new(5, 5));
    let directions = [
        (Direction::North, Direction::East),
        (Direction::South, Direction::East),
        (Direction::East, Direction::North),
        (Direction::West, Direction::North),
    ];
    let speeds = [
        SpeedLevel::Base,
        SpeedLevel::One,
        SpeedLevel::Two,
        SpeedLevel::Three,
        SpeedLevel::Four,
    ];

    for speed in speeds {
        for (requested, heading) in directions {
            for offset in [-256, 256] {
                let start = match requested {
                    Direction::North | Direction::South => Position {
                        x: center.x + offset,
                        y: center.y,
                    },
                    Direction::East | Direction::West => Position {
                        x: center.x,
                        y: center.y + offset,
                    },
                };
                let mut state = state(
                    World::new(open_arena(BTreeSet::new())),
                    start,
                    heading,
                    speed,
                    RulesConfig::classic_v3(),
                );
                press_and_step(&mut state, requested, 8);

                let actor = &state.actors()[0];
                assert_eq!(actor.heading(), requested, "{requested:?} at {speed:?}");
                match requested {
                    Direction::North => {
                        assert_eq!(actor.position().x, center.x);
                        assert!(actor.position().y < center.y);
                    }
                    Direction::South => {
                        assert_eq!(actor.position().x, center.x);
                        assert!(actor.position().y > center.y);
                    }
                    Direction::East => {
                        assert_eq!(actor.position().y, center.y);
                        assert!(actor.position().x > center.x);
                    }
                    Direction::West => {
                        assert_eq!(actor.position().y, center.y);
                        assert!(actor.position().x < center.x);
                    }
                }
            }
        }
    }
}

#[test]
fn offsets_beyond_quarter_tile_are_rejected_and_legacy_profiles_stay_narrow() {
    let center = Position::center(CellId::new(5, 5));
    let outside = Position {
        x: center.x + 257,
        y: center.y,
    };
    let mut forgiving = state(
        World::new(open_arena(BTreeSet::new())),
        outside,
        Direction::East,
        SpeedLevel::Four,
        RulesConfig::classic_v3(),
    );
    press_and_step(&mut forgiving, Direction::North, 20);
    assert_eq!(forgiving.actors()[0].position(), outside);
    assert_eq!(forgiving.actors()[0].heading(), Direction::East);

    for rules in [RulesConfig::classic_v1(), RulesConfig::classic_v2()] {
        let legacy_offset = Position {
            x: center.x + 129,
            y: center.y,
        };
        let mut legacy = state(
            World::new(open_arena(BTreeSet::new())),
            legacy_offset,
            Direction::East,
            SpeedLevel::Four,
            rules,
        );
        press_and_step(&mut legacy, Direction::North, 20);
        assert_eq!(legacy.actors()[0].position(), legacy_offset);
        assert_eq!(legacy.actors()[0].heading(), Direction::East);
    }
}

#[test]
fn forgiving_turns_still_respect_blockers_and_swept_flame_damage() {
    let center = Position::center(CellId::new(5, 5));
    let start = Position {
        x: center.x + 256,
        y: center.y,
    };
    let mut blocked = state(
        World::new(open_arena(BTreeSet::from([CellId::new(5, 4)]))),
        start,
        Direction::East,
        SpeedLevel::Four,
        RulesConfig::classic_v3(),
    );
    press_and_step(&mut blocked, Direction::North, 20);
    assert_eq!(blocked.actors()[0].heading(), Direction::North);
    assert_eq!(blocked.actors()[0].position().x, center.x);
    assert_eq!(
        blocked.actors()[0].position().y,
        5 * UNITS_PER_CELL + HALF_FOOTPRINT
    );

    let mut burning_world = World::new(open_arena(BTreeSet::new()));
    burning_world.hazards.insert(CellId::new(5, 4), Tick(100));
    let mut burning = state(
        burning_world,
        start,
        Direction::East,
        SpeedLevel::Four,
        RulesConfig::classic_v3(),
    );
    press_and_step(&mut burning, Direction::North, 20);
    assert!(!burning.actors()[0].alive());
}

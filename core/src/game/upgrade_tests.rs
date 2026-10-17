use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;

use super::{ActorInit, InputAction, RoundResult, SeatInput, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::assignment::RoundAssignment;
use crate::game::events::StepEventKind;
use crate::game::input::Direction;
use crate::game::pickups::{Pickup, PickupKind};
use crate::game::rules::RulesConfig;
use crate::game::test_support::add_test_bomb;
use crate::game::types::{CellId, EntityId, Position, SeatId, SpeedLevel, Tick, UNITS_PER_CELL};
use crate::game::world::World;

const TARGET: CellId = CellId::new(5, 5);

fn open_arena() -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row * ARENA_WIDTH + column)] = Terrain::Wall;
            }
        }
    }
    Arena::new(
        terrain,
        BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap()
}

fn world_with_pickups(pickups: impl IntoIterator<Item = (CellId, PickupKind)>) -> World {
    World::with_pickups(
        open_arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        pickups.into_iter().collect(),
    )
    .unwrap()
}

fn actor(id: u32, seat: SeatId, position: Position, speed: SpeedLevel) -> ActorInit {
    ActorInit {
        id: EntityId(id),
        seat,
        position,
        heading: Direction::East,
        speed,
    }
}

fn state_with(
    rules: RulesConfig,
    world: World,
    one: Position,
    two: Position,
    assignment: RoundAssignment,
) -> State {
    State::new_at_tick_with_assignment(
        rules,
        world,
        [
            actor(10, SeatId::One, one, SpeedLevel::Base),
            actor(20, SeatId::Two, two, SpeedLevel::Base),
        ],
        Tick(0),
        assignment,
    )
    .unwrap()
}

fn v2_state(world: World, one: Position, two: Position) -> State {
    state_with(
        RulesConfig::classic_v2(),
        world,
        one,
        two,
        RoundAssignment::first(),
    )
}

fn safe_two() -> Position {
    Position::center(CellId::new(12, 10))
}

fn expose(state: &mut State, cell: CellId, kind: PickupKind) {
    let previous = state.world.pickups.exposed.insert(cell, Pickup::new(kind));
    assert!(previous.is_none());
}

fn collect_again(state: &mut State, kind: PickupKind) -> StepEventKind {
    expose(state, TARGET, kind);
    let report = state.step(&[]).unwrap();
    assert_eq!(report.events.len(), 1);
    report.events[0].kind.clone()
}

#[test]
fn all_upgrades_cap_and_still_consume_and_emit_at_the_cap() {
    let mut capacity = v2_state(
        World::new(open_arena()),
        Position::center(TARGET),
        safe_two(),
    );
    for expected in 2..=8 {
        assert!(matches!(
            collect_again(&mut capacity, PickupKind::BombCapacity),
            StepEventKind::PickupCollected {
                kind: PickupKind::BombCapacity,
                ..
            }
        ));
        assert_eq!(capacity.actors()[0].bomb_capacity(), expected);
    }
    collect_again(&mut capacity, PickupKind::BombCapacity);
    assert_eq!(capacity.actors()[0].bomb_capacity(), 8);
    let mut range = v2_state(
        World::new(open_arena()),
        Position::center(TARGET),
        safe_two(),
    );
    for expected in 3..=8 {
        collect_again(&mut range, PickupKind::BlastRange);
        assert_eq!(range.actors()[0].bomb_range(), expected);
    }
    collect_again(&mut range, PickupKind::BlastRange);
    assert_eq!(range.actors()[0].bomb_range(), 8);
    let mut speed = v2_state(
        World::new(open_arena()),
        Position::center(TARGET),
        safe_two(),
    );
    for expected in [
        SpeedLevel::One,
        SpeedLevel::Two,
        SpeedLevel::Three,
        SpeedLevel::Four,
    ] {
        collect_again(&mut speed, PickupKind::MovementSpeed);
        assert_eq!(speed.actors()[0].speed(), expected);
    }
    collect_again(&mut speed, PickupKind::MovementSpeed);
    assert_eq!(speed.actors()[0].speed(), SpeedLevel::Four);
    assert!(speed.world().exposed_pickups().is_empty());
}

#[test]
fn closest_actor_wins_and_exact_ties_follow_both_round_parities() {
    let center = Position::center(TARGET);
    let farther = Position {
        x: center.x - 200,
        y: center.y,
    };
    let closer = Position {
        x: center.x + 50,
        y: center.y,
    };
    let mut closest = v2_state(
        world_with_pickups([(TARGET, PickupKind::BlastRange)]),
        farther,
        closer,
    );
    let report = closest.step(&[]).unwrap();
    assert!(matches!(
        report.events[0].kind,
        StepEventKind::PickupCollected {
            seat: SeatId::Two,
            ..
        }
    ));
    for (round, winner) in [(1, SeatId::One), (2, SeatId::Two)] {
        let assignment = RoundAssignment::new(NonZeroU32::new(round).unwrap());
        let mut tied = state_with(
            RulesConfig::classic_v2(),
            world_with_pickups([(TARGET, PickupKind::BlastRange)]),
            Position {
                x: center.x - 100,
                y: center.y,
            },
            Position {
                x: center.x + 100,
                y: center.y,
            },
            assignment,
        );
        let report = tied.step(&[]).unwrap();
        assert!(
            matches!(report.events[0].kind, StepEventKind::PickupCollected { seat, .. } if seat == winner)
        );
    }
}

#[test]
fn assignment_is_independent_of_actor_array_order_and_does_not_teleport_fixtures() {
    let one = actor(
        10,
        SeatId::One,
        Position::center(CellId::new(4, 5)),
        SpeedLevel::Base,
    );
    let two = actor(
        20,
        SeatId::Two,
        Position::center(CellId::new(9, 7)),
        SpeedLevel::Base,
    );
    for round in [1, 2] {
        let assignment = RoundAssignment::new(NonZeroU32::new(round).unwrap());
        let state = State::new_at_tick_with_assignment(
            RulesConfig::classic_v2(),
            World::new(open_arena()),
            [two, one],
            Tick(77),
            assignment,
        )
        .unwrap();
        assert_eq!(state.actors()[0].position(), one.position);
        assert_eq!(state.actors()[1].position(), two.position);
        assert_eq!(state.round_assignment(), assignment);
        assert_eq!(
            assignment.priority_seat(),
            if round == 1 { SeatId::One } else { SeatId::Two }
        );
    }
}

#[test]
fn damage_precedes_collection_and_collection_precedes_round_end() {
    let dead_cell = CellId::new(4, 5);
    let survivor_cell = CellId::new(9, 5);
    let mut world = world_with_pickups([
        (dead_cell, PickupKind::BombCapacity),
        (survivor_cell, PickupKind::BlastRange),
    ]);
    world.hazards.insert(dead_cell, Tick(10));
    let mut state = v2_state(
        world,
        Position::center(dead_cell),
        Position::center(survivor_cell),
    );
    let report = state.step(&[]).unwrap();
    assert_eq!(state.result(), Some(RoundResult::Winner(SeatId::Two)));
    assert!(state.world().exposed_pickups().contains_key(&dead_cell));
    assert!(!state.world().exposed_pickups().contains_key(&survivor_cell));
    assert!(matches!(
        report.events[0].kind,
        StepEventKind::ActorEliminated {
            seat: SeatId::One,
            ..
        }
    ));
    assert!(matches!(
        report.events[1].kind,
        StepEventKind::PickupCollected {
            seat: SeatId::Two,
            ..
        }
    ));
    assert!(matches!(
        report.events[2].kind,
        StepEventKind::RoundEnded(RoundResult::Winner(SeatId::Two))
    ));
}

#[test]
fn crossing_without_ending_in_the_item_cell_does_not_collect() {
    let right_edge = i32::from(TARGET.column + 1) * UNITS_PER_CELL;
    let start = Position {
        x: right_edge - 20,
        y: Position::center(TARGET).y,
    };
    let mut state = v2_state(
        world_with_pickups([(TARGET, PickupKind::MovementSpeed)]),
        start,
        safe_two(),
    );
    state.actors[0].speed = SpeedLevel::Four;
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    assert_eq!(state.actors()[0].position().cell(), Some(CellId::new(6, 5)));
    assert!(state.world().exposed_pickups().contains_key(&TARGET));
}

#[test]
fn classic_v1_and_completed_rounds_do_not_collect() {
    let pickup = (TARGET, PickupKind::BombCapacity);
    let mut v1 = state_with(
        RulesConfig::classic_v1(),
        world_with_pickups([pickup]),
        Position::center(TARGET),
        safe_two(),
        RoundAssignment::first(),
    );
    assert!(v1.step(&[]).unwrap().events.is_empty());
    assert!(v1.world().exposed_pickups().contains_key(&TARGET));
    let mut frozen = v2_state(
        World::new(open_arena()),
        Position::center(CellId::new(4, 5)),
        safe_two(),
    );
    frozen.world.hazards.insert(CellId::new(4, 5), Tick(10));
    frozen.step(&[]).unwrap();
    expose(
        &mut frozen,
        safe_two().cell().unwrap(),
        PickupKind::BlastRange,
    );
    let report = frozen.step(&[]).unwrap();
    assert!(report.events.is_empty());
    assert!(
        frozen
            .world()
            .exposed_pickups()
            .contains_key(&safe_two().cell().unwrap())
    );
}

#[path = "upgrade_placement_tests.rs"]
mod placement;

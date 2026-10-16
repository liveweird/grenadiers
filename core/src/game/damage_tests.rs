use std::collections::{BTreeMap, BTreeSet};

use super::step::trace_hits_hazards;
use super::{ActorInit, DrawReason, InputAction, RoundResult, SeatInput, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::geometry::{MovementSegment, MovementTrace};
use crate::game::input::Direction;
use crate::game::rules::RulesConfig;
use crate::game::test_support::add_test_bomb_with_deadline;
use crate::game::types::{
    CellId, EntityId, HALF_FOOTPRINT, Position, SeatId, SpeedLevel, Tick, UNITS_PER_CELL,
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

fn actor(id: u32, seat: SeatId, position: Position) -> ActorInit {
    ActorInit {
        id: EntityId(id),
        seat,
        position,
        heading: Direction::East,
        speed: SpeedLevel::Four,
    }
}

fn state(one: Position, two: Position) -> State {
    State::new(
        RulesConfig::classic_v1(),
        open_world(),
        [actor(10, SeatId::One, one), actor(20, SeatId::Two, two)],
    )
    .unwrap()
}

fn input(seat: SeatId, action: InputAction) -> SeatInput {
    SeatInput { seat, action }
}

#[test]
fn stationary_overlap_kills_but_exact_edge_contact_does_not() {
    let flame = CellId::new(4, 5);
    let safe = Position::center(CellId::new(12, 10));
    let mut overlapping = state(Position::center(flame), safe);
    overlapping.world.hazards.insert(flame, Tick(10));

    overlapping.step(&[]).unwrap();
    assert!(!overlapping.actors()[0].alive());
    assert_eq!(overlapping.result(), Some(RoundResult::Winner(SeatId::Two)));

    let touching = Position {
        x: i32::from(flame.column) * UNITS_PER_CELL - HALF_FOOTPRINT,
        y: Position::center(flame).y,
    };
    let mut edge = state(touching, safe);
    edge.world.hazards.insert(flame, Tick(10));
    edge.step(&[]).unwrap();
    assert!(edge.actors().iter().all(|actor| actor.alive()));
    assert_eq!(edge.result(), None);
}

#[test]
fn flames_expire_before_damage_at_their_end_tick() {
    let flame = CellId::new(4, 5);
    let one = Position::center(flame);
    let two = Position::center(CellId::new(12, 10));
    let mut expiring = state(one, two);
    expiring.world.hazards.insert(flame, Tick(1));

    expiring.step(&[]).unwrap();
    assert!(expiring.actors().iter().all(|actor| actor.alive()));
    assert!(!expiring.world().hazards().contains_key(&flame));

    let mut still_active = state(one, two);
    still_active.world.hazards.insert(flame, Tick(2));
    still_active.step(&[]).unwrap();
    assert!(!still_active.actors()[0].alive());
    assert_eq!(
        still_active.result(),
        Some(RoundResult::Winner(SeatId::Two))
    );
}

#[test]
fn movement_trace_detects_entering_and_starting_in_a_flame() {
    let flame = CellId::new(4, 5);
    let flame_left = i32::from(flame.column) * UNITS_PER_CELL;
    let safe = Position::center(CellId::new(12, 10));
    let mut entering = state(
        Position {
            x: flame_left - HALF_FOOTPRINT - 20,
            y: Position::center(flame).y,
        },
        safe,
    );
    entering.world.hazards.insert(flame, Tick(10));
    let report = entering
        .step(&[input(SeatId::One, InputAction::Press(Direction::East))])
        .unwrap();
    assert!(!report.traces[0].is_empty());
    assert!(!entering.actors()[0].alive());

    let mut leaving = state(
        Position {
            x: flame_left - HALF_FOOTPRINT + 1,
            y: Position::center(flame).y,
        },
        safe,
    );
    leaving.world.hazards.insert(flame, Tick(10));
    leaving
        .step(&[input(SeatId::One, InputAction::Press(Direction::West))])
        .unwrap();
    assert!(leaving.actors()[0].position().x < flame_left - HALF_FOOTPRINT);
    assert!(!leaving.actors()[0].alive());
}

#[test]
fn turn_correction_contact_kills_even_when_the_final_footprint_is_safe() {
    let flame = CellId::new(4, 6);
    let mut state = state(
        Position {
            x: Position::center(CellId::new(4, 5)).x + 50,
            y: i32::from(flame.row) * UNITS_PER_CELL - HALF_FOOTPRINT + 20,
        },
        Position::center(CellId::new(12, 10)),
    );
    state.world.hazards.insert(flame, Tick(10));

    let report = state
        .step(&[input(SeatId::One, InputAction::Press(Direction::North))])
        .unwrap();

    assert_eq!(report.traces[0].len(), 2);
    assert!(
        state.actors()[0].position().y + HALF_FOOTPRINT <= i32::from(flame.row) * UNITS_PER_CELL
    );
    assert!(!state.actors()[0].alive());
}

#[test]
fn piecewise_trace_does_not_damage_through_the_diagonal_gap() {
    let start = Position { x: 512, y: 512 };
    let corner = Position { x: 512, y: 1536 };
    let end = Position { x: 1536, y: 1536 };
    let mut trace = MovementTrace::new(start);
    trace.push(MovementSegment::new(start, corner));
    trace.push(MovementSegment::new(corner, end));
    let hazards = BTreeMap::from([(CellId::new(1, 0), Tick(10))]);

    assert!(!trace_hits_hazards(&trace, &hazards));
}

#[test]
fn old_flame_death_suppresses_same_tick_placement() {
    let cell = CellId::new(4, 5);
    let mut state = state(
        Position::center(cell),
        Position::center(CellId::new(12, 10)),
    );
    state.world.hazards.insert(cell, Tick(10));

    state
        .step(&[input(SeatId::One, InputAction::PlaceBomb)])
        .unwrap();

    assert!(!state.actors()[0].alive());
    assert!(state.world().bombs().is_empty());
    assert_eq!(state.live_bomb_count(SeatId::One), 0);
}

#[test]
fn dead_owners_bombs_complete_the_current_chain_before_the_result() {
    let old_flame = CellId::new(2, 9);
    let mut state = state(
        Position::center(old_flame),
        Position::center(CellId::new(12, 10)),
    );
    state.world.hazards.insert(old_flame, Tick(10));
    add_test_bomb_with_deadline(
        &mut state,
        EntityId(30),
        CellId::new(5, 5),
        EntityId(10),
        2,
        Tick(0),
        Tick(1),
    )
    .unwrap();
    add_test_bomb_with_deadline(
        &mut state,
        EntityId(31),
        CellId::new(7, 5),
        EntityId(20),
        2,
        Tick(0),
        Tick(999),
    )
    .unwrap();

    state.step(&[]).unwrap();

    assert!(state.world().bombs().is_empty());
    assert!(state.world().hazards().contains_key(&CellId::new(9, 5)));
    assert_eq!(state.result(), Some(RoundResult::Winner(SeatId::Two)));
}

#[test]
fn new_flame_does_not_retroactively_damage_the_earlier_movement_path() {
    let flame = CellId::new(2, 5);
    let mut state = state(
        Position {
            x: i32::from(flame.column) * UNITS_PER_CELL - HALF_FOOTPRINT + 1,
            y: Position::center(flame).y,
        },
        Position::center(CellId::new(12, 10)),
    );
    add_test_bomb_with_deadline(
        &mut state,
        EntityId(30),
        CellId::new(4, 5),
        EntityId(20),
        2,
        Tick(0),
        Tick(1),
    )
    .unwrap();

    let report = state
        .step(&[input(SeatId::One, InputAction::Press(Direction::West))])
        .unwrap();

    assert!(report.traces[0].overlaps_cell(flame));
    assert!(state.world().hazards().contains_key(&flame));
    assert!(
        state.actors()[0].position().x + HALF_FOOTPRINT <= i32::from(flame.column) * UNITS_PER_CELL
    );
    assert!(state.actors().iter().all(|actor| actor.alive()));
    assert_eq!(state.result(), None);
}

#[test]
fn one_blast_can_self_eliminate_and_a_complete_chain_can_draw() {
    let mut self_kill = state(
        Position::center(CellId::new(4, 5)),
        Position::center(CellId::new(12, 10)),
    );
    add_test_bomb_with_deadline(
        &mut self_kill,
        EntityId(30),
        CellId::new(4, 5),
        EntityId(10),
        2,
        Tick(0),
        Tick(1),
    )
    .unwrap();
    self_kill.step(&[]).unwrap();
    assert_eq!(self_kill.result(), Some(RoundResult::Winner(SeatId::Two)));

    let mut chain = state(
        Position::center(CellId::new(4, 5)),
        Position::center(CellId::new(6, 5)),
    );
    add_test_bomb_with_deadline(
        &mut chain,
        EntityId(30),
        CellId::new(2, 5),
        EntityId(10),
        2,
        Tick(0),
        Tick(1),
    )
    .unwrap();
    add_test_bomb_with_deadline(
        &mut chain,
        EntityId(31),
        CellId::new(4, 5),
        EntityId(20),
        2,
        Tick(0),
        Tick(999),
    )
    .unwrap();
    chain.step(&[]).unwrap();
    assert_eq!(
        chain.result(),
        Some(RoundResult::Draw(DrawReason::MutualElimination))
    );
    assert!(chain.world().bombs().is_empty());
}

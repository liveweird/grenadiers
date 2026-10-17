use std::collections::{BTreeMap, BTreeSet};

use super::{ActorInit, DrawReason, InputAction, RoundResult, SeatInput, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::events::StepEventKind;
use crate::game::input::Direction;
use crate::game::pickups::PickupKind;
use crate::game::rules::RulesConfig;
use crate::game::test_support::add_test_bomb_with_deadline;
use crate::game::types::{CellId, EntityId, Position, SeatId, SpeedLevel, Tick};
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

fn actor(id: u32, seat: SeatId, cell: CellId) -> ActorInit {
    ActorInit {
        id: EntityId(id),
        seat,
        position: Position::center(cell),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    }
}

fn state_at(one: CellId, two: CellId) -> State {
    State::new(
        RulesConfig::classic_v1(),
        open_world(),
        [actor(10, SeatId::One, one), actor(20, SeatId::Two, two)],
    )
    .unwrap()
}

fn place(seat: SeatId) -> SeatInput {
    SeatInput {
        seat,
        action: InputAction::PlaceBomb,
    }
}

#[test]
fn same_tick_placement_chain_damage_and_result_follow_phase_order() {
    let shared = CellId::new(5, 5);
    let mut state = state_at(shared, shared);
    add_test_bomb_with_deadline(
        &mut state,
        EntityId(30),
        CellId::new(3, 5),
        EntityId(20),
        2,
        Tick(0),
        Tick(1),
    )
    .unwrap();

    let report = state.step(&[place(SeatId::One)]).unwrap();

    assert!(report.events.iter().all(|event| event.tick == Tick(1)));
    assert_eq!(report.events.len(), 6);
    let StepEventKind::BombPlaced(placed) = report.events[0].kind else {
        panic!("placement must be first");
    };
    assert_eq!(placed.id, EntityId(31));
    assert_eq!(placed.owner, EntityId(10));
    assert_eq!(placed.cell, shared);
    assert!(matches!(
        report.events[1].kind,
        StepEventKind::BombDetonated(bomb) if bomb.id == EntityId(30)
    ));
    assert!(matches!(
        report.events[2].kind,
        StepEventKind::BombDetonated(bomb) if bomb.id == EntityId(31)
    ));
    assert_eq!(
        report.events[3].kind,
        StepEventKind::ActorEliminated {
            id: EntityId(10),
            seat: SeatId::One,
        }
    );
    assert_eq!(
        report.events[4].kind,
        StepEventKind::ActorEliminated {
            id: EntityId(20),
            seat: SeatId::Two,
        }
    );
    assert_eq!(
        report.events[5].kind,
        StepEventKind::RoundEnded(RoundResult::Draw(DrawReason::MutualElimination))
    );
    assert!(state.world().bombs().is_empty());
}

#[test]
fn old_fire_elimination_precedes_survivor_placement_and_round_end() {
    let burning = CellId::new(4, 5);
    let survivor = CellId::new(10, 5);
    let mut state = state_at(burning, survivor);
    state.world.hazards.insert(burning, Tick(10));

    let report = state.step(&[place(SeatId::Two)]).unwrap();

    assert_eq!(report.events.len(), 3);
    assert!(matches!(
        report.events[0].kind,
        StepEventKind::ActorEliminated {
            seat: SeatId::One,
            ..
        }
    ));
    assert!(matches!(
        report.events[1].kind,
        StepEventKind::BombPlaced(bomb) if bomb.owner == EntityId(20) && bomb.cell == survivor
    ));
    assert_eq!(
        report.events[2].kind,
        StepEventKind::RoundEnded(RoundResult::Winner(SeatId::Two))
    );

    let frozen = state.step(&[place(SeatId::Two)]).unwrap();
    assert!(frozen.events.is_empty());
    assert!(frozen.traces.iter().all(|trace| trace.is_empty()));
}

#[test]
fn rejected_placement_and_test_fixture_insertion_emit_nothing() {
    let mut state = state_at(CellId::new(2, 5), CellId::new(12, 10));
    add_test_bomb_with_deadline(
        &mut state,
        EntityId(30),
        CellId::new(8, 5),
        EntityId(10),
        2,
        Tick(0),
        Tick(99),
    )
    .unwrap();

    let report = state.step(&[place(SeatId::One)]).unwrap();

    assert!(report.events.is_empty());
    assert_eq!(state.world().bombs().len(), 1);
}

#[test]
fn canonical_events_ignore_actor_and_distinct_input_order() {
    let inits = [
        actor(10, SeatId::One, CellId::new(2, 5)),
        actor(20, SeatId::Two, CellId::new(12, 10)),
    ];
    let mut first = State::new(RulesConfig::classic_v1(), open_world(), inits).unwrap();
    let mut second = State::new(
        RulesConfig::classic_v1(),
        open_world(),
        [inits[1], inits[0]],
    )
    .unwrap();

    let left = first
        .step(&[place(SeatId::Two), place(SeatId::One)])
        .unwrap();
    let right = second
        .step(&[place(SeatId::One), place(SeatId::Two)])
        .unwrap();

    assert_eq!(left.events, right.events);
    assert!(matches!(
        left.events[0].kind,
        StepEventKind::BombPlaced(bomb) if bomb.owner == EntityId(10)
    ));
    assert!(matches!(
        left.events[1].kind,
        StepEventKind::BombPlaced(bomb) if bomb.owner == EntityId(20)
    ));
}

#[test]
fn pickup_events_use_cell_order_and_canonical_actor_identity() {
    let first_cell = CellId::new(9, 4);
    let second_cell = CellId::new(3, 8);
    let world = World::with_pickups(
        open_world().arena().clone(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([
            (second_cell, PickupKind::MovementSpeed),
            (first_cell, PickupKind::BombCapacity),
        ]),
    )
    .unwrap();
    let inits = [
        actor(10, SeatId::One, first_cell),
        actor(20, SeatId::Two, second_cell),
    ];
    let mut state = State::new(RulesConfig::classic_v2(), world, [inits[1], inits[0]]).unwrap();

    let report = state.step(&[]).unwrap();

    assert_eq!(report.events.len(), 2);
    assert!(matches!(
        report.events[0].kind,
        StepEventKind::PickupCollected {
            id: EntityId(10),
            seat: SeatId::One,
            cell,
            kind: PickupKind::BombCapacity,
        } if cell == first_cell
    ));
    assert!(matches!(
        report.events[1].kind,
        StepEventKind::PickupCollected {
            id: EntityId(20),
            seat: SeatId::Two,
            cell,
            kind: PickupKind::MovementSpeed,
        } if cell == second_cell
    ));
}

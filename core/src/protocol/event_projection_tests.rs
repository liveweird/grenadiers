use std::collections::{BTreeMap, BTreeSet};

use jsonschema::validator_for;
use serde_json::{Value, json};

use super::{EventKind, ProjectionError, Seat, project_events};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::input::Direction;
use crate::game::pickups::PickupKind;
use crate::game::rules::RulesConfig;
use crate::game::simulation::{ActorInit, InputAction, SeatInput, State};
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

fn state_at(one: CellId, two: CellId) -> State {
    let actor = |id, seat, cell| ActorInit {
        id: EntityId(id),
        seat,
        position: Position::center(cell),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    };
    State::new(
        RulesConfig::classic_v1(),
        open_world(),
        [actor(10, SeatId::One, one), actor(20, SeatId::Two, two)],
    )
    .unwrap()
}

fn place_one() -> SeatInput {
    SeatInput {
        seat: SeatId::One,
        action: InputAction::PlaceBomb,
    }
}

fn chain_report() -> crate::game::StepReport {
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
    state.step(&[place_one()]).unwrap()
}

fn event_schema() -> jsonschema::Validator {
    let mut schema: Value =
        serde_json::from_str(include_str!("../../../api/gameplay.schema.json")).unwrap();
    schema["$ref"] = json!("#/$defs/WireEvent");
    validator_for(&schema).unwrap()
}

#[test]
fn chain_report_projects_every_phase_event_in_original_order() {
    let report = chain_report();
    let original = report.clone();

    let projected = project_events(std::slice::from_ref(&report)).unwrap();

    assert_eq!(report, original);
    assert_eq!(projected.len(), 6);
    assert!(projected.iter().all(|event| event.tick.get() == 1));
    assert_eq!(
        projected
            .iter()
            .map(|event| event.index)
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3, 4, 5]
    );
    assert!(matches!(
        projected[0].kind,
        EventKind::BombPlaced { bomb } if bomb.id.get() == 31
    ));
    assert!(matches!(
        projected[1].kind,
        EventKind::BombDetonated { bomb } if bomb.id.get() == 30
    ));
    assert!(matches!(
        projected[2].kind,
        EventKind::BombDetonated { bomb } if bomb.id.get() == 31
    ));
    assert!(matches!(
        projected[3].kind,
        EventKind::ActorEliminated { .. }
    ));
    assert!(matches!(
        projected[4].kind,
        EventKind::ActorEliminated { .. }
    ));
    assert!(matches!(projected[5].kind, EventKind::RoundEnded { .. }));
    let schema = event_schema();
    assert!(
        projected
            .iter()
            .all(|event| schema.is_valid(&serde_json::to_value(event).unwrap()))
    );
}

#[test]
fn three_consecutive_reports_are_accepted_and_indices_restart_per_tick() {
    let mut state = state_at(CellId::new(2, 5), CellId::new(12, 10));
    let first = state.step(&[place_one()]).unwrap();
    let second = state
        .step(&[SeatInput {
            seat: SeatId::Two,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    let third = state.step(&[]).unwrap();

    let events = project_events(&[first, second, third]).unwrap();

    assert_eq!(
        events
            .iter()
            .map(|event| event.tick.get())
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(
        events.iter().map(|event| event.index).collect::<Vec<_>>(),
        vec![0, 0]
    );
}

#[test]
fn rejects_nonconsecutive_or_oversized_report_windows() {
    let mut state = state_at(CellId::new(2, 5), CellId::new(12, 10));
    let reports = [
        state.step(&[]).unwrap(),
        state.step(&[]).unwrap(),
        state.step(&[]).unwrap(),
        state.step(&[]).unwrap(),
    ];
    assert_eq!(
        project_events(&reports),
        Err(ProjectionError::InvalidReportWindow)
    );

    let mut gap = [reports[0].clone(), reports[1].clone()];
    gap[1].tick = Tick(3);
    assert_eq!(
        project_events(&gap),
        Err(ProjectionError::InvalidReportWindow)
    );

    let mut too_many = chain_report();
    too_many.events = vec![too_many.events[0].clone(); 65];
    assert_eq!(
        project_events(&[too_many]),
        Err(ProjectionError::InvalidReportWindow)
    );

    let mut unsafe_tick = reports[0].clone();
    unsafe_tick.tick = Tick(super::MAX_SAFE_TICK + 1);
    assert_eq!(
        project_events(&[unsafe_tick]),
        Err(ProjectionError::InvalidTick)
    );
}

#[test]
fn rejects_mismatched_ticks_and_invalid_embedded_bombs() {
    let original = chain_report();

    let mut mismatched = original.clone();
    mismatched.events[0].tick = Tick(2);
    assert_eq!(
        project_events(&[mismatched]),
        Err(ProjectionError::InvalidEvent)
    );

    let mut wrong_placement_tick = original.clone();
    let crate::game::StepEventKind::BombPlaced(ref mut bomb) = wrong_placement_tick.events[0].kind
    else {
        panic!("expected placement");
    };
    bomb.placed_at = Tick(0);
    assert_eq!(
        project_events(&[wrong_placement_tick]),
        Err(ProjectionError::InvalidEvent)
    );

    let mut future_detonation = original.clone();
    let crate::game::StepEventKind::BombDetonated(ref mut bomb) = future_detonation.events[1].kind
    else {
        panic!("expected detonation");
    };
    bomb.placed_at = Tick(2);
    bomb.detonates_at = Tick(2);
    assert_eq!(
        project_events(&[future_detonation]),
        Err(ProjectionError::InvalidEvent)
    );

    let mut invalid_range = original;
    let crate::game::StepEventKind::BombDetonated(ref mut bomb) = invalid_range.events[1].kind
    else {
        panic!("expected detonation");
    };
    bomb.range = 0;
    assert_eq!(
        project_events(&[invalid_range]),
        Err(ProjectionError::InvalidEvent)
    );
}

#[test]
fn projects_real_version_two_pickup_collection() {
    let target = CellId::new(5, 5);
    let world = World::with_pickups(
        open_world().arena().clone(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(target, PickupKind::BombCapacity)]),
    )
    .unwrap();
    let actor = |id, seat, cell| ActorInit {
        id: EntityId(id),
        seat,
        position: Position::center(cell),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    };
    let mut state = State::new(
        RulesConfig::classic_v2(),
        world,
        [
            actor(10, SeatId::One, target),
            actor(20, SeatId::Two, CellId::new(12, 10)),
        ],
    )
    .unwrap();

    let report = state.step(&[]).unwrap();

    assert!(matches!(
        report.events[0].kind,
        crate::game::StepEventKind::PickupCollected { .. }
    ));
    let projected = project_events(&[report]).unwrap();
    assert!(matches!(projected[0].kind, EventKind::PickupCollected {
        id, seat: Seat::One, cell, kind: super::PickupKind::BombCapacity,
    } if id.get() == 10 && cell.column() == 5 && cell.row() == 5));
}

use std::collections::BTreeSet;

use super::spike_pit_state_tests::{PIT, SAFE, actors, arena, enabled, rules, state_at, step_to};
use super::State;
use crate::game::types::{HALF_FOOTPRINT, UNITS_PER_CELL};
use crate::game::{
    CellId, Direction, EffectKind, EnabledItems, Equipment, InputAction, PickupKind, Position,
    RoundAssignment, SeatId, SeatInput, SimulationSeed, Tick, World,
};

fn press_east(seat: SeatId) -> SeatInput {
    SeatInput {
        seat,
        action: InputAction::Press(Direction::East),
    }
}

#[test]
fn raised_pit_uses_the_actual_movement_trace() {
    let mut game = state_at(Tick(0), EnabledItems::EMPTY, enabled(), PIT, SAFE);
    game.step(&[]).unwrap();
    let tangent = Position {
        x: i32::from(PIT.column) * UNITS_PER_CELL - HALF_FOOTPRINT,
        y: Position::center(PIT).y,
    };
    game.set_test_actor_pose(SeatId::One, tangent, Direction::East);
    step_to(&mut game, 6);

    let report = game.step(&[press_east(SeatId::One)]).unwrap();
    assert!(report.traces[0].overlaps_cell(PIT));
    assert!(!game.actors()[0].alive());
}

#[test]
fn bracelet_forced_segment_contacts_a_raised_pit() {
    let forced_pit = CellId::new(3, 4);
    let mut game = state_at(
        Tick(0),
        EnabledItems::BRACELET,
        enabled(),
        SAFE,
        forced_pit,
    );
    game.set_test_spike_pit_cell(0, forced_pit);
    game.replace_test_actor_equipment(SeatId::One, Equipment::Bracelet);
    game.step(&[]).unwrap();

    let victim = Position { x: 2_687, y: 4_500 };
    let pusher = Position {
        x: victim.x - 2 * HALF_FOOTPRINT,
        y: victim.y,
    };
    game.set_test_actor_pose(SeatId::One, pusher, Direction::East);
    game.set_test_actor_pose(SeatId::Two, victim, Direction::East);
    step_to(&mut game, 6);

    let report = game.step(&[press_east(SeatId::One)]).unwrap();
    assert_eq!(report.forced_movements.len(), 1);
    assert!(report.forced_movements[0].segment.overlaps_cell(forced_pit));
    assert!(!game.actors()[1].alive());
}

#[test]
fn teleport_contacts_only_the_two_destinations() {
    let items = EnabledItems::from_bits(
        EnabledItems::TELEPORTER.bits() | EnabledItems::SHIELD.bits(),
    )
    .unwrap();
    let mut arrival = State::new_at_tick_with_assignment_and_seed(
        rules(items, enabled()),
        World::new(arena(BTreeSet::new())),
        actors(SAFE, PIT),
        Tick(0),
        RoundAssignment::first(),
        SimulationSeed::new([19; 32]),
    )
    .unwrap();
    arrival.apply_test_effect(SeatId::Two, EffectKind::Shield, Tick(1));
    arrival.step(&[]).unwrap();
    step_to(&mut arrival, 6);
    arrival.insert_test_pickup(SAFE, PickupKind::Teleporter);
    arrival.step(&[]).unwrap();
    assert_eq!(arrival.actors()[0].position(), Position::center(PIT));
    assert!(!arrival.actors()[0].alive(), "raised destination damages arrival");
    assert!(arrival.actors()[1].alive());

    let mut no_path = State::new_at_tick_with_assignment_and_seed(
        rules(EnabledItems::TELEPORTER, enabled()),
        World::new(arena(BTreeSet::new())),
        actors(PIT, SAFE),
        Tick(0),
        RoundAssignment::first(),
        SimulationSeed::new([23; 32]),
    )
    .unwrap();
    no_path.step(&[]).unwrap();
    let west = CellId::new(PIT.column - 1, PIT.row);
    let east = CellId::new(PIT.column + 1, PIT.row);
    no_path.set_test_actor_pose(SeatId::One, Position::center(west), Direction::East);
    no_path.set_test_actor_pose(SeatId::Two, Position::center(east), Direction::West);
    step_to(&mut no_path, 6);
    no_path.insert_test_pickup(west, PickupKind::Teleporter);
    no_path.step(&[]).unwrap();
    assert!(no_path.actors().iter().all(|actor| actor.alive()));
    assert_eq!(no_path.actors()[0].position(), Position::center(east));
    assert_eq!(no_path.actors()[1].position(), Position::center(west));
}

#[test]
fn stopwatch_does_not_pause_pits_and_frozen_occupants_are_hit() {
    let mut game = state_at(Tick(0), EnabledItems::STOPWATCH, enabled(), PIT, SAFE);
    game.step(&[]).unwrap();
    step_to(&mut game, 6);
    game.apply_test_effect(SeatId::Two, EffectKind::Stopwatch, Tick(6));
    assert!(game.voluntary_actions_frozen()[0]);

    game.step(&[press_east(SeatId::One)]).unwrap();
    assert!(!game.actors()[0].alive());
}


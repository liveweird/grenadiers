use crate::cpu::{CpuRng, MovementIntent, ObservedActor, ObservedPickup, WorkBudget};
use crate::game::types::TICKS_PER_SECOND;
use crate::game::{
    ActorInit, CellId, Direction, EntityId, InputAction, PickupKind, Position, RulesConfig, SeatId,
    SeatInput, SpeedLevel, State, World, fixed_arena,
};
use crate::room::RoomSettings;

use super::movement::{bounded_alignment_hold_ticks, hold_ticks};
use super::placement::{TurnAlignment, ordinary_turn_alignment};
use super::tests::{key, observation};
use super::{UtilityMemory, plan};

fn actor_mut(parts: &mut crate::cpu::ObservationParts) -> &mut crate::cpu::VisibleActor {
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor
}

fn visible_actor(actor: &crate::game::Actor) -> crate::cpu::VisibleActor {
    crate::cpu::VisibleActor {
        id: actor.id(),
        seat: actor.seat(),
        position: actor.position(),
        heading: actor.heading(),
        speed: actor.speed(),
        alive: actor.alive(),
        bomb_capacity: actor.bomb_capacity(),
        bomb_range: actor.bomb_range(),
        live_bomb_count: 0,
        equipment: None,
        weapon: None,
        effects: Vec::new(),
        damage_grace_remaining_ticks: 0,
        voluntary_actions_frozen: false,
        last_teleport_tick: None,
        unknown: crate::cpu::UnknownActorFacts::ALL,
    }
}

fn run_pulse(state: &mut State, direction: Direction, ticks: u8) {
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(direction),
        }])
        .unwrap();
    for _ in 1..ticks {
        state.step(&[]).unwrap();
    }
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Clear,
        }])
        .unwrap();
    for _ in 0..10 {
        state.step(&[]).unwrap();
    }
}

fn plan_with(
    parts: crate::cpu::ObservationParts,
    memory: UtilityMemory,
) -> (super::ControlPlan, u64) {
    let observation = crate::cpu::Observation::new(parts).unwrap();
    let mut rng = CpuRng::from_seed(key(), [41; 32]);
    let plan = plan(&observation, &memory, &mut rng, &mut WorkBudget::new(100)).unwrap();
    (plan, rng.draws())
}

#[test]
fn retained_reference_poses_align_and_preserve_the_goal() {
    let cases = [
        (
            Position {
                x: 12_174,
                y: 11_776,
            },
            Direction::West,
        ),
        (
            Position {
                x: 13_478,
                y: 11_776,
            },
            Direction::East,
        ),
    ];
    for (position, correction) in cases {
        let current = position.cell().unwrap();
        let next = CellId::new(current.column, current.row - 1);
        let mut parts = observation(current).into_parts();
        let actor = actor_mut(&mut parts);
        actor.position = position;
        actor.heading = correction;
        let memory = UtilityMemory::from_route(key(), &[next], false);
        let (planned, draws) = plan_with(parts, memory);

        assert_eq!(
            planned.intent.movement,
            MovementIntent::Direction(correction)
        );
        assert_eq!(planned.intent.manual, None);
        assert_eq!(planned.next_memory, memory);
        assert_eq!(draws, 0);
    }
}

#[test]
fn fresh_reference_poses_align_without_replacing_the_selected_goal() {
    for (position, heading, correction) in [
        (
            Position {
                x: 12_174,
                y: 11_776,
            },
            Direction::West,
            Direction::West,
        ),
        (
            Position {
                x: 13_478,
                y: 11_776,
            },
            Direction::East,
            Direction::East,
        ),
    ] {
        let current = position.cell().unwrap();
        let next = CellId::new(current.column, current.row - 1);
        let mut parts = observation(current).into_parts();
        let actor = actor_mut(&mut parts);
        actor.position = position;
        actor.heading = heading;
        parts.pickups.push(ObservedPickup {
            cell: next,
            kind: PickupKind::BombCapacity,
        });
        let (planned, draws) = plan_with(parts, UtilityMemory::new(key()));

        assert_eq!(
            planned.intent.movement,
            MovementIntent::Direction(correction)
        );
        assert_eq!(planned.intent.manual, None);
        assert_eq!(planned.next_memory.route()[0], Some(next));
        assert_eq!(draws, 0);
    }
}

#[test]
fn supported_speeds_tolerances_offsets_and_residuals_end_in_center_window() {
    let cell = CellId::new(7, 7);
    let center = Position::center(cell);
    for speed in [
        SpeedLevel::Base,
        SpeedLevel::One,
        SpeedLevel::Two,
        SpeedLevel::Three,
        SpeedLevel::Four,
    ] {
        for tolerance in [64_u32, 128, 192, 256] {
            for heading in [
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ] {
                for signed_offset in [-(tolerance as i32) - 1, -511, tolerance as i32 + 1, 511] {
                    let mut parts = observation(cell).into_parts();
                    let actor = actor_mut(&mut parts);
                    actor.speed = speed;
                    actor.heading = heading;
                    let desired = match heading {
                        Direction::North | Direction::South => Direction::East,
                        Direction::East | Direction::West => Direction::North,
                    };
                    let coordinate = match heading {
                        Direction::North | Direction::South => &mut actor.position.y,
                        Direction::East | Direction::West => &mut actor.position.x,
                    };
                    *coordinate += signed_offset;
                    let TurnAlignment::Correct(correction) =
                        ordinary_turn_alignment(actor, cell, desired, tolerance)
                    else {
                        panic!("outside tolerance must align");
                    };
                    let ticks = hold_ticks(actor, cell, correction);
                    assert_eq!(
                        bounded_alignment_hold_ticks(actor, cell, correction, tolerance),
                        Some(ticks)
                    );
                    for residual in 0..u32::from(TICKS_PER_SECOND) {
                        let travel = (residual
                            + u32::from(ticks) * u32::from(speed.units_per_second()))
                            / u32::from(TICKS_PER_SECOND);
                        let endpoint = if signed_offset.is_negative() {
                            signed_offset + i32::try_from(travel).unwrap()
                        } else {
                            signed_offset - i32::try_from(travel).unwrap()
                        };
                        let endpoint_offset = endpoint.unsigned_abs();
                        assert!(endpoint_offset <= tolerance.min(64));
                        assert!(endpoint_offset < signed_offset.unsigned_abs());
                        let endpoint_coordinate = match heading {
                            Direction::North | Direction::South => center.y + endpoint,
                            Direction::East | Direction::West => center.x + endpoint,
                        };
                        let expected_cell_coordinate = match heading {
                            Direction::North | Direction::South => cell.row,
                            Direction::East | Direction::West => cell.column,
                        };
                        assert_eq!(
                            endpoint_coordinate.div_euclid(1_024),
                            i32::from(expected_cell_coordinate)
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn supported_alignment_pulses_stop_then_turn_in_authoritative_game() {
    for speed in [
        SpeedLevel::Base,
        SpeedLevel::One,
        SpeedLevel::Two,
        SpeedLevel::Three,
        SpeedLevel::Four,
    ] {
        for tolerance in [64, 128, 192, 256] {
            let settings = RoomSettings::new(speed, tolerance, 3, 180).unwrap();
            let arena = fixed_arena();
            let spawns = arena.spawns();
            let cell = spawns[0];
            let mut position = Position::center(cell);
            position.x += tolerance + 1;
            let actors = [
                ActorInit {
                    id: EntityId(1),
                    seat: SeatId::One,
                    position,
                    heading: Direction::East,
                    speed,
                },
                ActorInit {
                    id: EntityId(2),
                    seat: SeatId::Two,
                    position: Position::center(spawns[1]),
                    heading: Direction::West,
                    speed,
                },
            ];
            let mut state = State::new(
                RulesConfig::configurable_v4(settings),
                World::new(arena),
                actors,
            )
            .unwrap();
            let actor = visible_actor(&state.actors()[0]);
            assert_eq!(
                ordinary_turn_alignment(&actor, cell, Direction::North, tolerance as u32),
                TurnAlignment::Correct(Direction::West)
            );
            let ticks =
                bounded_alignment_hold_ticks(&actor, cell, Direction::West, tolerance as u32)
                    .unwrap();
            run_pulse(&mut state, Direction::West, ticks);

            let stopped = visible_actor(&state.actors()[0]);
            assert!(stopped.position.x.abs_diff(Position::center(cell).x) <= 64);
            assert_eq!(
                ordinary_turn_alignment(&stopped, cell, Direction::North, tolerance as u32),
                TurnAlignment::Ready
            );
            let before_turn = stopped.position;
            state
                .step(&[SeatInput {
                    seat: SeatId::One,
                    action: InputAction::Press(Direction::North),
                }])
                .unwrap();
            state.step(&[]).unwrap();
            assert!(state.actors()[0].position().y < before_turn.y);
        }
    }
}

#[test]
fn unbounded_correction_is_rejected() {
    let cell = CellId::new(7, 7);
    let mut parts = observation(cell).into_parts();
    let actor = actor_mut(&mut parts);
    actor.position.x += 1;
    assert_eq!(
        bounded_alignment_hold_ticks(actor, cell, Direction::West, 0),
        None
    );
}

#[test]
fn same_axis_and_perpendicular_turns_within_tolerance_are_unchanged() {
    let cell = CellId::new(7, 7);
    let mut parts = observation(cell).into_parts();
    let actor = actor_mut(&mut parts);
    actor.heading = Direction::East;
    actor.position.x += 256;
    assert_eq!(
        ordinary_turn_alignment(actor, cell, Direction::West, 64),
        TurnAlignment::Ready
    );
    assert_eq!(
        ordinary_turn_alignment(actor, cell, Direction::North, 256),
        TurnAlignment::Ready
    );
    actor.position.x += 1;
    assert_eq!(
        ordinary_turn_alignment(actor, cell, Direction::North, 256),
        TurnAlignment::Correct(Direction::West)
    );
}

#[test]
fn planner_keeps_same_axis_and_in_tolerance_commands() {
    let current = CellId::new(7, 7);
    for (heading, next, x_offset, expected) in [
        (Direction::East, CellId::new(6, 7), 400, Direction::West),
        (Direction::East, CellId::new(7, 6), 256, Direction::North),
    ] {
        let mut parts = observation(current).into_parts();
        let actor = actor_mut(&mut parts);
        actor.heading = heading;
        actor.position.x += x_offset;
        let memory = UtilityMemory::from_route(key(), &[next], false);
        let (planned, draws) = plan_with(parts, memory);

        assert_eq!(planned.intent.movement, MovementIntent::Direction(expected));
        assert_eq!(planned.next_memory, memory);
        assert_eq!(draws, 0);
    }
}

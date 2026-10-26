use crate::game::{
    ActorInit, CellId, Direction, EntityId, InputAction, Position, RulesConfig, SeatId, SeatInput,
    SpeedLevel, State, World, fixed_arena,
};
use crate::room::RoomSettings;

use super::MAX_HOLD_TICKS;
use super::movement::hold_ticks;
use super::planner::alignment_direction;

fn observed_actor(actor: &crate::game::Actor) -> crate::cpu::VisibleActor {
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

#[test]
fn fastest_speed_pulse_and_maximum_idle_delay_preserve_the_turn() {
    let settings = RoomSettings::new(SpeedLevel::Four, 64, 3, 180).unwrap();
    let arena = fixed_arena();
    let spawns = arena.spawns();
    let actors = [SeatId::One, SeatId::Two].map(|seat| ActorInit {
        id: EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: Direction::East,
        speed: SpeedLevel::Four,
    });
    let mut state = State::new(
        RulesConfig::configurable_v4(settings),
        World::new(arena),
        actors,
    )
    .unwrap();
    // Seed a non-zero movement remainder, then use stationary alignment pulses
    // with the full external delay before the escape-length pulse.
    for direction in [Direction::West, Direction::East] {
        state
            .step(&[SeatInput {
                seat: SeatId::One,
                action: InputAction::Press(direction),
            }])
            .unwrap();
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
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    for _ in 1..MAX_HOLD_TICKS {
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
    let before = state.actors()[0].position();
    let target = Position::center(CellId::new(spawns[0].column + 1, spawns[0].row));
    assert!(before.x.abs_diff(target.x) <= 64);
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::South),
        }])
        .unwrap();
    assert!(state.actors()[0].position().y > before.y);
}

#[test]
fn both_axes_converge_along_current_heading_axis_without_illegal_turns() {
    let settings = RoomSettings::new(SpeedLevel::Base, 64, 3, 180).unwrap();
    let arena = fixed_arena();
    let spawns = arena.spawns();
    let mut first = Position::center(spawns[0]);
    first.x += 65;
    first.y += 65;
    let actors = [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: first,
            heading: Direction::East,
            speed: SpeedLevel::Base,
        },
        ActorInit {
            id: EntityId(2),
            seat: SeatId::Two,
            position: Position::center(spawns[1]),
            heading: Direction::West,
            speed: SpeedLevel::Base,
        },
    ];
    let mut state = State::new(
        RulesConfig::configurable_v4(settings),
        World::new(arena),
        actors,
    )
    .unwrap();
    let center = Position::center(spawns[0]);
    let mut directions = Vec::new();
    for _ in 0..4 {
        let actor = observed_actor(&state.actors()[0]);
        if actor.position.x.abs_diff(center.x) <= 64 && actor.position.y.abs_diff(center.y) <= 64 {
            break;
        }
        let direction = alignment_direction(&actor, spawns[0], 64).unwrap();
        directions.push(direction);
        let ticks = hold_ticks(&actor, spawns[0], direction);
        run_pulse(&mut state, direction, ticks);
    }
    let final_position = state.actors()[0].position();
    assert_eq!(directions, vec![Direction::West, Direction::North]);
    assert!(final_position.x.abs_diff(center.x) <= 64);
    assert!(final_position.y.abs_diff(center.y) <= 64);
}

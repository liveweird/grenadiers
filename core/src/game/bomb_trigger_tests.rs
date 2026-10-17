use super::{ActorInit, InputAction, MAX_INPUTS_PER_TICK, SeatInput, State, StateError, StepError};
use crate::game::bombs::{Bomb, BombTrigger};
use crate::game::input::Direction;
use crate::game::rules::RulesConfig;
use crate::game::test_support::add_test_bomb_with_trigger;
use crate::game::types::{CellId, EntityId, Position, SeatId, SpeedLevel, Tick};
use crate::game::{World, fixed_arena};

fn actors() -> [ActorInit; 2] {
    let arena = fixed_arena();
    let spawns = arena.spawns();
    SeatId::ALL.map(|seat| ActorInit {
        id: EntityId(seat.index() as u32 + 10),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    })
}

fn state() -> State {
    State::new(
        RulesConfig::classic_v1(),
        World::new(fixed_arena()),
        actors(),
    )
    .unwrap()
}

#[test]
fn state_construction_rejects_each_untimed_trigger() {
    for trigger in [BombTrigger::Remote, BombTrigger::Contact] {
        let mut world = World::new(fixed_arena());
        let bomb = Bomb::new(
            EntityId(30),
            CellId::new(8, 5),
            EntityId(10),
            2,
            Tick(0),
            trigger,
        )
        .unwrap();
        world.bombs.insert(bomb.id(), bomb);

        assert_eq!(
            State::new(RulesConfig::classic_v1(), world, actors()),
            Err(StateError::UnsupportedBombTrigger(EntityId(30)))
        );
    }
}

#[test]
fn step_rejects_each_untimed_trigger_before_any_mutation() {
    for trigger in [BombTrigger::Remote, BombTrigger::Contact] {
        let mut state = state();
        add_test_bomb_with_trigger(
            &mut state,
            EntityId(30),
            CellId::new(8, 5),
            EntityId(10),
            2,
            Tick(0),
            trigger,
        )
        .unwrap();
        state.world.hazards.insert(CellId::new(2, 5), Tick(1));
        let before = state.clone();

        assert_eq!(
            state.step(&[
                SeatInput {
                    seat: SeatId::One,
                    action: InputAction::Press(Direction::East),
                },
                SeatInput {
                    seat: SeatId::Two,
                    action: InputAction::PlaceBomb,
                },
            ]),
            Err(StepError::UnsupportedBombTrigger(EntityId(30)))
        );
        assert_eq!(state, before);
    }
}

#[test]
fn existing_input_and_tick_errors_precede_the_trigger_guard_atomically() {
    let mut state = state();
    add_test_bomb_with_trigger(
        &mut state,
        EntityId(30),
        CellId::new(8, 5),
        EntityId(10),
        2,
        Tick(0),
        BombTrigger::Remote,
    )
    .unwrap();
    let before_inputs = state.clone();
    let too_many = vec![
        SeatInput {
            seat: SeatId::One,
            action: InputAction::Clear,
        };
        MAX_INPUTS_PER_TICK + 1
    ];
    assert_eq!(
        state.step(&too_many),
        Err(StepError::TooManyInputs {
            count: MAX_INPUTS_PER_TICK + 1,
            maximum: MAX_INPUTS_PER_TICK,
        })
    );
    assert_eq!(state, before_inputs);

    state.tick = Tick(u64::MAX);
    let before_tick = state.clone();
    assert_eq!(state.step(&[]), Err(StepError::TickOverflow));
    assert_eq!(state, before_tick);
}

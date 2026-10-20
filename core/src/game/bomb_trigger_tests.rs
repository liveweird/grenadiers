use super::{ActorInit, InputAction, MAX_INPUTS_PER_TICK, SeatInput, State, StateError, StepError};
use crate::game::bombs::{BlastKind, Bomb, BombTrigger};
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
fn state_construction_rejects_every_special_blast_before_activation() {
    for blast in [BlastKind::Piercing, BlastKind::Power] {
        let mut world = World::new(fixed_arena());
        let bomb = Bomb::new_with_blast(
            EntityId(30),
            CellId::new(8, 5),
            EntityId(10),
            2,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(10),
            },
            blast,
        )
        .unwrap();
        world.bombs.insert(bomb.id(), bomb);

        assert_eq!(
            State::new(RulesConfig::classic_v1(), world, actors()),
            Err(StateError::UnsupportedBombBlast(EntityId(30)))
        );
    }
}

fn mixed_unsupported_bombs(
    blast_id: EntityId,
    trigger_id: EntityId,
    trigger: BombTrigger,
    blast: BlastKind,
) -> [Bomb; 2] {
    [
        Bomb::new_with_blast(
            blast_id,
            CellId::new(7, 5),
            EntityId(10),
            2,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(10),
            },
            blast,
        )
        .unwrap(),
        Bomb::new(
            trigger_id,
            CellId::new(8, 5),
            EntityId(10),
            2,
            Tick(0),
            trigger,
        )
        .unwrap(),
    ]
}

#[test]
fn state_constructor_preserves_trigger_precedence_over_blast_for_every_bomb_order() {
    for blast in [BlastKind::Piercing, BlastKind::Power] {
        for trigger in [BombTrigger::Remote, BombTrigger::Contact] {
            for (blast_id, trigger_id) in
                [(EntityId(30), EntityId(31)), (EntityId(31), EntityId(30))]
            {
                let mut world = World::new(fixed_arena());
                for bomb in mixed_unsupported_bombs(blast_id, trigger_id, trigger, blast) {
                    world.bombs.insert(bomb.id(), bomb);
                }

                assert_eq!(
                    State::new(RulesConfig::classic_v1(), world, actors()),
                    Err(StateError::UnsupportedBombTrigger(trigger_id))
                );
            }
        }
    }
}

#[test]
fn step_preserves_trigger_precedence_over_blast_for_every_bomb_order() {
    for blast in [BlastKind::Piercing, BlastKind::Power] {
        for trigger in [BombTrigger::Remote, BombTrigger::Contact] {
            for (blast_id, trigger_id) in
                [(EntityId(30), EntityId(31)), (EntityId(31), EntityId(30))]
            {
                let mut state = state();
                for bomb in mixed_unsupported_bombs(blast_id, trigger_id, trigger, blast) {
                    state.insert_bomb(bomb);
                }
                let before = state.clone();

                assert_eq!(
                    state.step(&[]),
                    Err(StepError::UnsupportedBombTrigger(trigger_id))
                );
                assert_eq!(state, before);
            }
        }
    }
}

#[test]
fn step_rejects_every_special_blast_before_any_mutation() {
    for blast in [BlastKind::Piercing, BlastKind::Power] {
        let mut state = state();
        let bomb = Bomb::new_with_blast(
            EntityId(30),
            CellId::new(8, 5),
            EntityId(10),
            2,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(10),
            },
            blast,
        )
        .unwrap();
        state.insert_bomb(bomb);
        let before = state.clone();

        assert_eq!(
            state.step(&[SeatInput {
                seat: SeatId::One,
                action: InputAction::Press(Direction::East),
            }]),
            Err(StepError::UnsupportedBombBlast(EntityId(30)))
        );
        assert_eq!(state, before);
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

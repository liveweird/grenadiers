use super::bracelet_test_support::{input, press, simple_state, state};
use super::{Direction, InputAction, Position, RoundResult, SeatId, StepError};
use crate::game::{
    BombTrigger, CellId, EffectKind, EnabledItems, EntityId, Equipment, Tick, World,
};

const ONE: Position = Position { x: 4_096, y: 4_096 };
const TWO: Position = Position { x: 4_736, y: 4_096 };

fn enabled(items: &[EnabledItems]) -> EnabledItems {
    EnabledItems::from_bits(items.iter().fold(0, |bits, item| bits | item.bits())).unwrap()
}

fn reset_contact(game: &mut super::State) {
    game.actors[0].position = ONE;
    game.actors[1].position = TWO;
}

#[test]
fn shove_cooldown_uses_physical_ticks_elapsed_while_holder_was_frozen() {
    let items = enabled(&[EnabledItems::BRACELET, EnabledItems::STOPWATCH]);
    let mut game = simple_state(items, ONE, TWO);
    assert!(
        game.step(&[press(SeatId::One, Direction::East)])
            .unwrap()
            .forced_movement
            .is_some()
    );
    assert_eq!(game.actors[0].last_successful_shove_tick(), Some(Tick(1)));

    game.apply_test_effect(SeatId::Two, EffectKind::Stopwatch, Tick(1));
    for expected_tick in 2..=30 {
        reset_contact(&mut game);
        assert!(game.step(&[]).unwrap().forced_movement.is_none());
        assert_eq!(game.tick(), Tick(expected_tick));
    }
    game.clear_test_effects(SeatId::Two);
    reset_contact(&mut game);
    assert!(game.step(&[]).unwrap().forced_movement.is_some());
    assert_eq!(game.tick(), Tick(31));
    assert_eq!(game.running_updates(), 31);
}

#[test]
fn gear_replacement_keeps_shove_history_and_punch_cooldown_is_independent() {
    let items = enabled(&[EnabledItems::BRACELET, EnabledItems::PUNCH]);
    let mut game = simple_state(items, ONE, TWO);
    game.step(&[press(SeatId::One, Direction::East)]).unwrap();
    assert_eq!(game.actors[0].last_successful_shove_tick(), Some(Tick(1)));

    game.replace_actor_equipment(SeatId::One, Equipment::Punch);
    game.insert_test_bomb(
        EntityId(30),
        CellId::new(5, 4),
        EntityId(2),
        1,
        Tick(1),
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
    )
    .unwrap();
    game.step(&[input(SeatId::One, InputAction::UseEquipment)])
        .unwrap();
    assert_eq!(game.actors[0].last_successful_punch_tick(), Some(Tick(2)));
    assert_eq!(game.actors[0].last_successful_shove_tick(), Some(Tick(1)));

    game.replace_actor_equipment(SeatId::One, Equipment::Bracelet);
    reset_contact(&mut game);
    assert!(game.step(&[]).unwrap().forced_movement.is_none());
    assert_eq!(game.actors[0].last_successful_shove_tick(), Some(Tick(1)));
}

#[test]
fn death_result_and_terminal_steps_retain_history_while_new_state_resets_it() {
    let mut game = simple_state(EnabledItems::BRACELET, ONE, TWO);
    game.step(&[press(SeatId::One, Direction::East)]).unwrap();
    game.insert_test_flame(CellId::new(4, 4), Tick(10));
    game.step(&[]).unwrap();

    assert!(!game.actors[0].alive());
    assert_eq!(game.result(), Some(RoundResult::Winner(SeatId::Two)));
    assert_eq!(game.actors[0].last_successful_shove_tick(), Some(Tick(1)));
    let frozen_updates = game.running_updates();
    game.step(&[]).unwrap();
    assert_eq!(game.tick(), Tick(3));
    assert_eq!(game.running_updates(), frozen_updates);
    assert_eq!(game.actors[0].last_successful_shove_tick(), Some(Tick(1)));

    let reset = simple_state(EnabledItems::BRACELET, ONE, TWO);
    assert_eq!(reset.actors[0].last_successful_shove_tick(), None);
    assert_eq!(reset.actors[1].last_successful_shove_tick(), None);
}

#[test]
fn malformed_history_detects_disabled_preconstruction_and_future_values() {
    let far = Position { x: 8_192, y: 8_192 };
    let mut preconstruction = state(
        EnabledItems::BRACELET,
        World::new(super::bracelet_test_support::arena(Default::default())),
        ONE,
        far,
        Tick(10),
    );
    preconstruction.actors[0].restore_shove_cooldown(Some(Tick(10)));
    assert_eq!(
        preconstruction.step(&[]),
        Err(StepError::UnsupportedShoveHistory(EntityId(1)))
    );

    preconstruction.actors[0].restore_shove_cooldown(None);
    preconstruction.step(&[]).unwrap();
    preconstruction.actors[0].restore_shove_cooldown(Some(Tick(12)));
    assert_eq!(
        preconstruction.step(&[]),
        Err(StepError::UnsupportedShoveHistory(EntityId(1)))
    );

    let mut disabled = simple_state(EnabledItems::EMPTY, ONE, far);
    disabled.step(&[]).unwrap();
    disabled.actors[0].restore_shove_cooldown(Some(Tick(1)));
    assert_eq!(
        disabled.step(&[]),
        Err(StepError::UnsupportedShoveHistory(EntityId(1)))
    );
}

#[test]
fn older_input_tick_and_bomb_validation_errors_still_precede_shove_history() {
    let far = Position { x: 8_192, y: 8_192 };
    let mut too_many = simple_state(EnabledItems::BRACELET, ONE, far);
    too_many.actors[0].restore_shove_cooldown(Some(Tick(0)));
    let inputs = [press(SeatId::One, Direction::East); super::MAX_INPUTS_PER_TICK + 1];
    assert_eq!(
        too_many.step(&inputs),
        Err(StepError::TooManyInputs {
            count: super::MAX_INPUTS_PER_TICK + 1,
            maximum: super::MAX_INPUTS_PER_TICK,
        })
    );

    let mut overflow = state(
        EnabledItems::BRACELET,
        World::new(super::bracelet_test_support::arena(Default::default())),
        ONE,
        far,
        Tick(u64::MAX),
    );
    overflow.actors[0].restore_shove_cooldown(Some(Tick(u64::MAX)));
    assert_eq!(overflow.step(&[]), Err(StepError::TickOverflow));

    let mut bomb_first = simple_state(EnabledItems::BRACELET, ONE, far);
    bomb_first.step(&[]).unwrap();
    bomb_first.actors[0].restore_shove_cooldown(Some(Tick(1)));
    bomb_first
        .insert_test_bomb(
            EntityId(30),
            CellId::new(6, 4),
            EntityId(2),
            1,
            Tick(1),
            BombTrigger::Contact,
        )
        .unwrap();
    assert_eq!(
        bomb_first.step(&[]),
        Err(StepError::UnsupportedBombTrigger(EntityId(30)))
    );
}

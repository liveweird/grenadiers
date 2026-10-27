use crate::game::test_support::add_test_bomb_with_deadline;
use crate::game::{
    ActorInit, BlastKind, CellId, Direction, EffectKind, EnabledItems, EntityId, InputAction,
    Position, RulesConfig, SeatId, SeatInput, SpecialWeapon, SpeedLevel, State, StepEventKind,
    Tick, World, fixed_arena,
};
use crate::room::RoomSettings;

fn state(enabled: EnabledItems) -> State {
    let arena = fixed_arena();
    let spawns = arena.spawns().to_vec();
    State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled)),
        World::new(arena),
        SeatId::DUEL.map(|seat| ActorInit {
            id: EntityId(seat.index() as u32 + 10),
            seat,
            position: Position::center(spawns[seat.index()]),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        }),
    )
    .unwrap()
}

fn input(seat: SeatId, action: InputAction) -> SeatInput {
    SeatInput { seat, action }
}

#[test]
fn frozen_manual_action_is_dropped_while_inputs_update_and_thaw_has_no_delayed_placement() {
    let mut game = state(EnabledItems::STOPWATCH);
    game.actors[0]
        .effects
        .apply(EffectKind::Stopwatch, Tick(0))
        .unwrap();
    let start = game.actors[1].position;
    game.step(&[
        input(SeatId::Two, InputAction::Press(Direction::West)),
        input(SeatId::Two, InputAction::PlaceBomb),
    ])
    .unwrap();
    assert_eq!(game.actors[1].position, start);
    assert!(game.world.bombs.is_empty());
    for _ in 1..180 {
        game.step(&[]).unwrap();
    }
    assert_ne!(game.actors[1].position, start);
    assert!(game.world.bombs.is_empty());
}

#[test]
fn frozen_manual_attempt_does_not_suppress_involuntary_diarrhea_or_consume_weapon() {
    let enabled = EnabledItems::from_bits(
        EnabledItems::STOPWATCH.bits()
            | EnabledItems::DIARRHEA.bits()
            | EnabledItems::POWER_BOMB.bits(),
    )
    .unwrap();
    let mut game = state(enabled);
    game.actors[0]
        .effects
        .apply(EffectKind::Stopwatch, Tick(0))
        .unwrap();
    game.actors[1]
        .effects
        .apply(EffectKind::Diarrhea, Tick(0))
        .unwrap();
    game.actors[1]
        .inventory
        .replace_weapon(SpecialWeapon::Power);
    game.step(&[input(SeatId::Two, InputAction::PlaceBomb)])
        .unwrap();
    let bomb = game.world.bombs.values().next().unwrap();
    assert_eq!(bomb.blast(), BlastKind::Standard);
    assert_eq!(
        game.actors[1].inventory.weapon(),
        Some(SpecialWeapon::Power)
    );
}

#[test]
fn fire_chain_detonates_a_paused_due_bomb_with_emission_time_pause_history() {
    let mut game = state(EnabledItems::STOPWATCH);
    game.actors[0]
        .effects
        .apply(EffectKind::Stopwatch, Tick(0))
        .unwrap();
    let cell = CellId::new(7, 5);
    add_test_bomb_with_deadline(
        &mut game,
        EntityId(30),
        cell,
        EntityId(10),
        2,
        Tick(0),
        Tick(1),
    )
    .unwrap();
    game.insert_test_flame(cell, Tick(2));
    let report = game.step(&[]).unwrap();
    assert!(game.world.bombs.is_empty());
    let event = report
        .events
        .iter()
        .find(|event| matches!(event.kind, StepEventKind::BombDetonated(_)))
        .unwrap();
    let timing = event.timed_bomb.unwrap();
    assert!(timing.paused);
    assert_eq!(timing.history.paused_ticks, 1);
    assert_eq!(timing.detonates_at, Tick(2));
}

#[test]
fn holder_death_does_not_rewrite_the_fixed_update_context() {
    let mut game = state(EnabledItems::STOPWATCH);
    game.actors[0]
        .effects
        .apply(EffectKind::Stopwatch, Tick(0))
        .unwrap();
    let holder_cell = game.actors[0].position.cell().unwrap();
    let victim_start = game.actors[1].position;
    game.insert_test_flame(holder_cell, Tick(2));
    game.step(&[input(SeatId::Two, InputAction::Press(Direction::North))])
        .unwrap();
    assert!(!game.actors[0].alive());
    assert_eq!(game.actors[1].position, victim_start);
    assert!(!game.voluntary_actions_frozen()[1]);
}

use crate::game::test_support::add_test_bomb_with_deadline;
use std::collections::BTreeMap;

use crate::game::{
    ActorInit, CellId, Direction, EffectKind, EnabledItems, EntityId, InputAction, PickupKind,
    Position, RulesConfig, SeatId, SeatInput, SpeedLevel, State, StepError, Tick, World,
    fixed_arena,
};
use crate::room::RoomSettings;

fn state(at: Tick) -> State {
    let arena = fixed_arena();
    let spawns = arena.spawns();
    State::new_at_tick(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::STOPWATCH)),
        World::new(arena),
        SeatId::ALL.map(|seat| ActorInit {
            id: EntityId(seat.index() as u32 + 10),
            seat,
            position: Position::center(spawns[seat.index()]),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        }),
        at,
    )
    .unwrap()
}

fn input(seat: SeatId, action: InputAction) -> SeatInput {
    SeatInput { seat, action }
}

#[test]
fn stopwatch_freezes_opponent_and_pauses_preexisting_timed_bombs_until_exact_expiry() {
    let mut game = state(Tick(0));
    game.actors[0]
        .effects
        .apply(EffectKind::Stopwatch, Tick(0))
        .unwrap();
    add_test_bomb_with_deadline(
        &mut game,
        EntityId(30),
        CellId::new(7, 5),
        EntityId(10),
        2,
        Tick(0),
        Tick(180),
    )
    .unwrap();
    let frozen_start = game.actors[1].position;
    game.step(&[input(SeatId::Two, InputAction::Press(Direction::West))])
        .unwrap();
    assert_eq!(game.actors[1].position, frozen_start);
    let bomb = *game.world.bombs.get(&EntityId(30)).unwrap();
    assert_eq!(bomb.detonates_at(), Some(Tick(181)));
    assert_eq!(bomb.timed_history().unwrap().paused_ticks, 1);

    for _ in 1..179 {
        game.step(&[]).unwrap();
    }
    assert_eq!(game.tick(), Tick(179));
    assert_eq!(game.actors[1].position, frozen_start);
    game.step(&[]).unwrap();
    assert_eq!(game.tick(), Tick(180));
    assert_ne!(game.actors[1].position, frozen_start);
    let bomb = *game.world.bombs.get(&EntityId(30)).unwrap();
    assert_eq!(bomb.detonates_at(), Some(Tick(359)));
    assert_eq!(bomb.timed_history().unwrap().paused_ticks, 179);
}

#[test]
fn newly_placed_bomb_records_emission_pause_without_same_tick_arithmetic() {
    let mut game = state(Tick(0));
    game.actors[0]
        .effects
        .apply(EffectKind::Stopwatch, Tick(0))
        .unwrap();
    let report = game
        .step(&[input(SeatId::One, InputAction::PlaceBomb)])
        .unwrap();
    let bomb = *game.world.bombs.values().next().unwrap();
    assert_eq!(bomb.placed_at(), Tick(1));
    assert_eq!(bomb.detonates_at(), Some(Tick(181)));
    assert_eq!(bomb.timed_history().unwrap().paused_ticks, 0);
    let timing = report.events[0].timed_bomb.unwrap();
    assert!(timing.paused);
    assert_eq!(timing.history.paused_ticks, 0);
    assert_eq!(timing.detonates_at, Tick(181));
}

#[test]
fn overlapping_stopwatches_pause_once_and_overflow_preflight_changes_nothing() {
    let mut game = state(Tick(0));
    for actor in &mut game.actors {
        actor.effects.apply(EffectKind::Stopwatch, Tick(0)).unwrap();
    }
    add_test_bomb_with_deadline(
        &mut game,
        EntityId(30),
        CellId::new(7, 5),
        EntityId(10),
        2,
        Tick(0),
        Tick(180),
    )
    .unwrap();
    game.step(&[]).unwrap();
    assert_eq!(
        game.world.bombs[&EntityId(30)]
            .timed_history()
            .unwrap()
            .paused_ticks,
        1
    );

    let mut overflow = state(Tick(0));
    overflow.actors[0]
        .effects
        .apply(EffectKind::Stopwatch, Tick(0))
        .unwrap();
    add_test_bomb_with_deadline(
        &mut overflow,
        EntityId(40),
        CellId::new(7, 5),
        EntityId(10),
        2,
        Tick(0),
        Tick(u64::MAX),
    )
    .unwrap();
    let before = overflow.clone();
    assert_eq!(
        overflow.step(&[input(SeatId::Two, InputAction::Clear)]),
        Err(StepError::BombPauseOverflow(EntityId(40)))
    );
    assert_eq!(overflow, before);
}

#[test]
fn refresh_and_multiple_pause_periods_keep_one_physical_deadline_history() {
    let mut game = state(Tick(0));
    game.actors[0]
        .effects
        .apply(EffectKind::Stopwatch, Tick(0))
        .unwrap();
    add_test_bomb_with_deadline(
        &mut game,
        EntityId(30),
        CellId::new(7, 5),
        EntityId(10),
        2,
        Tick(0),
        Tick(180),
    )
    .unwrap();
    game.step(&[]).unwrap();
    game.actors[0].effects.clear();
    game.step(&[]).unwrap();
    game.actors[0]
        .effects
        .apply(EffectKind::Stopwatch, game.tick())
        .unwrap();
    game.step(&[]).unwrap();
    let bomb = game.world.bombs[&EntityId(30)];
    assert_eq!(bomb.timed_history().unwrap().paused_ticks, 2);
    assert_eq!(bomb.detonates_at(), Some(Tick(182)));

    game.actors[0]
        .effects
        .apply(EffectKind::Stopwatch, Tick(100))
        .unwrap();
    assert!(
        game.actors[0]
            .effects
            .active(EffectKind::Stopwatch, Tick(279))
    );
    assert!(
        !game.actors[0]
            .effects
            .active(EffectKind::Stopwatch, Tick(280))
    );
}

#[test]
fn late_stopwatch_pickup_failure_rolls_back_pause_input_and_world_mutations() {
    let at = Tick(u64::MAX - EffectKind::Stopwatch.duration_ticks());
    let arena = fixed_arena();
    let spawns = arena.spawns();
    let pickup_cell = spawns[1];
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(pickup_cell, PickupKind::Stopwatch)]),
    )
    .unwrap();
    let mut game = State::new_at_tick(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::STOPWATCH)),
        world,
        SeatId::ALL.map(|seat| ActorInit {
            id: EntityId(seat.index() as u32 + 10),
            seat,
            position: Position::center(spawns[seat.index()]),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        }),
        at,
    )
    .unwrap();
    game.actors[0]
        .effects
        .apply(EffectKind::Stopwatch, at)
        .unwrap();
    add_test_bomb_with_deadline(
        &mut game,
        EntityId(30),
        CellId::new(7, 5),
        EntityId(10),
        2,
        Tick(at.0 - 1),
        Tick(u64::MAX - 1),
    )
    .unwrap();
    let before = game.clone();
    assert_eq!(
        game.step(&[
            input(SeatId::Two, InputAction::Press(Direction::North)),
            input(SeatId::Two, InputAction::PlaceBomb),
        ]),
        Err(StepError::EffectExpiryOverflow(EffectKind::Stopwatch))
    );
    assert_eq!(game, before);
}

#[test]
fn timed_history_captures_base_half_double_and_combined_fuse_choices_once() {
    let cases = [
        (None, 180),
        (Some(EffectKind::ShortFuse), 90),
        (Some(EffectKind::LongFuse), 360),
    ];
    for (effect, expected) in cases {
        let enabled = EnabledItems::from_bits(
            EnabledItems::STOPWATCH.bits()
                | EnabledItems::SHORT_FUSE.bits()
                | EnabledItems::LONG_FUSE.bits(),
        )
        .unwrap();
        let arena = fixed_arena();
        let spawns = arena.spawns();
        let mut game = State::new(
            RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled)),
            World::new(arena),
            SeatId::ALL.map(|seat| ActorInit {
                id: EntityId(seat.index() as u32 + 10),
                seat,
                position: Position::center(spawns[seat.index()]),
                heading: Direction::East,
                speed: SpeedLevel::Base,
            }),
        )
        .unwrap();
        if let Some(effect) = effect {
            game.actors[0].effects.apply(effect, Tick(0)).unwrap();
        }
        game.step(&[input(SeatId::One, InputAction::PlaceBomb)])
            .unwrap();
        assert_eq!(
            game.world
                .bombs
                .values()
                .next()
                .unwrap()
                .timed_history()
                .unwrap()
                .captured_fuse_ticks,
            expected
        );
    }

    let mut combined = state(Tick(0));
    combined.actors[0]
        .effects
        .apply(EffectKind::ShortFuse, Tick(0))
        .unwrap();
    combined.actors[0]
        .effects
        .apply(EffectKind::LongFuse, Tick(0))
        .unwrap();
    combined
        .step(&[input(SeatId::One, InputAction::PlaceBomb)])
        .unwrap();
    assert_eq!(
        combined
            .world
            .bombs
            .values()
            .next()
            .unwrap()
            .timed_history()
            .unwrap()
            .captured_fuse_ticks,
        180
    );
}

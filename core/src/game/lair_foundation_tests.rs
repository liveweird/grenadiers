use super::{ActorInit, State};
use crate::game::{
    BombTrigger, CellId, Direction, EffectKind, EnabledItems, EntityId, Equipment, Position,
    RoundResult, RulesConfig, SeatId, SpeedLevel, StepError, StormCloudConfig, Tick, World,
    fixed_arena,
};
use crate::room::{LairConfig, LairSettings, RoomSettings};

fn state(config: LairConfig) -> State {
    state_with_items(config, EnabledItems::EMPTY)
}

fn state_with_items(config: LairConfig, items: EnabledItems) -> State {
    let arena = fixed_arena();
    let actors = SeatId::DUEL.map(|seat| ActorInit {
        id: EntityId(10 + seat.index() as u32),
        seat,
        position: Position::center(arena.spawns()[seat.index()]),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    });
    State::new(
        RulesConfig::environment_v8(
            RoomSettings::DEFAULT.with_enabled_items(items),
            StormCloudConfig::Disabled,
            config,
        ),
        World::new(arena),
        actors,
    )
    .unwrap()
}

#[test]
fn mobs_move_exactly_one_and_a_half_cells_per_second() {
    let mut game = state(LairConfig::Enabled(LairSettings::new(5, 3).unwrap()));
    clear_soft_blocks(&mut game);
    step_to(&mut game, 300);
    let before = game.mobs()[0].position;
    step_to(&mut game, 340);
    let after = game.mobs()[0].position;
    assert_eq!(
        i64::from(after.x).abs_diff(i64::from(before.x))
            + i64::from(after.y).abs_diff(i64::from(before.y)),
        1_024
    );
}

#[test]
fn stopwatch_freezes_existing_and_new_mobs_without_pausing_lair_timers() {
    let mut game = state(LairConfig::Enabled(LairSettings::new(5, 3).unwrap()));
    step_to(&mut game, 299);
    game.apply_test_effect(SeatId::One, EffectKind::Stopwatch, Tick(299));
    game.step(&[]).unwrap();
    let before = game.mobs();
    assert_eq!(before.len(), 2);
    assert!(before.iter().all(|mob| mob.frozen));
    assert_eq!(game.lairs().unwrap().map(|lair| lair.next_spawn_at), [Tick(600); 2]);
    game.step(&[]).unwrap();
    let after = game.mobs();
    assert_eq!(
        after.iter().map(|mob| mob.position).collect::<Vec<_>>(),
        before.iter().map(|mob| mob.position).collect::<Vec<_>>()
    );
}

#[test]
fn contact_uses_heart_grace_and_flames_remove_mobs() {
    let mut game = state(LairConfig::Enabled(LairSettings::new(5, 3).unwrap()));
    step_to(&mut game, 300);
    let mob_cell = game.mobs()[0].position.cell().unwrap();
    game.set_test_actor_pose(SeatId::One, game.mobs()[0].position, Direction::East);
    game.replace_test_actor_equipment(SeatId::One, Equipment::Heart);
    game.apply_test_effect(SeatId::One, EffectKind::Shield, Tick(300));
    game.step(&[]).unwrap();
    assert!(game.actor(SeatId::One).unwrap().alive());
    assert_eq!(
        game.actor(SeatId::One).unwrap().inventory().equipment(),
        Some(Equipment::Heart)
    );
    game.clear_test_effects(SeatId::One);
    game.step(&[]).unwrap();
    assert_eq!(game.actor(SeatId::One).unwrap().inventory().equipment(), None);
    game.step(&[]).unwrap();
    assert!(game.actor(SeatId::One).unwrap().alive());

    game.insert_test_flame(mob_cell, Tick(400));
    game.step(&[]).unwrap();
    assert_eq!(game.mobs().len(), 1);
}

#[test]
fn moving_mob_triggers_a_contact_mine_and_dies_in_its_blast() {
    let mut game = state_with_items(
        LairConfig::Enabled(LairSettings::new(5, 3).unwrap()),
        EnabledItems::LANDMINE,
    );
    clear_soft_blocks(&mut game);
    step_to(&mut game, 300);
    game.step(&[]).unwrap();
    let cell = game.mobs()[0].position.cell().unwrap();
    game.insert_test_bomb(
        EntityId(90),
        cell,
        EntityId(10),
        1,
        Tick(301),
        BombTrigger::Contact,
    )
    .unwrap();
    game.step(&[]).unwrap();
    assert!(!game.world().bombs().contains_key(&EntityId(90)));
    assert_eq!(game.mobs().len(), 1);
}

#[test]
fn bomb_danger_reverses_a_moving_mob_without_a_bomb_action() {
    let mut game = state(LairConfig::Enabled(LairSettings::new(5, 3).unwrap()));
    clear_soft_blocks(&mut game);
    step_to(&mut game, 300);
    let spawned = game.mobs()[0].position;
    game.step(&[]).unwrap();
    let approaching = game.mobs()[0].position;
    let (ahead, dx, dy) = if approaching.x < spawned.x {
        (CellId::new(5, 5), approaching.x - spawned.x, 0)
    } else if approaching.x > spawned.x {
        (CellId::new(7, 5), approaching.x - spawned.x, 0)
    } else if approaching.y < spawned.y {
        (CellId::new(6, 4), 0, approaching.y - spawned.y)
    } else {
        (CellId::new(6, 6), 0, approaching.y - spawned.y)
    };
    game.insert_test_bomb(
        EntityId(90),
        ahead,
        EntityId(10),
        1,
        Tick(301),
        BombTrigger::Timed {
            detonates_at: Tick(10_000),
        },
    )
    .unwrap();
    game.step(&[]).unwrap();
    let after = game.mobs()[0].position;
    assert!((after.x - approaching.x) * dx + (after.y - approaching.y) * dy < 0);
    assert_eq!(after, spawned, "the reversal clamps at the prior center");
    game.step(&[]).unwrap();
    assert_ne!(game.mobs()[0].position, spawned, "escape does not oscillate");
    assert_eq!(game.world().bombs().len(), 1);
}

#[test]
fn death_releases_only_its_lairs_capacity() {
    let mut game = state(LairConfig::Enabled(LairSettings::new(5, 1).unwrap()));
    step_to(&mut game, 300);
    let killed = game.mobs()[0].position.cell().unwrap();
    game.insert_test_flame(killed, Tick(400));
    game.step(&[]).unwrap();
    assert_eq!(game.lairs().unwrap().map(|lair| lair.living_mobs), [0, 1]);
    step_to(&mut game, 600);
    assert_eq!(game.mobs().len(), 2);
    assert_eq!(game.lairs().unwrap().map(|lair| lair.living_mobs), [1, 1]);
}

#[test]
fn terminal_state_and_preflight_failures_leave_the_overlay_atomic() {
    let mut terminal = state(LairConfig::Enabled(LairSettings::new(5, 3).unwrap()));
    step_to(&mut terminal, 300);
    terminal.result = Some(RoundResult::Winner(SeatId::One));
    let frozen = terminal.clone();
    terminal.step(&[]).unwrap();
    assert_eq!(terminal.mobs(), frozen.mobs());
    assert_eq!(terminal.lairs(), frozen.lairs());

    let mut timer_overflow = state(LairConfig::Enabled(LairSettings::new(5, 3).unwrap()));
    timer_overflow.tick = Tick(u64::MAX - 1);
    let before = timer_overflow.clone();
    assert_eq!(
        timer_overflow.step(&[]),
        Err(StepError::LairTimerOverflow)
    );
    assert_eq!(timer_overflow, before);

    let mut overflow = state(LairConfig::Enabled(LairSettings::new(5, 3).unwrap()));
    step_to(&mut overflow, 299);
    overflow.next_entity_id = u64::from(u32::MAX) + 1;
    let before = overflow.clone();
    assert_eq!(overflow.step(&[]), Err(StepError::EntityIdOverflow));
    assert_eq!(overflow, before);
}

fn step_to(state: &mut State, tick: u64) {
    while state.tick().0 < tick {
        state.step(&[]).unwrap();
    }
}

fn clear_soft_blocks(state: &mut State) {
    let blocks = state.world.arena.blocks().clone();
    state.world.arena.remove_blocks(&blocks);
}

#[test]
fn exact_interval_spawns_once_per_lair_with_stable_ids_and_layout() {
    let mut game = state(LairConfig::default());
    assert_eq!(
        game.lairs().unwrap().map(|lair| lair.cell),
        [CellId::new(6, 6), CellId::new(8, 6)]
    );
    step_to(&mut game, 599);
    assert!(game.mobs().is_empty());
    game.step(&[]).unwrap();
    let mobs = game.mobs();
    assert_eq!(mobs.len(), 2);
    assert_eq!(mobs.iter().map(|mob| mob.id).collect::<Vec<_>>(), [EntityId(0), EntityId(1)]);
    assert_eq!(game.lairs().unwrap().map(|lair| lair.next_spawn_at), [Tick(1_200); 2]);
}

#[test]
fn blocked_attempt_is_skipped_without_a_burst() {
    let mut game = state(LairConfig::Enabled(LairSettings::new(5, 3).unwrap()));
    let blocked = [
        CellId::new(6, 5),
        CellId::new(7, 6),
        CellId::new(6, 7),
        CellId::new(5, 6),
        CellId::new(8, 5),
        CellId::new(9, 6),
        CellId::new(8, 7),
    ];
    for (offset, cell) in blocked.into_iter().enumerate() {
        game.insert_test_bomb(
            EntityId(100 + offset as u32),
            cell,
            EntityId(10),
            1,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(10_000),
            },
        )
        .unwrap();
    }
    step_to(&mut game, 300);
    assert!(game.mobs().is_empty());
    game.world.bombs.clear();
    step_to(&mut game, 599);
    assert!(game.mobs().is_empty(), "a blocked attempt does not queue births");
    game.step(&[]).unwrap();
    assert_eq!(game.mobs().len(), 2);
    assert_eq!(
        game.mobs().iter().map(|mob| mob.id).collect::<Vec<_>>(),
        [EntityId(107), EntityId(108)]
    );
}

#[test]
fn disabled_profile_and_closed_rules_remain_without_overlay() {
    assert!(state(LairConfig::Disabled).lairs().is_none());

    let arena = fixed_arena();
    let actors = SeatId::DUEL.map(|seat| ActorInit {
        id: EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(arena.spawns()[seat.index()]),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    });
    let old = State::new(
        RulesConfig::environment_v7(RoomSettings::DEFAULT, StormCloudConfig::Disabled),
        World::new(arena),
        actors,
    )
    .unwrap();
    assert!(old.lairs().is_none());
    assert!(old.mobs().is_empty());
}

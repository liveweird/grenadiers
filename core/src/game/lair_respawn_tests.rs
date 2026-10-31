use super::{ActorInit, State};
use crate::game::{
    ArenaGenerationConfig, BombTrigger, CellId, Direction, EffectKind, EnabledItems, EntityId,
    PickupKind, PlayerCount, Position, RoundAssignment, RulesConfig, SeatId, SimulationSeed,
    SpeedLevel, StepError, StormCloudConfig, Tick, World, fixed_arena,
    generate_variable_multiplayer_arena,
};
use crate::room::{BoardSize, RespawnLairConfig, RespawnLairSettings, RoomSettings};

fn state(items: EnabledItems) -> State {
    let arena = fixed_arena();
    let actors = SeatId::DUEL.map(|seat| ActorInit {
        id: EntityId(10 + seat.index() as u32),
        seat,
        position: Position::center(arena.spawns()[seat.index()]),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    });
    State::new(
        RulesConfig::environment_v9(
            RoomSettings::DEFAULT.with_enabled_items(items),
            StormCloudConfig::Disabled,
            RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap()),
        ),
        World::new(arena),
        actors,
    )
    .unwrap()
}

fn seeded_state(items: EnabledItems) -> State {
    let assignment = RoundAssignment::first();
    let generated = generate_variable_multiplayer_arena(
        [41; 32],
        items,
        ArenaGenerationConfig {
            player_count: PlayerCount::Three,
            board: BoardSize::FifteenByThirteen,
            assignment,
        },
    )
    .unwrap();
    let (mut world, _) = World::from_generated(generated).unwrap();
    let blocks = world.arena.blocks().clone();
    world.arena.remove_blocks(&blocks);
    let actors = SeatId::ALL[..3]
        .iter()
        .copied()
        .map(|seat| ActorInit {
            id: EntityId(10 + seat.index() as u32),
            seat,
            position: Position::center(world.arena.spawns()[seat.index()]),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        })
        .collect::<Vec<_>>();
    State::new_at_tick_with_assignment_and_seed(
        RulesConfig::environment_v9(
            RoomSettings::DEFAULT.with_enabled_items(items),
            StormCloudConfig::Disabled,
            RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap()),
        ),
        world,
        actors,
        Tick(0),
        assignment,
        SimulationSeed::new([73; 32]),
    )
    .unwrap()
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

fn kill_mob(state: &mut State, lair: u8, expiry: Tick) {
    let cell = state
        .mobs()
        .into_iter()
        .find(|mob| mob.lair == lair)
        .unwrap()
        .position
        .cell()
        .unwrap();
    state.insert_test_flame(cell, expiry);
    state.step(&[]).unwrap();
}

#[test]
fn living_mobs_suspend_each_lairs_timer_and_never_accumulate_births() {
    let mut game = state(EnabledItems::EMPTY);
    step_to(&mut game, 300);
    assert_eq!(game.mobs().len(), 2);
    assert_eq!(
        game.lair_statuses().unwrap().map(|lair| lair.next_spawn_at),
        [None, None]
    );
    assert!(game.lairs().is_none(), "the finite rules8 view is closed");

    for at in [300, 470, 640, 810] {
        step_to(&mut game, at);
        game.apply_test_effect(SeatId::One, EffectKind::Stopwatch, Tick(at));
    }
    step_to(&mut game, 900);
    assert_eq!(
        game.lair_statuses().unwrap().map(|lair| lair.living_mobs),
        [1, 1]
    );
    assert_eq!(game.mobs().len(), 2);
}

#[test]
fn deaths_schedule_independent_full_grace_deadlines() {
    let mut game = state(EnabledItems::EMPTY);
    step_to(&mut game, 300);
    kill_mob(&mut game, 0, Tick(400));
    assert_eq!(game.tick(), Tick(301));
    assert_eq!(
        game.lair_statuses().unwrap().map(|lair| lair.next_spawn_at),
        [Some(Tick(601)), None]
    );
    step_to(&mut game, 310);
    kill_mob(&mut game, 1, Tick(400));
    assert_eq!(
        game.lair_statuses().unwrap().map(|lair| lair.next_spawn_at),
        [Some(Tick(601)), Some(Tick(611))]
    );

    step_to(&mut game, 600);
    assert!(game.mobs().is_empty());
    game.step(&[]).unwrap();
    assert_eq!(
        game.mobs().iter().map(|mob| mob.lair).collect::<Vec<_>>(),
        [0]
    );
    step_to(&mut game, 610);
    assert_eq!(game.mobs().len(), 1);
    game.step(&[]).unwrap();
    assert_eq!(
        game.mobs().iter().map(|mob| mob.lair).collect::<Vec<_>>(),
        [0, 1]
    );
}

#[test]
fn blocked_due_lair_retries_without_rotating_spawn_priority() {
    let mut game = state(EnabledItems::EMPTY);
    step_to(&mut game, 300);
    kill_mob(&mut game, 0, Tick(400));
    game.apply_test_effect(SeatId::One, EffectKind::Stopwatch, Tick(301));
    step_to(&mut game, 400);
    for (offset, cell) in [
        CellId::new(6, 5),
        CellId::new(7, 6),
        CellId::new(6, 7),
        CellId::new(5, 6),
    ]
    .into_iter()
    .enumerate()
    {
        game.insert_test_bomb(
            EntityId(100 + offset as u32),
            cell,
            EntityId(10),
            1,
            Tick(400),
            BombTrigger::Timed {
                detonates_at: Tick(10_000),
            },
        )
        .unwrap();
    }
    step_to(&mut game, 470);
    game.apply_test_effect(SeatId::One, EffectKind::Stopwatch, Tick(470));
    step_to(&mut game, 602);
    assert!(game.mobs().iter().all(|mob| mob.lair != 0));
    assert_eq!(
        game.lair_statuses().unwrap()[0].next_spawn_at,
        Some(Tick(601))
    );
    game.world.bombs.clear();
    game.step(&[]).unwrap();
    let replacement = game.mobs().into_iter().find(|mob| mob.lair == 0).unwrap();
    assert_eq!(replacement.position, Position::center(CellId::new(7, 6)));
    assert_eq!(game.lair_statuses().unwrap()[0].next_spawn_at, None);
}

#[test]
fn contact_mine_death_starts_replacement_grace() {
    let mut game = state(EnabledItems::LANDMINE);
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
    assert_eq!(game.tick(), Tick(302));
    assert_eq!(
        game.lair_statuses().unwrap()[0].next_spawn_at,
        Some(Tick(602))
    );
}

#[test]
fn death_timer_overflow_rolls_back_the_complete_step() {
    let mut death = state(EnabledItems::EMPTY);
    step_to(&mut death, 300);
    let cell = death.mobs()[0].position.cell().unwrap();
    death.tick = Tick(u64::MAX - 2);
    death.insert_test_flame(cell, Tick(u64::MAX));
    let before = death.clone();
    assert_eq!(death.step(&[]), Err(StepError::LairTimerOverflow));
    assert_eq!(death, before);
}

#[test]
fn late_birth_id_overflow_restores_seeded_randomness_and_state() {
    let mut birth = seeded_state(EnabledItems::TELEPORTER);
    step_to(&mut birth, 300);
    let cells = birth
        .mobs()
        .into_iter()
        .map(|mob| mob.position.cell().unwrap())
        .collect::<Vec<_>>();
    for cell in cells {
        birth.insert_test_flame(cell, Tick(400));
    }
    birth.step(&[]).unwrap();
    step_to(&mut birth, 600);
    let pickup = birth.actor(SeatId::One).unwrap().position().cell().unwrap();
    birth.insert_test_pickup(pickup, PickupKind::Teleporter);
    let mut successful = birth.clone();
    successful.step(&[]).unwrap();
    assert_ne!(
        successful.randomness, birth.randomness,
        "the update must consume simulation randomness before the late spawn"
    );
    birth.next_entity_id = u64::from(u32::MAX) + 1;
    let before = birth.clone();
    assert_eq!(birth.step(&[]), Err(StepError::EntityIdOverflow));
    assert_eq!(birth, before);
}

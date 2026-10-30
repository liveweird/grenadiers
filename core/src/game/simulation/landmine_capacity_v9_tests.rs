use crate::game::{
    ActorInit, BombTrigger, CellId, Direction, EffectKind, EnabledItems, EntityId, InputAction,
    Pickup, PickupKind, Position, RulesConfig, SeatId, SeatInput, SpecialWeapon, SpeedLevel, State,
    StepEventKind, StormCloudConfig, World, fixed_arena,
};
use crate::room::{LairConfig, RespawnLairConfig, RoomSettings};

fn actors() -> [ActorInit; 2] {
    [
        ActorInit {
            id: EntityId(10),
            seat: SeatId::One,
            position: Position::center(CellId::new(3, 1)),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        },
        ActorInit {
            id: EntityId(20),
            seat: SeatId::Two,
            position: Position::center(CellId::new(13, 11)),
            heading: Direction::West,
            speed: SpeedLevel::Base,
        },
    ]
}

fn rules(version: u16) -> RulesConfig {
    let items = EnabledItems::from_bits(
        EnabledItems::LANDMINE.bits()
            | EnabledItems::DIARRHEA.bits()
            | EnabledItems::REMOTE_BOMB.bits()
            | EnabledItems::SHIELD.bits(),
    )
    .unwrap();
    let settings = RoomSettings::DEFAULT.with_enabled_items(items);
    if version == 9 {
        RulesConfig::environment_v9(
            settings,
            StormCloudConfig::Disabled,
            RespawnLairConfig::Disabled,
        )
    } else {
        RulesConfig::environment_v8(settings, StormCloudConfig::Disabled, LairConfig::Disabled)
    }
}

fn state(version: u16) -> State {
    let mut arena = fixed_arena();
    let blocks = arena.blocks().clone();
    arena.remove_blocks(&blocks);
    State::new(rules(version), World::new(arena), actors()).unwrap()
}

fn place(state: &mut State) {
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
}

fn move_to(state: &mut State, cell: CellId) {
    state.actors[0].position = Position::center(cell);
    state.actors[0].bomb_grace.clear();
}

fn collect_weapon(state: &mut State, pickup: PickupKind, weapon: SpecialWeapon) {
    let cell = state.actors[0].position.cell().unwrap();
    assert_eq!(
        state
            .world
            .pickups
            .exposed
            .insert(cell, Pickup::new(pickup)),
        None
    );
    state.step(&[]).unwrap();
    assert_eq!(state.actors[0].inventory.weapon(), Some(weapon));
}

fn collect_mine(state: &mut State) {
    collect_weapon(state, PickupKind::Landmine, SpecialWeapon::Landmine);
}

#[test]
fn rules_nine_mines_are_extra_single_use_entities_not_normal_capacity_slots() {
    let mut game = state(9);
    collect_mine(&mut game);
    place(&mut game);
    let first_mine = *game.world.bombs.values().next().unwrap();
    assert_eq!(first_mine.trigger(), BombTrigger::Contact);
    assert_eq!(game.actors[0].inventory.weapon(), None);
    assert_eq!(game.live_bomb_count(SeatId::One), 0);

    move_to(&mut game, CellId::new(7, 1));
    place(&mut game);
    assert_eq!(game.world.bombs.len(), 2);
    assert_eq!(game.live_bomb_count(SeatId::One), 1);

    move_to(&mut game, CellId::new(9, 1));
    place(&mut game);
    assert_eq!(game.world.bombs.len(), 2);

    collect_mine(&mut game);
    place(&mut game);
    assert_eq!(game.actors[0].inventory.weapon(), None);
    assert_eq!(game.world.bombs.len(), 3);
    assert_eq!(game.live_bomb_count(SeatId::One), 1);

    move_to(&mut game, CellId::new(11, 1));
    game.actors[1].position = Position {
        x: 2_750,
        y: Position::center(first_mine.cell()).y,
    };
    game.actors[1].heading = Direction::East;
    let tick = game.tick();
    game.actors[1]
        .effects
        .apply(EffectKind::Shield, tick)
        .unwrap();
    let report = game
        .step(&[SeatInput {
            seat: SeatId::Two,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    assert!(!game.world.bombs.contains_key(&first_mine.id()));
    assert!(report.events.iter().any(
        |event| matches!(event.kind, StepEventKind::BombDetonated(view) if view.id == first_mine.id())
    ));
    assert_eq!(game.live_bomb_count(SeatId::One), 1);
    place(&mut game);
    assert_eq!(game.world.bombs.len(), 2);
}

#[test]
fn rules_nine_full_normal_capacity_still_accepts_a_held_mine() {
    let mut game = state(9);
    place(&mut game);
    assert_eq!(game.live_bomb_count(SeatId::One), 1);
    move_to(&mut game, CellId::new(7, 1));
    collect_mine(&mut game);
    place(&mut game);
    assert_eq!(game.world.bombs.len(), 2);
    assert_eq!(game.live_bomb_count(SeatId::One), 1);
    assert_eq!(game.actors[0].inventory.weapon(), None);
}

#[test]
fn rules_nine_remote_uses_a_normal_slot_and_keeps_trigger_priority_over_a_mine() {
    let mut game = state(9);
    collect_mine(&mut game);
    place(&mut game);
    move_to(&mut game, CellId::new(7, 1));
    collect_weapon(&mut game, PickupKind::RemoteBomb, SpecialWeapon::Remote);
    place(&mut game);
    assert_eq!(game.live_bomb_count(SeatId::One), 1);
    assert!(
        game.world
            .bombs
            .values()
            .any(|bomb| matches!(bomb.trigger(), BombTrigger::Remote))
    );

    move_to(&mut game, CellId::new(9, 1));
    collect_mine(&mut game);
    let tick = game.tick();
    game.actors[0]
        .effects
        .apply(EffectKind::Shield, tick)
        .unwrap();
    place(&mut game);
    assert_eq!(game.live_bomb_count(SeatId::One), 0);
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Landmine)
    );
    move_to(&mut game, CellId::new(11, 1));
    place(&mut game);
    assert_eq!(game.actors[0].inventory.weapon(), None);
    assert_eq!(game.live_bomb_count(SeatId::One), 0);
    assert_eq!(game.world.bombs.len(), 2);
}

#[test]
fn diarrhea_respects_normal_capacity_and_never_consumes_a_held_mine() {
    let mut game = state(9);
    collect_mine(&mut game);
    place(&mut game);
    let mine = game
        .world
        .bombs
        .values()
        .find(|bomb| bomb.trigger() == BombTrigger::Contact)
        .copied()
        .unwrap();

    move_to(&mut game, CellId::new(7, 1));
    place(&mut game);
    let ordinary = game
        .world
        .bombs
        .values()
        .find(|bomb| matches!(bomb.trigger(), BombTrigger::Timed { .. }))
        .copied()
        .unwrap();
    assert_eq!(game.live_bomb_count(SeatId::One), 1);

    move_to(&mut game, CellId::new(9, 1));
    collect_mine(&mut game);
    move_to(&mut game, CellId::new(11, 1));
    let tick = game.tick();
    game.actors[0]
        .effects
        .apply(EffectKind::Diarrhea, tick)
        .unwrap();
    let blocked = game.step(&[]).unwrap();
    assert!(
        blocked
            .events
            .iter()
            .all(|event| !matches!(event.kind, StepEventKind::BombPlaced(_)))
    );
    assert_eq!(game.world.bombs.len(), 2);
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Landmine)
    );

    game.insert_test_flame(ordinary.cell(), crate::game::Tick(game.tick().0 + 10));
    let destruction = game.step(&[]).unwrap();
    assert!(!game.world.bombs.contains_key(&ordinary.id()));
    assert!(game.world.bombs.contains_key(&mine.id()));
    assert!(destruction.events.iter().any(
        |event| matches!(event.kind, StepEventKind::BombDetonated(view) if view.id == ordinary.id())
    ));
    assert_eq!(game.live_bomb_count(SeatId::One), 0);
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Landmine)
    );

    let automatic = game.step(&[]).unwrap();
    assert!(game.world.bombs.contains_key(&mine.id()));
    assert_eq!(game.live_bomb_count(SeatId::One), 1);
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Landmine)
    );
    assert!(automatic.events.iter().any(|event| matches!(
        event.kind,
        StepEventKind::BombPlaced(view) if matches!(view.trigger, BombTrigger::Timed { .. })
    )));
}

#[test]
fn rules_eight_preserves_landmines_as_capacity_consumers() {
    let mut game = state(8);
    collect_mine(&mut game);
    place(&mut game);
    assert_eq!(game.live_bomb_count(SeatId::One), 1);
    move_to(&mut game, CellId::new(7, 1));
    place(&mut game);
    assert_eq!(game.world.bombs.len(), 1);
}

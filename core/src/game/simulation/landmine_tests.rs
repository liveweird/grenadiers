use std::collections::BTreeMap;

use crate::game::{
    ActorInit, BlastKind, Bomb, BombTrigger, CellId, Direction, EffectKind, EnabledItems, EntityId,
    InputAction, PickupKind, Position, RulesConfig, SeatId, SeatInput, SpecialWeapon, SpeedLevel,
    State, StateError, StepError, StepEventKind, Tick, World, fixed_arena,
};
use crate::room::RoomSettings;

fn actors(one: CellId, two: CellId) -> [ActorInit; 2] {
    [
        ActorInit {
            id: EntityId(10),
            seat: SeatId::One,
            position: Position::center(one),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        },
        ActorInit {
            id: EntityId(20),
            seat: SeatId::Two,
            position: Position::center(two),
            heading: Direction::West,
            speed: SpeedLevel::Base,
        },
    ]
}

fn rules(enabled: EnabledItems) -> RulesConfig {
    RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled))
}

fn state(enabled: EnabledItems, one: CellId, pickup: Option<PickupKind>, tick: Tick) -> State {
    State::new_at_tick(
        rules(enabled),
        World::with_pickups(
            fixed_arena(),
            BTreeMap::new(),
            BTreeMap::new(),
            pickup
                .map(|kind| BTreeMap::from([(one, kind)]))
                .unwrap_or_default(),
        )
        .unwrap(),
        actors(one, CellId::new(13, 11)),
        tick,
    )
    .unwrap()
}

fn mine(id: u32, owner: EntityId, cell: CellId, range: u16) -> Bomb {
    Bomb::new(
        EntityId(id),
        cell,
        owner,
        range,
        Tick(0),
        BombTrigger::Contact,
    )
    .unwrap()
}

fn input(action: InputAction) -> [SeatInput; 1] {
    [SeatInput {
        seat: SeatId::One,
        action,
    }]
}

#[test]
fn landmine_requires_its_item_and_standard_blast() {
    let cell = CellId::new(5, 5);
    let mut disabled = World::new(fixed_arena());
    disabled
        .bombs
        .insert(EntityId(30), mine(30, EntityId(10), cell, 2));
    assert_eq!(
        State::new(
            rules(EnabledItems::EMPTY),
            disabled,
            actors(CellId::new(1, 1), CellId::new(13, 11))
        ),
        Err(StateError::UnsupportedBombTrigger(EntityId(30)))
    );

    let mut special = World::new(fixed_arena());
    let bomb = Bomb::new_with_blast(
        EntityId(30),
        cell,
        EntityId(10),
        2,
        Tick(0),
        BombTrigger::Contact,
        BlastKind::Power,
    )
    .unwrap();
    special.bombs.insert(bomb.id(), bomb);
    let enabled =
        EnabledItems::from_bits(EnabledItems::LANDMINE.bits() | EnabledItems::POWER_BOMB.bits())
            .unwrap();
    assert_eq!(
        State::new(
            rules(enabled),
            special,
            actors(CellId::new(1, 1), CellId::new(13, 11))
        ),
        Err(StateError::UnsupportedBombBlast(EntityId(30)))
    );
}

#[test]
fn successful_manual_placement_consumes_charge_and_bypasses_fuse_overflow() {
    let enabled =
        EnabledItems::from_bits(EnabledItems::LANDMINE.bits() | EnabledItems::LONG_FUSE.bits())
            .unwrap();
    let mut game = state(
        enabled,
        CellId::new(3, 1),
        Some(PickupKind::Landmine),
        Tick(u64::MAX - 2),
    );
    game.step(&[]).unwrap();
    game.actors[0]
        .effects
        .apply(EffectKind::LongFuse, Tick(u64::MAX - 602))
        .unwrap();
    game.actors[0].range = 4;
    let report = game.step(&input(InputAction::PlaceBomb)).unwrap();
    let placed = *game.world.bombs.values().next().unwrap();
    assert_eq!(placed.trigger(), BombTrigger::Contact);
    assert_eq!(placed.blast(), BlastKind::Standard);
    assert_eq!(placed.range(), 4);
    assert_eq!(placed.placed_at(), Tick(u64::MAX));
    assert_eq!(placed.detonates_at(), None);
    assert_eq!(game.actors[0].inventory.weapon(), None);
    assert!(game.actors[0].bomb_grace.contains(&placed.id()));
    assert!(report.events.iter().any(|event| matches!(
        event.kind,
        StepEventKind::BombPlaced(view) if view.id == placed.id()
    )));
}

#[test]
fn initial_overlap_is_safe_until_full_exit_then_owner_reentry_detonates() {
    let cell = CellId::new(3, 1);
    let mut world = World::new(fixed_arena());
    world
        .bombs
        .insert(EntityId(30), mine(30, EntityId(10), cell, 1));
    let mut game = State::new(
        rules(EnabledItems::LANDMINE),
        world,
        actors(cell, CellId::new(13, 11)),
    )
    .unwrap();
    assert!(game.actors[0].bomb_grace.contains(&EntityId(30)));
    for _ in 0..3 {
        let report = game.step(&[]).unwrap();
        assert!(
            report
                .events
                .iter()
                .all(|event| !matches!(event.kind, StepEventKind::BombDetonated(_)))
        );
    }
    game.step(&input(InputAction::Press(Direction::West)))
        .unwrap();
    for _ in 1..26 {
        game.step(&[]).unwrap();
    }
    game.step(&input(InputAction::Clear)).unwrap();
    assert!(game.world.bombs.contains_key(&EntityId(30)));
    assert!(!game.actors[0].bomb_grace.contains(&EntityId(30)));

    let mut reports = vec![
        game.step(&input(InputAction::Press(Direction::East)))
            .unwrap(),
    ];
    while game.world.bombs.contains_key(&EntityId(30)) {
        reports.push(game.step(&[]).unwrap());
    }
    assert!(!game.world.bombs.contains_key(&EntityId(30)));
    assert!(
        reports
            .iter()
            .flat_map(|report| &report.events)
            .any(|event| {
                matches!(event.kind, StepEventKind::BombDetonated(view) if view.id == EntityId(30))
            })
    );
}

#[test]
fn nongraced_rival_can_trigger_while_owner_remains_graced() {
    let cell = CellId::new(3, 1);
    let mut world = World::new(fixed_arena());
    world
        .bombs
        .insert(EntityId(30), mine(30, EntityId(10), cell, 1));
    let mut initialized = actors(cell, CellId::new(2, 1));
    initialized[1].position = Position {
        x: 2_750,
        y: Position::center(cell).y,
    };
    let mut game = State::new(rules(EnabledItems::LANDMINE), world, initialized).unwrap();
    assert!(game.actors[0].bomb_grace.contains(&EntityId(30)));
    assert!(!game.actors[1].bomb_grace.contains(&EntityId(30)));
    let report = game
        .step(&[SeatInput {
            seat: SeatId::Two,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    assert!(report.events.iter().any(
        |event| matches!(event.kind, StepEventKind::BombDetonated(view) if view.id == EntityId(30))
    ));
}

#[test]
fn accepted_contact_persists_when_existing_fire_kills_the_actor_after_movement() {
    let mine_cell = CellId::new(3, 1);
    let mut game = state(EnabledItems::LANDMINE, CellId::new(2, 1), None, Tick(0));
    game.actors[0].position = Position {
        x: 2_750,
        y: Position::center(mine_cell).y,
    };
    game.insert_bomb(mine(30, game.actors[0].id, mine_cell, 1));
    game.actors[0].bomb_grace.clear();
    game.world.hazards.insert(CellId::new(2, 1), Tick(10));

    let report = game
        .step(&input(InputAction::Press(Direction::East)))
        .unwrap();

    assert!(!game.actors[0].alive());
    assert!(!game.world.bombs.contains_key(&EntityId(30)));
    assert!(report.events.iter().any(
        |event| matches!(event.kind, StepEventKind::BombDetonated(view) if view.id == EntityId(30))
    ));
}

#[test]
fn lingering_fire_destroys_a_mine_silently_without_extending_fire() {
    let cell = CellId::new(5, 5);
    let mut game = state(EnabledItems::LANDMINE, CellId::new(1, 1), None, Tick(0));
    game.insert_bomb(mine(30, game.actors[0].id, cell, 3));
    game.world.hazards.insert(cell, Tick(10));
    let report = game.step(&[]).unwrap();
    assert!(!game.world.bombs.contains_key(&EntityId(30)));
    assert_eq!(game.world.hazards.get(&cell), Some(&Tick(10)));
    assert!(
        report
            .events
            .iter()
            .all(|event| !matches!(event.kind, StepEventKind::BombDetonated(_)))
    );
}

#[test]
fn automatic_drop_never_consumes_or_places_the_landmine_charge() {
    let enabled =
        EnabledItems::from_bits(EnabledItems::LANDMINE.bits() | EnabledItems::DIARRHEA.bits())
            .unwrap();
    let mut game = state(enabled, CellId::new(3, 1), None, Tick(0));
    game.actors[0]
        .inventory
        .replace_weapon(SpecialWeapon::Landmine);
    game.actors[0]
        .effects
        .apply(EffectKind::Diarrhea, Tick(0))
        .unwrap();
    game.step(&[]).unwrap();
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Landmine)
    );
    assert!(matches!(
        game.world.bombs.values().next().unwrap().trigger(),
        BombTrigger::Timed { .. }
    ));
}

#[test]
fn rollback_restores_contact_detonation_movement_and_grace() {
    let enabled =
        EnabledItems::from_bits(EnabledItems::LANDMINE.bits() | EnabledItems::SHIELD.bits())
            .unwrap();
    let mut game = state(
        enabled,
        CellId::new(2, 1),
        Some(PickupKind::Shield),
        Tick(u64::MAX - 300),
    );
    game.actors[0].position = Position {
        x: 2_300,
        y: Position::center(CellId::new(3, 1)).y,
    };
    game.insert_bomb(mine(30, game.actors[0].id, CellId::new(3, 1), 1));
    game.actors[0].bomb_grace.clear();
    let before = game.clone();
    assert_eq!(
        game.step(&input(InputAction::Press(Direction::East))),
        Err(StepError::EffectExpiryOverflow(EffectKind::Shield))
    );
    assert_eq!(game, before);
}

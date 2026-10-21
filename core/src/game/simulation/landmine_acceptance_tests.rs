use std::collections::{BTreeMap, BTreeSet};

use crate::game::{
    ActorInit, Bomb, BombTrigger, CellId, Direction, EffectKind, EnabledItems, EntityId, Equipment,
    InputAction, Position, RulesConfig, SeatId, SeatInput, SpecialWeapon, SpeedLevel, State,
    StepEventKind, Tick, World, fixed_arena,
};
use crate::room::RoomSettings;

fn enabled(bits: u32) -> EnabledItems {
    EnabledItems::from_bits(bits).unwrap()
}

fn actors(one: Position, two: Position) -> [ActorInit; 2] {
    [
        ActorInit {
            id: EntityId(10),
            seat: SeatId::One,
            position: one,
            heading: Direction::East,
            speed: SpeedLevel::Base,
        },
        ActorInit {
            id: EntityId(20),
            seat: SeatId::Two,
            position: two,
            heading: Direction::West,
            speed: SpeedLevel::Base,
        },
    ]
}

fn state(items: EnabledItems, world: World, initialized: [ActorInit; 2]) -> State {
    State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(items)),
        world,
        initialized,
    )
    .unwrap()
}

fn bomb(id: u32, owner: u32, cell: CellId, range: u16, trigger: BombTrigger) -> Bomb {
    Bomb::new(EntityId(id), cell, EntityId(owner), range, Tick(0), trigger).unwrap()
}

fn seat(seat: SeatId, action: InputAction) -> SeatInput {
    SeatInput { seat, action }
}

#[test]
fn every_initially_overlapping_actor_is_graced_and_a_nongraced_entry_still_triggers() {
    let cell = CellId::new(3, 1);
    let mut world = World::new(fixed_arena());
    world
        .bombs
        .insert(EntityId(30), bomb(30, 10, cell, 1, BombTrigger::Contact));
    let both = actors(Position::center(cell), Position::center(cell));
    let mut game = state(EnabledItems::LANDMINE, world.clone(), both);
    assert!(
        game.actors
            .iter()
            .all(|actor| actor.bomb_grace.contains(&EntityId(30)))
    );
    assert!(game.step(&[]).unwrap().events.is_empty());

    let mut separate = actors(Position::center(cell), Position { x: 2_750, y: 1_536 });
    separate[0].heading = Direction::West;
    let mut game = state(EnabledItems::LANDMINE, world, separate);
    assert!(game.actors[0].bomb_grace.contains(&EntityId(30)));
    assert!(!game.actors[1].bomb_grace.contains(&EntityId(30)));
    let report = game
        .step(&[seat(SeatId::Two, InputAction::Press(Direction::East))])
        .unwrap();
    assert!(report.events.iter().any(
        |event| matches!(event.kind, StepEventKind::BombDetonated(view) if view.id == EntityId(30))
    ));
}

#[test]
fn one_swept_entry_can_accept_multiple_overlapping_mines() {
    let mut world = World::new(fixed_arena());
    for (id, cell) in [(30, CellId::new(3, 1)), (31, CellId::new(4, 1))] {
        world
            .bombs
            .insert(EntityId(id), bomb(id, 10, cell, 1, BombTrigger::Contact));
    }
    let initialized = actors(
        Position { x: 3_750, y: 1_536 },
        Position::center(CellId::new(13, 11)),
    );
    let mut game = state(EnabledItems::LANDMINE, world, initialized);
    game.actors[0].bomb_grace.clear();
    let report = game
        .step(&[seat(SeatId::One, InputAction::Press(Direction::East))])
        .unwrap();
    let detonated: BTreeSet<_> = report
        .events
        .iter()
        .filter_map(|event| match event.kind {
            StepEventKind::BombDetonated(view) => Some(view.id),
            _ => None,
        })
        .collect();
    assert_eq!(detonated, BTreeSet::from([EntityId(30), EntityId(31)]));
}

#[test]
fn actual_turn_correction_trace_contacts_on_its_second_segment() {
    let mut world = World::new(fixed_arena());
    world.bombs.insert(
        EntityId(30),
        bomb(30, 20, CellId::new(3, 4), 1, BombTrigger::Contact),
    );
    let mut initialized = actors(
        Position { x: 3_616, y: 5_445 },
        Position::center(CellId::new(13, 11)),
    );
    initialized[0].heading = Direction::East;
    let mut game = state(EnabledItems::LANDMINE, world, initialized);
    let report = game
        .step(&[seat(SeatId::One, InputAction::Press(Direction::North))])
        .unwrap();
    assert_eq!(report.traces[0].len(), 2);
    assert!(report.events.iter().any(
        |event| matches!(event.kind, StepEventKind::BombDetonated(view) if view.id == EntityId(30))
    ));
}

#[test]
fn shield_and_heart_each_protect_contact_damage_through_the_damage_gateway() {
    for protection in ["shield", "heart"] {
        let mut world = World::new(fixed_arena());
        world.bombs.insert(
            EntityId(30),
            bomb(30, 20, CellId::new(3, 1), 1, BombTrigger::Contact),
        );
        let initialized = actors(
            Position { x: 2_750, y: 1_536 },
            Position::center(CellId::new(13, 11)),
        );
        let items = enabled(
            EnabledItems::LANDMINE.bits()
                | EnabledItems::SHIELD.bits()
                | EnabledItems::HEART.bits(),
        );
        let mut game = state(items, world, initialized);
        if protection == "shield" {
            game.actors[0]
                .effects
                .apply(EffectKind::Shield, Tick(0))
                .unwrap();
        } else {
            game.actors[0].inventory.replace_equipment(Equipment::Heart);
        }
        let report = game
            .step(&[seat(SeatId::One, InputAction::Press(Direction::East))])
            .unwrap();
        assert!(game.actors[0].alive());
        assert!(report.events.iter().any(|event| matches!(
            event.kind,
            StepEventKind::DamagePrevented {
                id: EntityId(10),
                ..
            }
        )));
        assert_eq!(game.actors[0].inventory.equipment(), None);
        if protection == "heart" {
            assert!(game.actors[0].damage_grace.is_some());
        } else {
            assert!(game.actors[0].effects.damage_immune(Tick(1)));
        }
    }
}

#[test]
fn newly_placed_mine_is_not_a_same_tick_contact_target() {
    let cell = CellId::new(3, 1);
    let start = Position {
        x: i32::from(cell.column) * 1_024 - 1,
        y: Position::center(cell).y,
    };
    let mut game = state(
        EnabledItems::LANDMINE,
        World::new(fixed_arena()),
        actors(start, Position::center(CellId::new(13, 11))),
    );
    game.actors[0]
        .inventory
        .replace_weapon(SpecialWeapon::Landmine);
    let report = game
        .step(&[
            seat(SeatId::One, InputAction::Press(Direction::East)),
            seat(SeatId::One, InputAction::PlaceBomb),
        ])
        .unwrap();
    let segment = report.traces[0].segments().next().unwrap();
    assert_ne!(segment.start().cell(), Some(cell));
    assert_eq!(segment.end().cell(), Some(cell));
    assert!(segment.overlaps_cell(cell));
    assert_eq!(game.world.bombs.len(), 1);
    assert_eq!(game.world.bombs.values().next().unwrap().cell(), cell);
    assert!(report.events.iter().any(|event| matches!(
        event.kind,
        StepEventKind::BombPlaced(view) if view.trigger == BombTrigger::Contact
    )));
    assert!(
        report
            .events
            .iter()
            .all(|event| !matches!(event.kind, StepEventKind::BombDetonated(_)))
    );
}

#[test]
fn destruction_later_in_the_tick_does_not_release_capacity_for_placement() {
    let mut world = World::new(fixed_arena());
    world.bombs.insert(
        EntityId(30),
        bomb(30, 10, CellId::new(5, 5), 1, BombTrigger::Contact),
    );
    world.bombs.insert(
        EntityId(40),
        bomb(
            40,
            20,
            CellId::new(3, 5),
            3,
            BombTrigger::Timed {
                detonates_at: Tick(1),
            },
        ),
    );
    let mut game = state(
        EnabledItems::LANDMINE,
        world,
        actors(
            Position::center(CellId::new(1, 1)),
            Position::center(CellId::new(13, 11)),
        ),
    );
    game.actors[0]
        .inventory
        .replace_weapon(SpecialWeapon::Landmine);
    let report = game
        .step(&[seat(SeatId::One, InputAction::PlaceBomb)])
        .unwrap();
    assert!(game.world.bombs.is_empty());
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Landmine)
    );
    assert!(
        report
            .events
            .iter()
            .all(|event| !matches!(event.kind, StepEventKind::BombPlaced(_)))
    );
}

#[test]
fn armed_remote_action_has_priority_over_a_held_landmine() {
    let items = enabled(EnabledItems::REMOTE_BOMB.bits() | EnabledItems::LANDMINE.bits());
    let mut world = World::new(fixed_arena());
    world.bombs.insert(
        EntityId(30),
        bomb(30, 10, CellId::new(5, 5), 1, BombTrigger::Remote),
    );
    let mut game = state(
        items,
        world,
        actors(
            Position::center(CellId::new(1, 1)),
            Position::center(CellId::new(13, 11)),
        ),
    );
    game.actors[0]
        .inventory
        .replace_weapon(SpecialWeapon::Landmine);
    let report = game
        .step(&[seat(SeatId::One, InputAction::PlaceBomb)])
        .unwrap();
    assert!(game.world.bombs.is_empty());
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Landmine)
    );
    assert!(report.events.iter().any(
        |event| matches!(event.kind, StepEventKind::BombDetonated(view) if view.id == EntityId(30))
    ));
    assert!(
        report
            .events
            .iter()
            .all(|event| !matches!(event.kind, StepEventKind::BombPlaced(_)))
    );
}

#[test]
fn mixed_triggers_use_one_plan_and_silent_destruction_clears_grace() {
    let items = enabled(EnabledItems::REMOTE_BOMB.bits() | EnabledItems::LANDMINE.bits());
    let mut world = World::with_pickups(
        fixed_arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
    )
    .unwrap();
    for bomb in [
        bomb(30, 10, CellId::new(3, 5), 4, BombTrigger::Remote),
        bomb(
            40,
            20,
            CellId::new(7, 5),
            1,
            BombTrigger::Timed {
                detonates_at: Tick(1),
            },
        ),
        bomb(50, 10, CellId::new(5, 5), 1, BombTrigger::Contact),
        bomb(60, 20, CellId::new(12, 11), 1, BombTrigger::Contact),
    ] {
        world.bombs.insert(bomb.id(), bomb);
    }
    let mut game = state(
        items,
        world,
        actors(
            Position::center(CellId::new(1, 1)),
            Position {
                x: 11_950,
                y: 11_776,
            },
        ),
    );
    for actor in &mut game.actors {
        actor.bomb_grace.insert(EntityId(50));
    }
    let report = game
        .step(&[
            seat(SeatId::One, InputAction::PlaceBomb),
            seat(SeatId::Two, InputAction::Press(Direction::East)),
        ])
        .unwrap();
    let detonated: BTreeSet<_> = report
        .events
        .iter()
        .filter_map(|event| match event.kind {
            StepEventKind::BombDetonated(view) => Some(view.id),
            _ => None,
        })
        .collect();
    assert_eq!(
        detonated,
        BTreeSet::from([EntityId(30), EntityId(40), EntityId(60)])
    );
    assert!(!detonated.contains(&EntityId(50)));
    assert!(game.world.bombs.is_empty());
    assert!(game.world.hazards.contains_key(&CellId::new(6, 5)));
    assert!(
        game.actors
            .iter()
            .all(|actor| !actor.bomb_grace.contains(&EntityId(50)))
    );
}

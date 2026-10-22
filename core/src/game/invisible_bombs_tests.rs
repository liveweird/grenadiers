use std::collections::BTreeMap;

use super::*;
use crate::room::RoomSettings;

fn input(seat: SeatId, action: InputAction) -> SeatInput {
    SeatInput { seat, action }
}

fn move_cells(game: &mut State, seat: SeatId, direction: Direction, cells: usize) {
    game.step(&[input(seat, InputAction::Press(direction))])
        .unwrap();
    for _ in 1..cells * 20 {
        game.step(&[]).unwrap();
    }
    game.step(&[input(seat, InputAction::Release(direction))])
        .unwrap();
}

fn generated_state(fuse: u64) -> State {
    let generated = generate_expanded_arena([31; 32], EnabledItems::INVISIBLE_BOMBS);
    let (arena, concealed, _) = generated.into_parts();
    let spawns = arena.spawns();
    let settings = RoomSettings::new(SpeedLevel::Base, 256, 3, fuse)
        .unwrap()
        .with_enabled_items(EnabledItems::INVISIBLE_BOMBS);
    State::new(
        RulesConfig::expanded_v5(settings),
        World::with_pickups(arena, concealed, BTreeMap::new(), BTreeMap::new()).unwrap(),
        SeatId::ALL.map(|seat| ActorInit {
            id: EntityId(seat.index() as u32 + 10),
            seat,
            position: Position::center(spawns[seat.index()]),
            heading: if seat == SeatId::One {
                Direction::East
            } else {
                Direction::West
            },
            speed: SpeedLevel::Base,
        }),
    )
    .unwrap()
}

fn is_bomb_event(event: &StepEvent) -> bool {
    matches!(
        event.kind,
        StepEventKind::BombPlaced(_) | StepEventKind::BombDetonated(_)
    )
}

#[test]
fn bomb_event_audience_intersects_cloak_and_each_blinded_recipient() {
    let mut game = generated_state(180);
    game.apply_test_effect(SeatId::One, EffectKind::InvisibleBombs, Tick(0));
    let two_placement = game
        .step(&[input(SeatId::Two, InputAction::PlaceBomb)])
        .unwrap();
    assert!(two_placement.events.iter().any(|event| {
        event.audience == EventAudience::Two && matches!(event.kind, StepEventKind::BombPlaced(_))
    }));

    let mut hidden_owner = generated_state(180);
    hidden_owner.apply_test_effect(SeatId::One, EffectKind::InvisibleBombs, Tick(0));
    hidden_owner.apply_test_effect(SeatId::One, EffectKind::InvisibilityCloak, Tick(0));
    hidden_owner.apply_test_effect(SeatId::Two, EffectKind::InvisibleBombs, Tick(0));
    let private = hidden_owner
        .step(&[input(SeatId::One, InputAction::PlaceBomb)])
        .unwrap();
    assert!(private.events.iter().any(|event| {
        event.audience == EventAudience::Neither
            && matches!(event.kind, StepEventKind::BombPlaced(_))
    }));
}

#[test]
fn seed_31_proves_the_full_ordinary_blindness_route_and_expiry_reveal() {
    let mut game = generated_state(300);
    move_cells(&mut game, SeatId::One, Direction::East, 1);
    game.step(&[input(SeatId::One, InputAction::PlaceBomb)])
        .unwrap();
    move_cells(&mut game, SeatId::One, Direction::West, 1);
    move_cells(&mut game, SeatId::One, Direction::South, 1);
    while !game.world().bombs().is_empty()
        || game.world().hazards().contains_key(&CellId::new(3, 1))
    {
        game.step(&[]).unwrap();
    }

    move_cells(&mut game, SeatId::One, Direction::North, 1);
    game.step(&[input(SeatId::One, InputAction::Press(Direction::East))])
        .unwrap();
    let collected = loop {
        let report = game.step(&[]).unwrap();
        if report.events.iter().any(|event| {
            matches!(
                event.kind,
                StepEventKind::PickupCollected {
                    kind: PickupKind::InvisibleBombs,
                    ..
                }
            )
        }) {
            break report.tick;
        }
    };
    while game.actors()[0].position().x < Position::center(CellId::new(3, 1)).x {
        game.step(&[]).unwrap();
    }
    game.step(&[input(SeatId::One, InputAction::Release(Direction::East))])
        .unwrap();
    assert!(game.actors()[0].effects().hides_bombs(collected));

    let own = game
        .step(&[input(SeatId::One, InputAction::PlaceBomb)])
        .unwrap();
    let own_id = match own.events.iter().find_map(|event| match event.kind {
        StepEventKind::BombPlaced(bomb) => Some(bomb.id),
        _ => None,
    }) {
        Some(id) => id,
        None => panic!("ordinary placement must succeed"),
    };
    assert!(
        own.events
            .iter()
            .filter(|event| is_bomb_event(event))
            .all(|event| !event.audience.includes(SeatId::One))
    );
    move_cells(&mut game, SeatId::One, Direction::West, 2);
    move_cells(&mut game, SeatId::One, Direction::South, 1);

    move_cells(&mut game, SeatId::Two, Direction::West, 1);
    let rival = game
        .step(&[input(SeatId::Two, InputAction::PlaceBomb)])
        .unwrap();
    let rival_id = rival
        .events
        .iter()
        .find_map(|event| match event.kind {
            StepEventKind::BombPlaced(bomb) => Some(bomb.id),
            _ => None,
        })
        .unwrap();
    assert!(
        rival
            .events
            .iter()
            .filter(|event| is_bomb_event(event))
            .all(|event| !event.audience.includes(SeatId::One))
    );
    assert_eq!(game.world().bombs().len(), 2);
    move_cells(&mut game, SeatId::Two, Direction::East, 1);
    move_cells(&mut game, SeatId::Two, Direction::North, 1);

    let mut hidden_detonations = Vec::new();
    while game.world().bombs().contains_key(&own_id) || game.world().bombs().contains_key(&rival_id)
    {
        hidden_detonations.extend(game.step(&[]).unwrap().events);
    }
    assert!(
        hidden_detonations
            .iter()
            .filter(|event| is_bomb_event(event))
            .all(|event| !event.audience.includes(SeatId::One))
    );
    assert!(!game.world().hazards().is_empty());
    while !game.world().hazards().is_empty() {
        game.step(&[]).unwrap();
    }

    let deadline = Tick(collected.0 + EffectKind::InvisibleBombs.duration_ticks());
    while game.tick().0 < collected.0 + 400 {
        game.step(&[]).unwrap();
    }
    move_cells(&mut game, SeatId::Two, Direction::South, 1);
    move_cells(&mut game, SeatId::Two, Direction::West, 1);
    let late = game
        .step(&[input(SeatId::Two, InputAction::PlaceBomb)])
        .unwrap();
    let late_bomb = late
        .events
        .iter()
        .find_map(|event| match event.kind {
            StepEventKind::BombPlaced(bomb) => Some(bomb),
            _ => None,
        })
        .unwrap();
    assert!((collected.0 + 390..=collected.0 + 470).contains(&late.tick.0));
    assert_eq!(late_bomb.cell, CellId::new(12, 11));
    assert!(
        late.events
            .iter()
            .filter(|event| is_bomb_event(event))
            .all(|event| !event.audience.includes(SeatId::One))
    );
    move_cells(&mut game, SeatId::Two, Direction::East, 1);
    move_cells(&mut game, SeatId::Two, Direction::North, 1);

    while game.tick().0 + 1 < deadline.0 {
        assert!(game.world().bombs().contains_key(&late_bomb.id));
        game.step(&[]).unwrap();
    }
    assert!(game.actors()[0].effects().hides_bombs(game.tick()));
    game.step(&[]).unwrap();
    assert_eq!(game.tick(), deadline);
    assert!(!game.actors()[0].effects().hides_bombs(game.tick()));
    let revealed = game.world().bombs()[&late_bomb.id];
    assert_eq!(revealed.cell(), late_bomb.cell);
    assert_eq!(
        revealed.detonates_at().unwrap().0 - deadline.0,
        late_bomb.placed_at.0 + 300 - deadline.0
    );
    assert!(revealed.detonates_at().unwrap() > deadline);
}

#[test]
fn bomb_audience_uses_effect_state_at_emission_across_both_damage_phases() {
    let mut old_fire = generated_state(180);
    old_fire.apply_test_effect(SeatId::One, EffectKind::InvisibleBombs, Tick(0));
    let one_cell = old_fire.actors()[0].position().cell().unwrap();
    old_fire.insert_test_flame(one_cell, Tick(2));
    old_fire
        .insert_test_bomb(
            EntityId(30),
            CellId::new(7, 5),
            EntityId(11),
            1,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(1),
            },
        )
        .unwrap();
    let report = old_fire.step(&[]).unwrap();
    assert!(!old_fire.actors()[0].alive());
    assert!(report.events.iter().any(|event| {
        event.audience == EventAudience::Both
            && matches!(event.kind, StepEventKind::BombDetonated(bomb) if bomb.id == EntityId(30))
    }));

    let mut final_fire = generated_state(180);
    final_fire.apply_test_effect(SeatId::One, EffectKind::InvisibleBombs, Tick(0));
    let one_cell = final_fire.actors()[0].position().cell().unwrap();
    final_fire
        .insert_test_bomb(
            EntityId(30),
            one_cell,
            EntityId(11),
            1,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(1),
            },
        )
        .unwrap();
    let report = final_fire.step(&[]).unwrap();
    assert!(!final_fire.actors()[0].alive());
    assert!(report.events.iter().any(|event| {
        event.audience == EventAudience::Two
            && matches!(event.kind, StepEventKind::BombDetonated(bomb) if bomb.id == EntityId(30))
    }));
    assert!(report.events.iter().any(|event| {
        event.audience == EventAudience::Both
            && matches!(
                event.kind,
                StepEventKind::ActorEliminated {
                    seat: SeatId::One,
                    ..
                }
            )
    }));
}

#[test]
fn late_collection_overflow_rolls_back_placement_input_and_world() {
    let at = Tick(u64::MAX - EffectKind::InvisibleBombs.duration_ticks());
    let arena = fixed_arena();
    let spawns = arena.spawns();
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(spawns[0], PickupKind::InvisibleBombs)]),
    )
    .unwrap();
    let mut game = State::new_at_tick(
        RulesConfig::expanded_v5(
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::INVISIBLE_BOMBS),
        ),
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
    let before = game.clone();
    assert_eq!(
        game.step(&[
            input(SeatId::One, InputAction::PlaceBomb),
            input(SeatId::Two, InputAction::Press(Direction::West)),
        ]),
        Err(StepError::EffectExpiryOverflow(EffectKind::InvisibleBombs))
    );
    assert_eq!(game, before);
}

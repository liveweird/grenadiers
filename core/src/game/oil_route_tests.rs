use std::collections::BTreeMap;

use crate::room::RoomSettings;

use super::*;

fn input(action: InputAction) -> [SeatInput; 1] {
    [SeatInput {
        seat: SeatId::One,
        action,
    }]
}

fn move_cells(state: &mut State, direction: Direction, cells: usize) {
    state.step(&input(InputAction::Press(direction))).unwrap();
    for _ in 1..cells * 20 {
        state.step(&[]).unwrap();
    }
    state.step(&input(InputAction::Release(direction))).unwrap();
}

fn advance_to(state: &mut State, target: Tick) {
    let remaining = target.0.checked_sub(state.tick().0).unwrap();
    assert!(remaining <= 300, "route phase must stay tightly bounded");
    for _ in 0..remaining {
        state.step(&[]).unwrap();
    }
    assert_eq!(state.tick(), target);
}

fn placed_bomb(report: &StepReport) -> BombView {
    report
        .events
        .iter()
        .find_map(|event| match event.kind {
            StepEventKind::BombPlaced(view) => Some(view),
            _ => None,
        })
        .expect("route placement emits its ordinary bomb")
}

fn deadline(view: BombView) -> Tick {
    match view.trigger {
        BombTrigger::Timed { detonates_at } => detonates_at,
        BombTrigger::Remote | BombTrigger::Contact => panic!("ordinary route uses a timed bomb"),
    }
}

#[test]
fn seed_31_proves_the_ordinary_reveal_clamp_ignite_and_persistent_route() {
    let generated = generate_expanded_arena([31; 32], EnabledItems::OIL_BARREL);
    let (arena, concealed, _) = generated.into_parts();
    for cell in [CellId::new(4, 1), CellId::new(3, 2)] {
        assert!(arena.blocks().contains(&cell));
    }
    let spawns = arena.spawns().to_vec();
    let actors = SeatId::DUEL.map(|seat| ActorInit {
        id: EntityId(seat.index() as u32 + 10),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    });
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::OIL_BARREL);
    let mut state = State::new(
        RulesConfig::expanded_v5(settings),
        World::with_pickups(arena, concealed, BTreeMap::new(), BTreeMap::new()).unwrap(),
        actors,
    )
    .unwrap();
    let oil = CellId::new(3, 1);

    move_cells(&mut state, Direction::East, 1);
    let first = placed_bomb(&state.step(&input(InputAction::PlaceBomb)).unwrap());
    assert_eq!(first.placed_at, state.tick());
    assert_eq!(
        first.trigger,
        BombTrigger::Timed {
            detonates_at: Tick(first.placed_at.0 + 180),
        }
    );
    let first_deadline = deadline(first);
    move_cells(&mut state, Direction::West, 1);
    move_cells(&mut state, Direction::South, 1);
    advance_to(&mut state, Tick(first_deadline.0 - 1));
    assert!(state.world().bombs().contains_key(&first.id));
    assert_eq!(state.world().oil.state(oil), Some(OilState::Concealed));
    assert!(state.public_view().oil_barrels.is_empty());

    let first_detonation = state.step(&[]).unwrap();
    assert_eq!(first_detonation.tick, first_deadline);
    assert!(!state.world().bombs().contains_key(&first.id));
    assert_eq!(state.world().oil.state(oil), Some(OilState::Pending));
    assert!(state.public_view().oil_barrels.is_empty());
    let first_fire_expiry = Tick(first_deadline.0 + state.rules().flame_ticks());
    assert_eq!(state.world().hazards().get(&oil), Some(&first_fire_expiry));

    advance_to(&mut state, Tick(first_fire_expiry.0 - 1));
    assert_eq!(state.world().oil.state(oil), Some(OilState::Pending));
    assert!(state.public_view().oil_barrels.is_empty());
    assert_eq!(state.world().hazards().get(&oil), Some(&first_fire_expiry));
    state.step(&[]).unwrap();
    assert_eq!(state.tick(), first_fire_expiry);
    assert!(!state.world().hazards().contains_key(&oil));
    assert_eq!(state.world().oil.state(oil), Some(OilState::Unlit));
    assert!(
        state
            .public_view()
            .oil_barrels
            .iter()
            .any(|barrel| { barrel.cell == oil && barrel.state == OilBarrelState::Unlit })
    );
    assert!(state.world().arena().blocks().contains(&CellId::new(4, 1)));
    assert!(state.world().arena().blocks().contains(&CellId::new(3, 2)));

    move_cells(&mut state, Direction::North, 1);
    move_cells(&mut state, Direction::East, 1);
    state
        .step(&input(InputAction::Press(Direction::East)))
        .unwrap();
    for _ in 0..10 {
        state.step(&[]).unwrap();
    }
    state
        .step(&input(InputAction::Release(Direction::East)))
        .unwrap();
    assert_eq!(state.actors()[0].position().x, 2_752);
    let second = placed_bomb(&state.step(&input(InputAction::PlaceBomb)).unwrap());
    assert_eq!(second.cell, CellId::new(2, 1));
    assert_eq!(second.placed_at, state.tick());
    assert_eq!(
        second.trigger,
        BombTrigger::Timed {
            detonates_at: Tick(second.placed_at.0 + 180),
        }
    );
    let second_deadline = deadline(second);

    state
        .step(&input(InputAction::Press(Direction::West)))
        .unwrap();
    for _ in 0..30 {
        if state.actors()[0].position().x <= Position::center(CellId::new(1, 1)).x {
            break;
        }
        state.step(&[]).unwrap();
    }
    assert!(state.actors()[0].position().x <= Position::center(CellId::new(1, 1)).x);
    state.step(&input(InputAction::Clear)).unwrap();
    move_cells(&mut state, Direction::South, 1);
    advance_to(&mut state, Tick(second_deadline.0 - 1));
    assert!(state.world().bombs().contains_key(&second.id));
    assert_eq!(state.world().oil.state(oil), Some(OilState::Unlit));

    let second_detonation = state.step(&[]).unwrap();
    assert_eq!(second_detonation.tick, second_deadline);
    assert_eq!(state.world().oil.state(oil), Some(OilState::Burning));
    assert!(!state.world().hazards().contains_key(&oil));
    let second_fire_expiry = Tick(second_deadline.0 + state.rules().flame_ticks());
    assert!(!state.world().hazards().is_empty());
    assert!(
        state
            .world()
            .hazards()
            .values()
            .all(|expiry| *expiry == second_fire_expiry)
    );
    assert!(state.actors().iter().all(Actor::alive));

    advance_to(&mut state, Tick(second_fire_expiry.0 - 1));
    assert!(!state.world().hazards().is_empty());
    assert_eq!(state.world().oil.state(oil), Some(OilState::Burning));
    state.step(&[]).unwrap();
    assert_eq!(state.tick(), second_fire_expiry);
    assert!(state.world().hazards().is_empty());
    advance_to(&mut state, Tick(second_deadline.0 + 60));
    assert!(state.world().hazards().is_empty());
    assert_eq!(state.world().oil.state(oil), Some(OilState::Burning));
    assert!(
        state
            .public_view()
            .oil_barrels
            .iter()
            .any(|barrel| { barrel.cell == oil && barrel.state == OilBarrelState::Burning })
    );
    assert!(state.actors().iter().all(Actor::alive));

    move_cells(&mut state, Direction::North, 1);
    move_cells(&mut state, Direction::East, 1);
    assert_eq!(state.actors()[0].position().cell(), Some(CellId::new(2, 1)));
    assert!(state.actors().iter().all(Actor::alive));
}

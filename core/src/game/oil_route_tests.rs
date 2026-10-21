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

#[test]
fn seed_31_proves_the_ordinary_reveal_clamp_ignite_and_persistent_route() {
    let generated = generate_expanded_arena([31; 32], EnabledItems::OIL_BARREL);
    let (arena, concealed, _) = generated.into_parts();
    for cell in [CellId::new(4, 1), CellId::new(3, 2)] {
        assert!(arena.blocks().contains(&cell));
    }
    let spawns = arena.spawns();
    let actors = SeatId::ALL.map(|seat| ActorInit {
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
    state.step(&input(InputAction::PlaceBomb)).unwrap();
    move_cells(&mut state, Direction::West, 1);
    move_cells(&mut state, Direction::South, 1);
    while !state.world().bombs().is_empty() || state.world().hazards().contains_key(&oil) {
        state.step(&[]).unwrap();
    }
    state.step(&[]).unwrap();
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
    state.step(&input(InputAction::PlaceBomb)).unwrap();
    assert_eq!(
        state.world().bombs().values().next().unwrap().cell(),
        CellId::new(2, 1)
    );

    state
        .step(&input(InputAction::Press(Direction::West)))
        .unwrap();
    while state.actors()[0].position().x > Position::center(CellId::new(1, 1)).x {
        state.step(&[]).unwrap();
    }
    state.step(&input(InputAction::Clear)).unwrap();
    move_cells(&mut state, Direction::South, 1);
    while !state.world().bombs().is_empty() || !state.world().hazards().is_empty() {
        state.step(&[]).unwrap();
    }
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

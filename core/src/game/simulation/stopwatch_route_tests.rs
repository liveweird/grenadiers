use std::collections::BTreeMap;

use crate::game::{
    ActorInit, CellId, Direction, EffectKind, EnabledItems, EntityId, InputAction, Position,
    RulesConfig, SeatId, SeatInput, SpeedLevel, State, StepEventKind, World,
    generate_expanded_arena,
};
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

fn wait_until(game: &mut State, predicate: impl Fn(&State) -> bool) {
    while !predicate(game) {
        game.step(&[]).unwrap();
    }
}

#[test]
fn seed_31_proves_two_sequential_ordinary_stopwatch_windows() {
    let generated = generate_expanded_arena([31; 32], EnabledItems::STOPWATCH);
    let (arena, concealed, _) = generated.into_parts();
    let spawns = arena.spawns();
    let mut game = State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::STOPWATCH)),
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
    .unwrap();

    move_cells(&mut game, SeatId::One, Direction::East, 1);
    game.step(&[input(SeatId::One, InputAction::PlaceBomb)])
        .unwrap();
    move_cells(&mut game, SeatId::One, Direction::West, 1);
    move_cells(&mut game, SeatId::One, Direction::South, 1);
    move_cells(&mut game, SeatId::Two, Direction::West, 1);
    game.step(&[input(SeatId::Two, InputAction::PlaceBomb)])
        .unwrap();
    move_cells(&mut game, SeatId::Two, Direction::East, 1);
    move_cells(&mut game, SeatId::Two, Direction::North, 1);
    wait_until(&mut game, |state| {
        state.world.bombs.is_empty()
            && !state.world.hazards.contains_key(&CellId::new(3, 1))
            && !state.world.hazards.contains_key(&CellId::new(11, 11))
    });

    game.step(&[input(SeatId::One, InputAction::PlaceBomb)])
        .unwrap();
    let original = *game.world.bombs.values().next().unwrap();
    move_cells(&mut game, SeatId::One, Direction::North, 1);
    move_cells(&mut game, SeatId::One, Direction::East, 2);
    assert_eq!(game.actors[0].position, Position::center(CellId::new(3, 1)));
    let deadline = game.actors[0]
        .effect_observations()
        .find_map(|(kind, deadline)| (kind == EffectKind::Stopwatch).then_some(deadline))
        .expect("ordinary movement must collect the seeded stopwatch");
    assert!(
        game.actors[0]
            .effects
            .active(EffectKind::Stopwatch, game.tick())
    );
    let frozen_start = game.actors[1].position;
    let blocked = game
        .step(&[
            input(SeatId::Two, InputAction::Press(Direction::South)),
            input(SeatId::Two, InputAction::PlaceBomb),
        ])
        .unwrap();
    assert_eq!(game.actors[1].position, frozen_start);
    assert_eq!(game.world.bombs.len(), 1);
    assert!(
        blocked
            .events
            .iter()
            .all(|event| !matches!(event.kind, StepEventKind::BombPlaced(_)))
    );
    assert!(game.timed_bombs_paused());

    move_cells(&mut game, SeatId::One, Direction::West, 1);
    while game.tick().0 + 1 < deadline.0 {
        game.step(&[]).unwrap();
    }
    assert_eq!(game.actors[1].position, frozen_start);
    game.step(&[]).unwrap();
    assert_eq!(game.tick(), deadline);
    assert_ne!(game.actors[1].position, frozen_start);
    game.step(&[input(SeatId::Two, InputAction::Release(Direction::South))])
        .unwrap();
    assert_eq!(game.world.bombs.len(), 1);
    let extended = game.world.bombs[&original.id()];
    assert_eq!(extended.timed_history().unwrap().paused_ticks, 179);
    wait_until(&mut game, |state| {
        !state.world.bombs.contains_key(&original.id())
    });
    wait_until(&mut game, |state| state.world.hazards.is_empty());

    move_cells(&mut game, SeatId::Two, Direction::South, 1);
    move_cells(&mut game, SeatId::Two, Direction::West, 2);
    assert_eq!(game.actors[1].position.cell(), Some(CellId::new(11, 11)));
    assert!(
        game.actors[1]
            .effects
            .active(EffectKind::Stopwatch, game.tick())
    );
    let first_start = game.actors[0].position;
    let paused_placement = game
        .step(&[
            input(SeatId::One, InputAction::Press(Direction::West)),
            input(SeatId::One, InputAction::PlaceBomb),
            input(SeatId::Two, InputAction::PlaceBomb),
        ])
        .unwrap();
    assert_eq!(game.actors[0].position, first_start);
    assert_eq!(game.world.bombs.len(), 1);
    let fresh = *game.world.bombs.values().next().unwrap();
    assert_eq!(fresh.cell(), CellId::new(11, 11));
    assert_eq!(fresh.timed_history().unwrap().paused_ticks, 0);
    let placed = paused_placement
        .events
        .iter()
        .find(
            |event| matches!(event.kind, StepEventKind::BombPlaced(view) if view.id == fresh.id()),
        )
        .unwrap();
    assert!(placed.timed_bomb.unwrap().paused);
    assert_eq!(placed.timed_bomb.unwrap().history.paused_ticks, 0);

    let second_deadline = game.actors[1]
        .effect_observations()
        .find_map(|(kind, deadline)| (kind == EffectKind::Stopwatch).then_some(deadline))
        .unwrap();
    move_cells(&mut game, SeatId::Two, Direction::East, 2);
    move_cells(&mut game, SeatId::Two, Direction::North, 1);
    assert_eq!(game.actors[1].position.cell(), Some(CellId::new(13, 10)));
    while game.tick().0 + 1 < second_deadline.0 {
        game.step(&[]).unwrap();
    }
    assert_eq!(game.actors[0].position, first_start);
    assert_eq!(game.world.bombs.len(), 1);
    game.step(&[]).unwrap();
    assert_eq!(game.tick(), second_deadline);
    assert_ne!(game.actors[0].position, first_start);
    game.step(&[input(SeatId::One, InputAction::Release(Direction::West))])
        .unwrap();
    assert_eq!(game.world.bombs.len(), 1, "blocked action must not replay");
    let resumed = game.world.bombs[&fresh.id()];
    let expected_paused = second_deadline.0 - resumed.placed_at().0 - 1;
    assert_eq!(
        resumed.timed_history().unwrap().paused_ticks,
        expected_paused
    );
    assert_eq!(
        resumed.detonates_at().unwrap().0,
        resumed.placed_at().0 + 180 + expected_paused
    );
    while game.tick().0 + 1 < resumed.detonates_at().unwrap().0 {
        game.step(&[]).unwrap();
    }
    assert!(game.world.bombs.contains_key(&fresh.id()));
    let detonation = game.step(&[]).unwrap();
    assert!(detonation.events.iter().any(|event| matches!(
        event.kind,
        StepEventKind::BombDetonated(view) if view.id == fresh.id()
    )));
    assert!(!game.world.bombs.contains_key(&fresh.id()));
}

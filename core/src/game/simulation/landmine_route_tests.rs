use std::collections::BTreeMap;

use crate::game::{
    ActorInit, BombTrigger, CellId, Direction, EnabledItems, EntityId, InputAction, Position,
    RulesConfig, SeatId, SeatInput, SpecialWeapon, SpeedLevel, State, StepEventKind, World,
    generate_expanded_arena,
};
use crate::room::RoomSettings;

fn input(action: InputAction) -> [SeatInput; 1] {
    [SeatInput {
        seat: SeatId::One,
        action,
    }]
}

fn move_cells(game: &mut State, direction: Direction, cells: usize) {
    game.step(&input(InputAction::Press(direction))).unwrap();
    for _ in 1..cells * 20 {
        game.step(&[]).unwrap();
    }
    game.step(&input(InputAction::Release(direction))).unwrap();
}

#[test]
fn seed_31_proves_the_entire_reveal_collect_plant_wait_exit_and_contact_route() {
    let generated = generate_expanded_arena([31; 32], EnabledItems::LANDMINE);
    let (arena, concealed, _) = generated.into_parts();
    let spawns = arena.spawns().to_vec();
    let rules =
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::LANDMINE));
    let mut game = State::new(
        rules,
        World::with_pickups(arena, concealed, BTreeMap::new(), BTreeMap::new()).unwrap(),
        SeatId::DUEL.map(|seat| ActorInit {
            id: EntityId(seat.index() as u32 + 10),
            seat,
            position: Position::center(spawns[seat.index()]),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        }),
    )
    .unwrap();

    move_cells(&mut game, Direction::East, 1);
    assert_eq!(game.actors[0].position, Position::center(CellId::new(2, 1)));
    game.step(&input(InputAction::PlaceBomb)).unwrap();
    move_cells(&mut game, Direction::West, 1);
    move_cells(&mut game, Direction::South, 1);
    assert_eq!(game.actors[0].position, Position::center(CellId::new(1, 2)));
    while !game.world.bombs.is_empty() || game.world.hazards.contains_key(&CellId::new(3, 1)) {
        game.step(&[]).unwrap();
    }
    move_cells(&mut game, Direction::North, 1);
    move_cells(&mut game, Direction::East, 2);
    assert_eq!(game.actors[0].position, Position::center(CellId::new(3, 1)));
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(SpecialWeapon::Landmine)
    );

    let placement = game.step(&input(InputAction::PlaceBomb)).unwrap();
    let mine = *game.world.bombs.values().next().unwrap();
    assert_eq!(
        (mine.cell(), mine.trigger(), mine.range()),
        (CellId::new(3, 1), BombTrigger::Contact, 2)
    );
    assert_eq!(game.actors[0].inventory.weapon(), None);
    assert_eq!(
        placement
            .events
            .iter()
            .filter(|event| matches!(event.kind, StepEventKind::BombPlaced(_)))
            .count(),
        1
    );
    for _ in 0..3 {
        assert!(
            game.step(&[])
                .unwrap()
                .events
                .iter()
                .all(|event| !matches!(event.kind, StepEventKind::BombDetonated(_)))
        );
    }

    move_cells(&mut game, Direction::West, 1);
    assert!(!game.actors[0].bomb_grace.contains(&mine.id()));
    while game.tick().0 <= mine.placed_at().0 + 183 {
        game.step(&[]).unwrap();
    }
    assert_eq!(game.world.bombs.get(&mine.id()), Some(&mine));

    game.step(&input(InputAction::Press(Direction::East)))
        .unwrap();
    let mut detonation = None;
    for _ in 0..20 {
        let report = game.step(&[]).unwrap();
        if report.events.iter().any(|event| matches!(event.kind, StepEventKind::BombDetonated(view) if view.id == mine.id())) {
            detonation = Some(report);
            break;
        }
    }
    let report = detonation.expect("ordinary re-entry must detonate the seeded mine");
    assert!(!game.actors[0].alive());
    assert!(game.world.hazards.contains_key(&mine.cell()));
    assert!(
        report
            .events
            .iter()
            .all(|event| !matches!(event.kind, StepEventKind::BombPlaced(_)))
    );
}

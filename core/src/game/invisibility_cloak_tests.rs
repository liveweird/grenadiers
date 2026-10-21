use std::collections::BTreeMap;

use super::*;
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
fn seed_31_proves_the_entire_reveal_collect_move_and_expiry_route() {
    let generated = generate_expanded_arena([31; 32], EnabledItems::INVISIBILITY_CLOAK);
    let (arena, concealed, _) = generated.into_parts();
    let spawns = arena.spawns();
    let rules = RulesConfig::expanded_v5(
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::INVISIBILITY_CLOAK),
    );
    let mut game = State::new(
        rules,
        World::with_pickups(arena, concealed, BTreeMap::new(), BTreeMap::new()).unwrap(),
        SeatId::ALL.map(|seat| ActorInit {
            id: EntityId(seat.index() as u32 + 10),
            seat,
            position: Position::center(spawns[seat.index()]),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        }),
    )
    .unwrap();

    move_cells(&mut game, Direction::East, 1);
    let placement = game.step(&input(InputAction::PlaceBomb)).unwrap();
    assert!(placement.events.iter().any(|event| {
        event.audience == EventAudience::Both && matches!(event.kind, StepEventKind::BombPlaced(_))
    }));
    move_cells(&mut game, Direction::West, 1);
    move_cells(&mut game, Direction::South, 1);
    while !game.world().bombs().is_empty()
        || game.world().hazards().contains_key(&CellId::new(3, 1))
    {
        game.step(&[]).unwrap();
    }
    move_cells(&mut game, Direction::North, 1);
    game.step(&input(InputAction::Press(Direction::East)))
        .unwrap();
    let mut collected = None;
    for _ in 0..40 {
        let report = game.step(&[]).unwrap();
        if report.events.iter().any(|event| {
            matches!(
                event.kind,
                StepEventKind::PickupCollected {
                    kind: PickupKind::InvisibilityCloak,
                    ..
                }
            )
        }) {
            collected = Some(report);
            break;
        }
    }
    while game.actors()[0].position().x < Position::center(CellId::new(3, 1)).x {
        game.step(&[]).unwrap();
    }
    game.step(&input(InputAction::Release(Direction::East)))
        .unwrap();
    let collected = collected.expect("ordinary movement must collect the seeded cloak");
    let collected_at = collected.tick;
    assert_eq!(
        game.actors()[0].position(),
        Position::center(CellId::new(3, 1))
    );
    assert!(collected.events.iter().any(|event| {
        event.audience == EventAudience::One
            && matches!(
                event.kind,
                StepEventKind::PickupCollected {
                    kind: PickupKind::InvisibilityCloak,
                    ..
                }
            )
    }));
    assert!(
        game.actors()[0]
            .effects()
            .hidden_from_opponents(collected_at)
    );

    let private_placement = game.step(&input(InputAction::PlaceBomb)).unwrap();
    assert!(private_placement.events.iter().any(|event| {
        event.audience == EventAudience::One && matches!(event.kind, StepEventKind::BombPlaced(_))
    }));
    assert_eq!(game.public_view().bombs.len(), 1);
    move_cells(&mut game, Direction::West, 1);
    move_cells(&mut game, Direction::West, 1);
    move_cells(&mut game, Direction::South, 1);
    while !game.world().bombs().is_empty() || !game.world().hazards().is_empty() {
        game.step(&[]).unwrap();
    }
    move_cells(&mut game, Direction::North, 1);
    move_cells(&mut game, Direction::East, 1);
    assert_eq!(
        game.actors()[0].position(),
        Position::center(CellId::new(2, 1))
    );

    let deadline = Tick(collected_at.0 + EffectKind::InvisibilityCloak.duration_ticks());
    while game.tick().0 + 1 < deadline.0 {
        game.step(&[]).unwrap();
    }
    assert!(
        game.actors()[0]
            .effects()
            .hidden_from_opponents(game.tick())
    );
    game.step(&[]).unwrap();
    assert_eq!(game.tick(), deadline);
    assert!(
        !game.actors()[0]
            .effects()
            .hidden_from_opponents(game.tick())
    );
    assert_eq!(
        game.actors()[0].position(),
        Position::center(CellId::new(2, 1))
    );
}

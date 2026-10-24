use std::collections::BTreeMap;

use super::super::{ActorInit, State};
use crate::game::{
    CellId, Direction, EnabledItems, EntityId, Equipment, InputAction, PickupKind, Position,
    RulesConfig, SeatId, SeatInput, SpeedLevel, Tick, World, generate_expanded_arena,
};
use crate::room::RoomSettings;

fn input(action: InputAction) -> [SeatInput; 1] {
    [SeatInput {
        seat: SeatId::One,
        action,
    }]
}

fn assert_no_bomb_lifecycle_event(report: &crate::game::StepReport) {
    assert!(report.events.iter().all(|event| !matches!(
        event.kind,
        crate::game::StepEventKind::BombPlaced(_) | crate::game::StepEventKind::BombDetonated(_)
    )));
}

fn move_cells(game: &mut State, direction: Direction, cells: usize) {
    game.step(&input(InputAction::Press(direction))).unwrap();
    for _ in 1..cells * 20 {
        game.step(&[]).unwrap();
    }
    game.step(&input(InputAction::Release(direction))).unwrap();
}

fn wait_until_clear(game: &mut State, cell: CellId) {
    for _ in 0..400 {
        if game.world.bombs.is_empty() && !game.world.hazards.contains_key(&cell) {
            return;
        }
        game.step(&[]).unwrap();
    }
    panic!("ordinary-input bomb and flame wait exceeded 400 updates");
}

fn generated_state() -> State {
    let generated = generate_expanded_arena([31; 32], EnabledItems::KICK);
    assert_eq!(
        generated.concealed_pickups().get(&CellId::new(3, 1)),
        Some(&PickupKind::Kick)
    );
    assert!(generated.arena().blocks().contains(&CellId::new(3, 2)));
    assert!(generated.arena().blocks().contains(&CellId::new(4, 1)));
    let (arena, concealed, _) = generated.into_parts();
    let spawns = arena.spawns();
    let settings = RoomSettings::new(SpeedLevel::Base, 256, 3, 300)
        .unwrap()
        .with_enabled_items(EnabledItems::KICK);
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

#[test]
fn seed_31_ordinary_input_route_proves_pickup_corridor_kick_cadence_escape_and_fuse() {
    let mut game = generated_state();
    let pickup = CellId::new(3, 1);

    // Reveal and collect the sole Kick pickup without changing generated state directly.
    move_cells(&mut game, Direction::East, 1);
    game.step(&input(InputAction::PlaceBomb)).unwrap();
    move_cells(&mut game, Direction::West, 1);
    move_cells(&mut game, Direction::South, 1);
    wait_until_clear(&mut game, pickup);
    move_cells(&mut game, Direction::North, 1);
    move_cells(&mut game, Direction::East, 2);
    assert_eq!(game.actors[0].position, Position::center(pickup));
    assert_eq!(game.actors[0].inventory.equipment(), Some(Equipment::Kick));

    // A preparatory ordinary bomb opens both cells needed by the deterministic escape route.
    game.step(&input(InputAction::PlaceBomb)).unwrap();
    move_cells(&mut game, Direction::West, 2);
    move_cells(&mut game, Direction::South, 1);
    wait_until_clear(&mut game, pickup);
    for cell in [CellId::new(3, 2), CellId::new(4, 1)] {
        assert!(!game.world.arena.blocks().contains(&cell));
    }

    move_cells(&mut game, Direction::North, 1);
    move_cells(&mut game, Direction::East, 2);
    game.step(&input(InputAction::PlaceBomb)).unwrap();
    let demo = *game.world.bombs.values().next().unwrap();
    assert_eq!(demo.cell(), pickup);
    assert_eq!(demo.detonates_at(), Some(Tick(game.tick.0 + 300)));
    move_cells(&mut game, Direction::East, 1);

    game.step(&input(InputAction::Press(Direction::West)))
        .unwrap();
    for _ in 0..40 {
        if game.world.bombs[&demo.id()].has_been_kicked() {
            break;
        }
        game.step(&[]).unwrap();
    }
    assert!(game.world.bombs[&demo.id()].has_been_kicked());
    assert_eq!(game.world.bombs[&demo.id()].cell(), CellId::new(3, 1));
    let first_hop = game
        .step(&input(InputAction::Release(Direction::West)))
        .unwrap();
    assert_no_bomb_lifecycle_event(&first_hop);
    assert_eq!(game.world.bombs[&demo.id()].cell(), CellId::new(2, 1));
    for _ in 0..10 {
        let report = game.step(&[]).unwrap();
        assert_no_bomb_lifecycle_event(&report);
    }
    assert_eq!(game.world.bombs[&demo.id()].cell(), CellId::new(1, 1));
    for _ in 0..10 {
        let report = game.step(&[]).unwrap();
        assert_no_bomb_lifecycle_event(&report);
    }
    assert_eq!(game.world.bombs[&demo.id()].motion(), None);

    move_cells(&mut game, Direction::West, 1);
    assert_eq!(game.actors[0].position.cell(), Some(CellId::new(3, 1)));
    game.step(&input(InputAction::Press(Direction::East)))
        .unwrap();
    for _ in 0..10 {
        if (game.actors[0].position.x - Position::center(CellId::new(3, 1)).x).abs() <= 256 {
            break;
        }
        game.step(&[]).unwrap();
    }
    game.step(&input(InputAction::Release(Direction::East)))
        .unwrap();
    game.step(&input(InputAction::Press(Direction::South)))
        .unwrap();
    for _ in 0..40 {
        if game.actors[0].position.y >= Position::center(CellId::new(3, 2)).y {
            break;
        }
        game.step(&[]).unwrap();
    }
    game.step(&input(InputAction::Release(Direction::South)))
        .unwrap();
    assert_eq!(game.actors[0].position.cell(), Some(CellId::new(3, 2)));
    let mut detonation = None;
    for _ in 0..301 {
        if !game.world.bombs.contains_key(&demo.id()) {
            break;
        }
        let report = game.step(&[]).unwrap();
        detonation = detonation.or_else(|| {
            report.events.iter().find_map(|event| match event.kind {
                crate::game::StepEventKind::BombDetonated(bomb) if bomb.id == demo.id() => {
                    Some((event.tick, bomb))
                }
                _ => None,
            })
        });
    }
    let (tick, detonated) = detonation.unwrap();
    assert_eq!(tick, demo.detonates_at().unwrap());
    assert_eq!(detonated.cell, CellId::new(1, 1));
    assert_eq!(detonated.owner, demo.owner());
    assert_eq!(detonated.range, demo.range());
    assert!(game.actors.iter().all(|actor| actor.alive));
}

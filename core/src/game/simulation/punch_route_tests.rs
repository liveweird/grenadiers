use std::collections::BTreeMap;

use super::super::{ActorInit, State};
use crate::game::{
    CellId, Direction, EnabledItems, EntityId, Equipment, InputAction, PickupKind, Position,
    RulesConfig, SeatId, SeatInput, SpeedLevel, StepEventKind, Tick, World,
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
    let generated = generate_expanded_arena([31; 32], EnabledItems::PUNCH);
    assert_eq!(
        generated.concealed_pickups().get(&CellId::new(3, 1)),
        Some(&PickupKind::Punch)
    );
    let (arena, concealed, _) = generated.into_parts();
    let spawns = arena.spawns();
    let settings = RoomSettings::new(SpeedLevel::Base, 256, 3, 300)
        .unwrap()
        .with_enabled_items(EnabledItems::PUNCH);
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
fn seed_31_ordinary_input_route_reveals_collects_and_punches_with_captured_fuse() {
    let mut game = generated_state();
    let pickup = CellId::new(3, 1);

    move_cells(&mut game, Direction::East, 1);
    game.step(&input(InputAction::PlaceBomb)).unwrap();
    move_cells(&mut game, Direction::West, 1);
    move_cells(&mut game, Direction::South, 1);
    wait_until_clear(&mut game, pickup);
    move_cells(&mut game, Direction::North, 1);
    move_cells(&mut game, Direction::East, 2);
    assert_eq!(game.actors[0].position, Position::center(pickup));
    assert_eq!(
        game.actors[0].inventory.equipment(),
        Some(Equipment::Punch)
    );

    game.step(&input(InputAction::PlaceBomb)).unwrap();
    move_cells(&mut game, Direction::West, 2);
    move_cells(&mut game, Direction::South, 1);
    wait_until_clear(&mut game, pickup);
    for cell in [CellId::new(3, 2), CellId::new(4, 1)] {
        assert!(!game.world.arena.blocks().contains(&cell));
    }

    move_cells(&mut game, Direction::North, 1);
    move_cells(&mut game, Direction::East, 3);
    game.step(&input(InputAction::PlaceBomb)).unwrap();
    move_cells(&mut game, Direction::West, 1);
    move_cells(&mut game, Direction::South, 1);
    wait_until_clear(&mut game, CellId::new(4, 1));
    assert!(!game.world.arena.blocks().contains(&CellId::new(5, 1)));

    move_cells(&mut game, Direction::North, 1);
    move_cells(&mut game, Direction::East, 1);
    game.step(&input(InputAction::PlaceBomb)).unwrap();
    let demo = *game.world.bombs.values().next().unwrap();
    assert_eq!(demo.cell(), CellId::new(4, 1));
    assert_eq!(demo.detonates_at(), Some(Tick(game.tick.0 + 300)));
    move_cells(&mut game, Direction::East, 1);
    game.step(&input(InputAction::Press(Direction::West)))
        .unwrap();
    game.step(&input(InputAction::Release(Direction::West)))
        .unwrap();
    assert_eq!(game.actors[0].heading(), Direction::West);

    let punch = game.step(&input(InputAction::UseEquipment)).unwrap();
    assert!(punch.events.iter().all(|event| !matches!(
        event.kind,
        StepEventKind::BombPlaced(_) | StepEventKind::BombDetonated(_)
    )));
    let moved = game.world.bombs[&demo.id()];
    assert_eq!(moved.id(), demo.id());
    assert_eq!(moved.cell(), CellId::new(1, 1));
    assert_eq!(moved.owner(), demo.owner());
    assert_eq!(moved.range(), demo.range());
    assert_eq!(moved.detonates_at(), demo.detonates_at());
    assert!(moved.has_been_punched());
    assert_eq!(moved.motion(), None);

    move_cells(&mut game, Direction::West, 2);
    move_cells(&mut game, Direction::South, 1);
    let mut detonation = None;
    for _ in 0..301 {
        if !game.world.bombs.contains_key(&demo.id()) {
            break;
        }
        let report = game.step(&[]).unwrap();
        detonation = detonation.or_else(|| {
            report.events.iter().find_map(|event| match event.kind {
                StepEventKind::BombDetonated(bomb) if bomb.id == demo.id() => {
                    Some((event.tick, bomb, event.bomb_has_been_punched))
                }
                _ => None,
            })
        });
    }
    let (tick, detonated, punched) = detonation.unwrap();
    assert_eq!(tick, demo.detonates_at().unwrap());
    assert_eq!(detonated.cell, CellId::new(1, 1));
    assert!(punched);
    assert!(game.actors.iter().all(|actor| actor.alive));
}

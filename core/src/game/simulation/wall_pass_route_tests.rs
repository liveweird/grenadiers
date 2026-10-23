use std::collections::BTreeMap;

use super::{ActorInit, State};
use crate::game::{
    CellId, Direction, EnabledItems, EntityId, Equipment, InputAction, PickupKind, Position,
    RulesConfig, SeatId, SeatInput, SpeedLevel, World, generate_expanded_arena, movement,
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

fn generated_state() -> State {
    let generated = generate_expanded_arena([31; 32], EnabledItems::WALL_PASS);
    assert_eq!(
        generated.concealed_pickups().get(&CellId::new(3, 1)),
        Some(&PickupKind::WallPass)
    );
    assert!(generated.arena().blocks().contains(&CellId::new(3, 2)));
    let (arena, concealed, _) = generated.into_parts();
    let spawns = arena.spawns();
    State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::WALL_PASS)),
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
fn seed_31_ordinary_inputs_prove_reveal_pass_placement_clamp_resume_and_exit() {
    let mut game = generated_state();
    let pickup = CellId::new(3, 1);
    let passage = CellId::new(3, 2);

    move_cells(&mut game, Direction::East, 1);
    game.step(&input(InputAction::PlaceBomb)).unwrap();
    move_cells(&mut game, Direction::West, 1);
    move_cells(&mut game, Direction::South, 1);
    while !game.world.bombs.is_empty() || game.world.hazards.contains_key(&pickup) {
        game.step(&[]).unwrap();
    }
    move_cells(&mut game, Direction::North, 1);

    game.step(&input(InputAction::Press(Direction::East)))
        .unwrap();
    let mut collected = false;
    while game.actors[0].position.x < Position::center(pickup).x {
        let report = game.step(&[]).unwrap();
        collected |= report.events.iter().any(|event| {
            matches!(
                event.kind,
                crate::game::StepEventKind::PickupCollected {
                    kind: PickupKind::WallPass,
                    ..
                }
            )
        });
    }
    game.step(&input(InputAction::Release(Direction::East)))
        .unwrap();
    assert!(collected);
    assert_eq!(game.actors[0].position, Position::center(pickup));
    assert_eq!(
        game.actors[0].inventory.equipment(),
        Some(Equipment::WallPass)
    );

    move_cells(&mut game, Direction::South, 1);
    assert_eq!(game.actors[0].position, Position::center(passage));
    assert!(game.world.arena.blocks().contains(&passage));

    let before_id = game.next_entity_id;
    let before_inventory = game.actors[0].inventory;
    game.step(&input(InputAction::PlaceBomb)).unwrap();
    assert!(game.world.bombs.is_empty());
    assert_eq!(game.next_entity_id, before_id);
    assert_eq!(game.actors[0].inventory, before_inventory);

    move_cells(&mut game, Direction::West, 1);
    assert_eq!(game.actors[0].position.x, 3 * 1024 + 320);
    assert_eq!(game.actors[0].position.y, Position::center(passage).y);

    let mut resumed = game.clone();
    assert_eq!(resumed, game);
    assert_eq!(
        resumed.actors[0].inventory.equipment(),
        Some(Equipment::WallPass)
    );
    resumed
        .step(&input(InputAction::Press(Direction::North)))
        .unwrap();
    for _ in 0..40 {
        if !movement::position_overlaps_cell(resumed.actors[0].position, passage) {
            break;
        }
        resumed.step(&[]).unwrap();
    }
    resumed
        .step(&input(InputAction::Release(Direction::North)))
        .unwrap();
    assert!(!movement::position_overlaps_cell(
        resumed.actors[0].position,
        passage
    ));
    assert_eq!(resumed.actors[0].position.x, Position::center(passage).x);
    assert_eq!(
        resumed.actors[0].inventory.equipment(),
        Some(Equipment::WallPass)
    );
}

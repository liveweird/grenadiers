use std::collections::BTreeMap;

use crate::game::{
    ActorInit, CellId, Direction, EnabledItems, EntityId, InputAction, PickupKind, Position,
    RulesConfig, SeatId, SeatInput, SpeedLevel, State, StepEventKind, Tick, World,
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

fn wait_until_clear(game: &mut State, cell: CellId) {
    while !game.world().bombs().is_empty() || game.world().hazards().contains_key(&cell) {
        game.step(&[]).unwrap();
    }
}

fn collect_while_moving(game: &mut State, seat: SeatId, direction: Direction) -> Tick {
    game.step(&[input(seat, InputAction::Press(direction))])
        .unwrap();
    let collected = loop {
        let report = game.step(&[]).unwrap();
        if report.events.iter().any(|event| {
            matches!(
                event.kind,
                StepEventKind::PickupCollected {
                    kind: PickupKind::Teleporter,
                    ..
                }
            )
        }) {
            break report.tick;
        }
    };
    game.step(&[input(seat, InputAction::Release(direction))])
        .unwrap();
    collected
}

fn generated_state() -> State {
    assert_eq!(
        include_str!("../../../tests/fixtures/arena-v2-teleporter.seed").trim(),
        "1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f"
    );
    let generated = generate_expanded_arena([31; 32], EnabledItems::TELEPORTER);
    let (arena, concealed, _) = generated.into_parts();
    let spawns = arena.spawns();
    State::new(
        RulesConfig::expanded_v5(
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::TELEPORTER),
        ),
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
fn seed_31_ordinary_inputs_prove_safe_then_unsafe_teleporter_routes() {
    let mut game = generated_state();

    move_cells(&mut game, SeatId::One, Direction::East, 1);
    game.step(&[input(SeatId::One, InputAction::PlaceBomb)])
        .unwrap();
    move_cells(&mut game, SeatId::One, Direction::West, 1);
    move_cells(&mut game, SeatId::One, Direction::South, 1);
    wait_until_clear(&mut game, CellId::new(3, 1));
    move_cells(&mut game, SeatId::One, Direction::North, 1);

    let first_target = game.actors()[1].position();
    let first_collection = collect_while_moving(&mut game, SeatId::One, Direction::East);
    assert_eq!(game.actors()[0].position(), first_target);
    let successful_tick = game.last_teleport_tick;
    assert_eq!(successful_tick, Some(first_collection));

    move_cells(&mut game, SeatId::One, Direction::West, 1);
    game.step(&[input(SeatId::One, InputAction::PlaceBomb)])
        .unwrap();
    move_cells(&mut game, SeatId::One, Direction::East, 1);
    move_cells(&mut game, SeatId::One, Direction::North, 1);
    wait_until_clear(&mut game, CellId::new(11, 11));

    game.step(&[input(SeatId::Two, InputAction::PlaceBomb)])
        .unwrap();
    let unsafe_bomb = *game.world().bombs().values().next().unwrap();
    assert_eq!(unsafe_bomb.cell(), CellId::new(3, 1));
    game.step(&[input(SeatId::Two, InputAction::Press(Direction::West))])
        .unwrap();
    game.step(&[input(SeatId::Two, InputAction::Release(Direction::West))])
        .unwrap();
    assert!(game.actors()[1].bomb_grace.contains(&unsafe_bomb.id()));

    move_cells(&mut game, SeatId::One, Direction::South, 1);
    let unsafe_source = game.actors()[0].position();
    let unsafe_target = game.actors()[1].position();
    collect_while_moving(&mut game, SeatId::One, Direction::West);
    assert_eq!(
        game.actors()[0].position().cell(),
        Some(CellId::new(11, 11))
    );
    assert_ne!(game.actors()[0].position(), unsafe_target);
    assert_ne!(game.actors()[1].position(), unsafe_source);
    assert_eq!(game.last_teleport_tick, successful_tick);

    move_cells(&mut game, SeatId::Two, Direction::West, 2);
    move_cells(&mut game, SeatId::Two, Direction::South, 1);
    move_cells(&mut game, SeatId::One, Direction::East, 2);
    move_cells(&mut game, SeatId::One, Direction::North, 1);
    while !game.world().bombs().is_empty() || !game.world().hazards().is_empty() {
        game.step(&[]).unwrap();
    }
    assert!(game.actors().iter().all(|actor| actor.alive()));
    assert!(
        game.world()
            .exposed_pickups()
            .values()
            .all(|pickup| pickup.kind() != PickupKind::Teleporter)
    );
}

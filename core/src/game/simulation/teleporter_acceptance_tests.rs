use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;

use super::{ActorInit, RoundResult, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    BombTrigger, CellId, Direction, EffectKind, EnabledItems, EntityId, EventAudience, InputAction,
    PickupKind, Position, RoundAssignment, RulesConfig, SeatId, SeatInput, SpeedLevel,
    StepEventKind, Tick, World, fixed_arena,
};
use crate::room::RoomSettings;

fn actor(id: u32, seat: SeatId, position: Position) -> ActorInit {
    ActorInit {
        id: EntityId(id),
        seat,
        position,
        heading: Direction::East,
        speed: SpeedLevel::Base,
    }
}

fn state_with(
    positions: [Position; 2],
    pickups: BTreeMap<CellId, PickupKind>,
    assignment: RoundAssignment,
) -> State {
    State::new_at_tick_with_assignment(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL)),
        World::with_pickups(fixed_arena(), BTreeMap::new(), BTreeMap::new(), pickups).unwrap(),
        [
            actor(10, SeatId::One, positions[0]),
            actor(20, SeatId::Two, positions[1]),
        ],
        Tick(0),
        assignment,
    )
    .unwrap()
}

fn centered(cell: CellId) -> Position {
    Position::center(cell)
}

#[test]
fn same_tick_source_bomb_makes_only_the_second_destination_unsafe() {
    let cells = [CellId::new(1, 1), CellId::new(13, 11)];
    let positions = cells.map(centered);
    let mut game = state_with(
        positions,
        BTreeMap::from([(cells[0], PickupKind::Teleporter)]),
        RoundAssignment::first(),
    );

    let report = game
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();

    assert_eq!(
        game.actors.each_ref().map(|actor| actor.position),
        positions
    );
    assert_eq!(game.last_teleport_tick(), None);
    assert_eq!(game.world.bombs.len(), 1);
    let id = game.world.bombs.keys().next().copied().unwrap();
    assert!(game.actors[0].bomb_grace.contains(&id));
    assert!(!game.actors[1].bomb_grace.contains(&id));
    assert!(matches!(
        report.events.as_slice(),
        [
            crate::game::StepEvent {
                kind: StepEventKind::BombPlaced(_),
                ..
            },
            crate::game::StepEvent {
                kind: StepEventKind::PickupCollected {
                    kind: PickupKind::Teleporter,
                    ..
                },
                ..
            }
        ]
    ));
}

#[test]
fn same_tick_final_fire_rejects_even_when_shield_keeps_the_target_alive() {
    let one = CellId::new(1, 1);
    let two = CellId::new(5, 5);
    let positions = [centered(one), centered(two)];
    let mut game = state_with(
        positions,
        BTreeMap::from([(one, PickupKind::Teleporter)]),
        RoundAssignment::first(),
    );
    game.apply_test_effect(SeatId::Two, EffectKind::Shield, Tick(0));
    game.insert_test_bomb(
        EntityId(30),
        CellId::new(3, 5),
        EntityId(10),
        2,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(1),
        },
    )
    .unwrap();

    game.step(&[]).unwrap();

    assert!(game.actors[1].alive);
    assert!(game.world.hazards.contains_key(&two));
    assert_eq!(
        game.actors.each_ref().map(|actor| actor.position),
        positions
    );
    assert_eq!(game.last_teleport_tick(), None);
}

#[test]
fn tangency_is_safe_and_intermediate_mines_and_fire_are_not_swept() {
    let one_cell = CellId::new(1, 1);
    let tangent = Position {
        x: 2 * 1024 - 320,
        y: 2 * 1024 + 512,
    };
    let mut game = state_with(
        [centered(one_cell), tangent],
        BTreeMap::from([(one_cell, PickupKind::Teleporter)]),
        RoundAssignment::first(),
    );
    game.insert_test_bomb(
        EntityId(30),
        CellId::new(7, 5),
        EntityId(10),
        2,
        Tick(0),
        BombTrigger::Contact,
    )
    .unwrap();
    game.insert_test_flame(CellId::new(8, 5), Tick(100));

    game.step(&[]).unwrap();

    assert_eq!(
        game.actors.each_ref().map(|actor| actor.position),
        [tangent, centered(one_cell)]
    );
    assert_eq!(game.last_teleport_tick(), Some(Tick(1)));
    assert!(game.world.bombs.contains_key(&EntityId(30)));
    assert!(game.actors.iter().all(|actor| actor.alive));
}

fn arena_with_block(block: CellId) -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row * ARENA_WIDTH + column)] = Terrain::Wall;
            }
        }
    }
    Arena::new(
        terrain,
        BTreeSet::from([block]),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap()
}

#[test]
fn soft_blocks_and_out_of_bounds_destinations_reject_without_half_mutation() {
    let block = CellId::new(5, 5);
    let one = centered(CellId::new(1, 1));
    let safe = centered(CellId::new(13, 11));
    let mut game = State::new(
        RulesConfig::expanded_v5(
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::TELEPORTER),
        ),
        World::new(arena_with_block(block)),
        [actor(10, SeatId::One, one), actor(20, SeatId::Two, safe)],
    )
    .unwrap();

    for unsafe_position in [centered(block), Position { x: 319, y: 512 }] {
        game.actors[1].position = unsafe_position;
        let before = game.actors.clone();
        game.commit_teleport([true, false], Tick(1));
        assert_eq!(game.actors, before);
        assert_eq!(game.last_teleport_tick(), None);
    }
}

#[test]
fn simultaneous_pickups_use_both_assignment_parities_and_one_attempt() {
    let cells = [CellId::new(1, 1), CellId::new(13, 11)];
    let positions = cells.map(centered);
    for round in [1, 2] {
        let mut game = state_with(
            positions,
            BTreeMap::from([
                (cells[0], PickupKind::Teleporter),
                (cells[1], PickupKind::Teleporter),
            ]),
            RoundAssignment::new(NonZeroU32::new(round).unwrap()),
        );
        let report = game.step(&[]).unwrap();
        assert_eq!(
            game.actors.each_ref().map(|actor| actor.position),
            [positions[1], positions[0]]
        );
        assert_eq!(game.last_teleport_tick(), Some(Tick(1)));
        assert_eq!(
            report
                .events
                .iter()
                .filter(|event| matches!(
                    event.kind,
                    StepEventKind::PickupCollected {
                        kind: PickupKind::Teleporter,
                        ..
                    }
                ))
                .count(),
            2
        );
    }
}

#[test]
fn swap_preserves_held_effects_and_private_collection_while_frozen() {
    let cells = [CellId::new(1, 1), CellId::new(13, 11)];
    let positions = cells.map(centered);
    let mut game = state_with(
        positions,
        BTreeMap::from([(cells[0], PickupKind::Teleporter)]),
        RoundAssignment::first(),
    );
    for effect in [
        EffectKind::InvisibilityCloak,
        EffectKind::InvisibleBombs,
        EffectKind::ReverseControls,
        EffectKind::SlowFoot,
    ] {
        game.apply_test_effect(SeatId::One, effect, Tick(0));
    }
    game.apply_test_effect(SeatId::Two, EffectKind::Stopwatch, Tick(0));
    game.actors[0].held.press(Direction::East);

    let report = game.step(&[]).unwrap();

    assert_eq!(game.actors[0].position, positions[1]);
    assert_eq!(game.actors[0].held.active(), Some(Direction::East));
    for effect in [
        EffectKind::InvisibilityCloak,
        EffectKind::InvisibleBombs,
        EffectKind::ReverseControls,
        EffectKind::SlowFoot,
    ] {
        assert!(game.actors[0].effects.active(effect, Tick(1)));
    }
    assert!(report.events.iter().any(|event| {
        event.audience == EventAudience::One
            && matches!(
                event.kind,
                StepEventKind::PickupCollected {
                    kind: PickupKind::Teleporter,
                    ..
                }
            )
    }));
    let after_swap = game.actors[0].position;
    game.step(&[]).unwrap();
    assert_eq!(game.actors[0].position, after_swap);
}

#[test]
fn completed_round_freezes_pickup_and_new_state_resets_marker() {
    let cells = [CellId::new(1, 1), CellId::new(13, 11)];
    let positions = cells.map(centered);
    let mut game = state_with(
        positions,
        BTreeMap::from([(cells[0], PickupKind::Teleporter)]),
        RoundAssignment::first(),
    );
    game.step(&[]).unwrap();
    assert_eq!(game.last_teleport_tick(), Some(Tick(1)));
    game.result = Some(RoundResult::Winner(SeatId::One));
    let frozen = game.clone();
    let report = game.step(&[]).unwrap();
    assert!(report.events.is_empty());
    assert_eq!(game.world, frozen.world);
    assert_eq!(game.actors, frozen.actors);
    assert_eq!(game.result, frozen.result);
    assert_eq!(game.last_teleport_tick(), Some(Tick(1)));

    let replacement = state_with(positions, BTreeMap::new(), RoundAssignment::first());
    assert_eq!(replacement.last_teleport_tick(), None);
}

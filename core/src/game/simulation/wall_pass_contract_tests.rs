use std::collections::{BTreeMap, BTreeSet};

use super::{ActorInit, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    BombTrigger, CellId, DamagePreventionSource, Direction, EnabledItems, EntityId, Equipment,
    InputAction, PickupKind, Position, RulesConfig, SeatId, SeatInput, SpeedLevel, StepEventKind,
    Tick, World, movement,
};
use crate::room::RoomSettings;

fn arena(blocks: BTreeSet<CellId>) -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if row == 0 || column == 0 || row == ARENA_HEIGHT - 1 || column == ARENA_WIDTH - 1 {
                terrain[usize::from(row * ARENA_WIDTH + column)] = Terrain::Wall;
            }
        }
    }
    Arena::new(terrain, blocks, [CellId::new(1, 1), CellId::new(13, 11)]).unwrap()
}

fn actor(id: u32, seat: SeatId, cell: CellId) -> ActorInit {
    ActorInit {
        id: EntityId(id),
        seat,
        position: Position::center(cell),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    }
}

fn state(block: CellId, exposed: BTreeMap<CellId, PickupKind>, enabled: EnabledItems) -> State {
    State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled)),
        World::with_pickups(
            arena(BTreeSet::from([block])),
            BTreeMap::new(),
            BTreeMap::new(),
            exposed,
        )
        .unwrap(),
        [
            actor(1, SeatId::One, CellId::new(4, 5)),
            actor(2, SeatId::Two, CellId::new(13, 11)),
        ],
    )
    .unwrap()
}

fn press(game: &mut State, direction: Direction) -> crate::game::StepReport {
    game.step(&[SeatInput {
        seat: SeatId::One,
        action: InputAction::Press(direction),
    }])
    .unwrap()
}

#[test]
fn heart_collection_replaces_wall_pass_and_grants_only_exit_from_the_actual_overlap() {
    let heart = CellId::new(4, 5);
    let block = CellId::new(5, 5);
    let enabled =
        EnabledItems::from_bits(EnabledItems::WALL_PASS.bits() | EnabledItems::HEART.bits())
            .unwrap();
    let mut game = state(block, BTreeMap::from([(heart, PickupKind::Heart)]), enabled);
    game.actors[0].position.x = 5 * 1024 - 200;
    game.replace_actor_equipment(SeatId::One, Equipment::WallPass);

    let report = game.step(&[]).unwrap();

    assert!(report.events.iter().any(|event| matches!(
        event.kind,
        StepEventKind::PickupCollected {
            kind: PickupKind::Heart,
            ..
        }
    )));
    assert_eq!(game.actors[0].inventory.equipment(), Some(Equipment::Heart));
    assert_eq!(game.actors[0].soft_wall_grace, BTreeSet::from([block]));

    press(&mut game, Direction::West);
    for _ in 0..3 {
        game.step(&[]).unwrap();
    }
    assert!(!movement::position_overlaps_cell(
        game.actors[0].position,
        block
    ));
    assert!(game.actors[0].soft_wall_grace.is_empty());

    press(&mut game, Direction::East);
    for _ in 0..20 {
        game.step(&[]).unwrap();
    }
    assert_eq!(game.actors[0].position.x, 5 * 1024 - 320);
}

#[test]
fn dormant_equipment_transitions_share_the_bounded_seam_without_granting_new_grace() {
    let block = CellId::new(5, 5);
    for replacement in [Equipment::Kick, Equipment::Punch, Equipment::Bracelet] {
        let mut game = state(block, BTreeMap::new(), EnabledItems::WALL_PASS);
        game.actors[0].position = Position::center(block);
        game.replace_actor_equipment(SeatId::One, Equipment::WallPass);
        game.replace_actor_equipment(SeatId::One, Equipment::Heart);
        assert_eq!(game.actors[0].soft_wall_grace, BTreeSet::from([block]));
        game.replace_actor_equipment(SeatId::One, replacement);
        assert_eq!(game.actors[0].soft_wall_grace, BTreeSet::from([block]));
        press(&mut game, Direction::East);
        for _ in 0..20 {
            game.step(&[]).unwrap();
        }
        assert!(game.actors[0].soft_wall_grace.is_empty());
        assert!(!movement::position_overlaps_cell(
            game.actors[0].position,
            block
        ));
    }

    let mut game = state(block, BTreeMap::new(), EnabledItems::WALL_PASS);
    game.actors[0].position = Position::center(block);
    game.replace_actor_equipment(SeatId::One, Equipment::Heart);
    for replacement in [Equipment::Kick, Equipment::Punch, Equipment::Bracelet] {
        game.replace_actor_equipment(SeatId::One, replacement);
        assert!(game.actors[0].soft_wall_grace.is_empty());
    }

    game.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    game.replace_actor_equipment(SeatId::One, Equipment::Heart);
    assert_eq!(game.actors[0].soft_wall_grace, BTreeSet::from([block]));
    game.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    assert!(game.actors[0].soft_wall_grace.is_empty());
}

#[test]
fn an_actual_explosion_prunes_destroyed_grace_before_heart_protects_from_final_fire() {
    let block = CellId::new(5, 5);
    let mut game = state(block, BTreeMap::new(), EnabledItems::ALL);
    game.actors[0].position = Position::center(block);
    game.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    game.replace_actor_equipment(SeatId::One, Equipment::Heart);
    game.insert_test_bomb(
        EntityId(30),
        CellId::new(4, 5),
        EntityId(2),
        2,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(1),
        },
    )
    .unwrap();

    let report = game.step(&[]).unwrap();

    assert!(!game.world.arena.blocks().contains(&block));
    assert!(game.actors[0].soft_wall_grace.is_empty());
    assert!(game.actors[0].alive);
    assert_eq!(game.actors[0].inventory.equipment(), None);
    assert!(report.events.iter().any(|event| matches!(
        event.kind,
        StepEventKind::DamagePrevented {
            source: DamagePreventionSource::Heart,
            ..
        }
    )));
}

#[test]
fn two_segment_turn_retains_then_later_exit_prunes_and_blocks_reentry() {
    let block = CellId::new(8, 8);
    let mut game = state(block, BTreeMap::new(), EnabledItems::WALL_PASS);
    game.actors[0].position = Position {
        x: 8 * 1024 + 512,
        y: 8 * 1024 + 508,
    };
    game.actors[0].heading = Direction::North;
    game.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    game.replace_actor_equipment(SeatId::One, Equipment::Heart);

    let report = press(&mut game, Direction::East);
    let segments = report.traces[0].segments().collect::<Vec<_>>();
    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].start().x, segments[0].end().x);
    assert_eq!(segments[1].start().y, segments[1].end().y);
    assert_eq!(game.actors[0].soft_wall_grace, BTreeSet::from([block]));

    // Exiting the rear neighbor during correction requires over 192 units: the footprint half is
    // 320 and the center is 512 from the face. Even SpeedFour moves at most 86 units per tick, so
    // same-cell exit/reentry cannot occur within one two-segment update. Test the reachable exit.
    for _ in 0..32 {
        if game.actors[0].soft_wall_grace.is_empty() {
            break;
        }
        game.step(&[]).unwrap();
    }
    assert!(
        game.actors[0].soft_wall_grace.is_empty(),
        "eastward movement must leave the graced block within 32 steps"
    );
    press(&mut game, Direction::West);
    for _ in 0..30 {
        game.step(&[]).unwrap();
    }
    assert_eq!(game.actors[0].position.x, 9 * 1024 + 320);
    assert!(!movement::position_overlaps_cell(
        game.actors[0].position,
        block
    ));
}

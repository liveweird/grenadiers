use std::collections::{BTreeMap, BTreeSet};

use super::{ActorInit, State, StateError, StepError};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    CellId, Direction, EnabledItems, EntityId, Equipment, InputAction, PickupKind, Position,
    RulesConfig, SeatId, SeatInput, SpeedLevel, Tick, World, movement,
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

fn state(
    blocks: BTreeSet<CellId>,
    pickups: BTreeMap<CellId, PickupKind>,
    enabled: EnabledItems,
) -> State {
    State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled)),
        World::with_pickups(arena(blocks), BTreeMap::new(), BTreeMap::new(), pickups).unwrap(),
        [
            actor(1, SeatId::One, CellId::new(4, 5)),
            actor(2, SeatId::Two, CellId::new(13, 11)),
        ],
    )
    .unwrap()
}

fn step_empty(state: &mut State, count: usize) {
    for _ in 0..count {
        state.step(&[]).unwrap();
    }
}

#[test]
fn ordinary_collection_equips_wall_pass_and_enters_an_intact_block() {
    let pickup = CellId::new(4, 5);
    let block = CellId::new(5, 5);
    let mut game = state(
        BTreeSet::from([block]),
        BTreeMap::from([(pickup, PickupKind::WallPass)]),
        EnabledItems::WALL_PASS,
    );

    game.step(&[]).unwrap();
    assert_eq!(
        game.actors[0].inventory.equipment(),
        Some(Equipment::WallPass)
    );
    game.step(&[SeatInput {
        seat: SeatId::One,
        action: InputAction::Press(Direction::East),
    }])
    .unwrap();
    step_empty(&mut game, 14);

    assert!(movement::position_overlaps_cell(
        game.actors[0].position,
        block
    ));
    assert!(game.world.arena.blocks().contains(&block));
}

#[test]
fn disabled_wall_pass_pickups_are_rejected_and_equipment_alone_has_no_capability() {
    let pickup = CellId::new(4, 5);
    let block = CellId::new(5, 5);
    let world = World::with_pickups(
        arena(BTreeSet::from([block])),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(pickup, PickupKind::WallPass)]),
    )
    .unwrap();
    assert!(matches!(
        State::new(
            RulesConfig::expanded_v5(RoomSettings::DEFAULT),
            world,
            [
                actor(1, SeatId::One, pickup),
                actor(2, SeatId::Two, CellId::new(13, 11)),
            ],
        ),
        Err(StateError::UnsupportedPickup {
            kind: PickupKind::WallPass,
            ..
        })
    ));

    let mut game = state(
        BTreeSet::from([block]),
        BTreeMap::new(),
        EnabledItems::EMPTY,
    );
    game.actors[0]
        .inventory
        .replace_equipment(Equipment::WallPass);
    game.actors[0].held.press(Direction::East);
    step_empty(&mut game, 30);
    assert_eq!(game.actors[0].position.x, 5 * 1024 - 320);
}

#[test]
fn replacement_captures_exact_overlap_and_each_segment_prevents_reentry() {
    let blocks = BTreeSet::from([
        CellId::new(7, 7),
        CellId::new(8, 7),
        CellId::new(7, 8),
        CellId::new(8, 8),
    ]);
    let mut game = state(blocks, BTreeMap::new(), EnabledItems::WALL_PASS);
    game.actors[0].position = Position {
        x: 8 * 1024,
        y: 8 * 1024,
    };
    game.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    game.replace_actor_equipment(SeatId::One, Equipment::Heart);
    assert_eq!(game.actors[0].soft_wall_grace.len(), 4);

    game.actors[0].held.press(Direction::East);
    step_empty(&mut game, 7);
    assert_eq!(
        game.actors[0].soft_wall_grace,
        BTreeSet::from([CellId::new(8, 7), CellId::new(8, 8)])
    );
    game.actors[0].held.press(Direction::West);
    step_empty(&mut game, 20);
    assert_eq!(game.actors[0].position.x, 8 * 1024 + 320);
    assert_eq!(game.actors[0].soft_wall_grace.len(), 2);
}

#[test]
fn heart_absorption_preserves_exit_grace_and_death_clears_it() {
    let block = CellId::new(5, 5);
    let mut game = state(BTreeSet::from([block]), BTreeMap::new(), EnabledItems::ALL);
    game.actors[0].position = Position::center(block);
    game.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    game.replace_actor_equipment(SeatId::One, Equipment::Heart);
    game.world.hazards.insert(block, Tick(100));

    game.step(&[]).unwrap();
    assert!(game.actors[0].alive);
    assert_eq!(game.actors[0].inventory.equipment(), None);
    assert_eq!(game.actors[0].soft_wall_grace, BTreeSet::from([block]));

    game.actors[0].damage_grace = None;
    game.step(&[]).unwrap();
    assert!(!game.actors[0].alive);
    assert!(game.actors[0].soft_wall_grace.is_empty());
}

#[test]
fn reacquisition_and_destroyed_blocks_clear_obsolete_grace() {
    let block = CellId::new(5, 5);
    let mut game = state(
        BTreeSet::from([block]),
        BTreeMap::new(),
        EnabledItems::WALL_PASS,
    );
    game.actors[0].position = Position::center(block);
    game.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    game.replace_actor_equipment(SeatId::One, Equipment::Heart);
    assert_eq!(game.actors[0].soft_wall_grace, BTreeSet::from([block]));
    game.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    assert!(game.actors[0].soft_wall_grace.is_empty());

    game.replace_actor_equipment(SeatId::One, Equipment::Heart);
    game.world.arena.remove_blocks(&BTreeSet::from([block]));
    movement::prune_soft_wall_grace(&game.world, &mut game.actors[0]);
    assert!(game.actors[0].soft_wall_grace.is_empty());
}

#[test]
fn teleport_uses_each_actors_capability_and_failure_is_atomic() {
    let block = CellId::new(5, 5);
    let mut holder = state(
        BTreeSet::from([block]),
        BTreeMap::new(),
        EnabledItems::WALL_PASS,
    );
    holder.actors[1].position = Position::center(block);
    holder.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    let from = holder.actors[0].position;
    holder.commit_teleport([true, false], Tick(1));
    assert_eq!(holder.actors[0].position, Position::center(block));
    assert_eq!(holder.actors[1].position, from);

    let mut nonholder = state(
        BTreeSet::from([block]),
        BTreeMap::new(),
        EnabledItems::WALL_PASS,
    );
    nonholder.actors[1].position = Position::center(block);
    let before = nonholder.clone();
    nonholder.commit_teleport([true, false], Tick(1));
    assert_eq!(nonholder, before);
}

#[test]
fn teleport_retains_only_own_source_and_destination_grace() {
    let block = CellId::new(5, 5);
    let mut game = state(
        BTreeSet::from([block]),
        BTreeMap::new(),
        EnabledItems::WALL_PASS,
    );
    game.actors[0].position = Position {
        x: 5 * 1024 + 480,
        y: 5 * 1024 + 512,
    };
    game.actors[1].position = Position {
        x: 5 * 1024 + 544,
        y: 5 * 1024 + 512,
    };
    game.actors[0].soft_wall_grace.insert(block);
    game.actors[1].soft_wall_grace.insert(block);
    let positions = game.actors.each_ref().map(|actor| actor.position);

    game.commit_teleport([true, false], Tick(1));

    assert_eq!(game.actors[0].position, positions[1]);
    assert_eq!(game.actors[1].position, positions[0]);
    assert_eq!(game.actors[0].soft_wall_grace, BTreeSet::from([block]));
    assert_eq!(game.actors[1].soft_wall_grace, BTreeSet::from([block]));
}

#[test]
fn hard_walls_and_solid_bombs_remain_blocking_for_wall_pass() {
    let block = CellId::new(5, 5);
    let mut game = state(
        BTreeSet::from([block]),
        BTreeMap::new(),
        EnabledItems::WALL_PASS,
    );
    game.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    assert!(!movement::position_is_free(
        &game.world,
        Position { x: 320, y: 512 },
        &BTreeSet::new(),
        true,
        &BTreeSet::new(),
    ));
    game.insert_test_bomb(
        EntityId(30),
        CellId::new(6, 5),
        EntityId(2),
        1,
        Tick(0),
        crate::game::BombTrigger::Timed {
            detonates_at: Tick(100),
        },
    )
    .unwrap();
    assert!(!movement::position_is_free(
        &game.world,
        Position::center(CellId::new(6, 5)),
        &BTreeSet::new(),
        true,
        &BTreeSet::new(),
    ));
}

#[test]
fn late_timed_pickup_overflow_rolls_back_equipment_and_grace_transition() {
    let block = CellId::new(5, 5);
    let heart = CellId::new(4, 5);
    let shield = CellId::new(13, 11);
    let rules =
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL));
    let mut game = State::new_at_tick(
        rules,
        World::with_pickups(
            arena(BTreeSet::from([block])),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(heart, PickupKind::Heart), (shield, PickupKind::Shield)]),
        )
        .unwrap(),
        [actor(1, SeatId::One, heart), actor(2, SeatId::Two, shield)],
        Tick(u64::MAX - 1),
    )
    .unwrap();
    game.actors[0].position.x = 5 * 1024 - 200;
    game.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    let before = game.clone();

    assert!(matches!(
        game.step(&[]),
        Err(StepError::EffectExpiryOverflow(
            crate::game::EffectKind::Shield
        ))
    ));
    assert_eq!(game, before);
    assert_eq!(
        game.actors[0].inventory.equipment(),
        Some(Equipment::WallPass)
    );
    assert!(game.actors[0].soft_wall_grace.is_empty());
}

#[test]
fn inside_block_placement_retains_resources_but_remote_priority_still_triggers() {
    let block = CellId::new(5, 5);
    let enabled =
        EnabledItems::from_bits(EnabledItems::WALL_PASS.bits() | EnabledItems::REMOTE_BOMB.bits())
            .unwrap();
    let mut game = state(BTreeSet::from([block]), BTreeMap::new(), enabled);
    game.actors[0].position = Position::center(block);
    game.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    game.actors[0]
        .inventory
        .replace_weapon(crate::game::SpecialWeapon::Remote);
    let before_id = game.next_entity_id;

    game.step(&[SeatInput {
        seat: SeatId::One,
        action: InputAction::PlaceBomb,
    }])
    .unwrap();
    assert!(game.world.bombs.is_empty());
    assert_eq!(game.next_entity_id, before_id);
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(crate::game::SpecialWeapon::Remote)
    );

    game.insert_test_bomb(
        EntityId(30),
        CellId::new(10, 5),
        EntityId(1),
        1,
        game.tick,
        crate::game::BombTrigger::Remote,
    )
    .unwrap();
    game.step(&[SeatInput {
        seat: SeatId::One,
        action: InputAction::PlaceBomb,
    }])
    .unwrap();
    assert!(!game.world.bombs.contains_key(&EntityId(30)));
    assert_eq!(
        game.actors[0].inventory.weapon(),
        Some(crate::game::SpecialWeapon::Remote)
    );

    let enabled =
        EnabledItems::from_bits(EnabledItems::WALL_PASS.bits() | EnabledItems::DIARRHEA.bits())
            .unwrap();
    let mut diarrhea = state(BTreeSet::from([block]), BTreeMap::new(), enabled);
    diarrhea.actors[0].position = Position::center(block);
    diarrhea.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    diarrhea.apply_test_effect(SeatId::One, crate::game::EffectKind::Diarrhea, Tick(0));
    let before_id = diarrhea.next_entity_id;
    diarrhea.step(&[]).unwrap();
    assert!(diarrhea.world.bombs.is_empty());
    assert_eq!(diarrhea.next_entity_id, before_id);
}

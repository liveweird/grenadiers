use std::collections::{BTreeMap, BTreeSet};

use super::super::{ActorInit, InputAction, SeatInput, State, StepError};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::bombs::{Bomb, BombMotion, BombTrigger};
use crate::game::effects::EffectKind;
use crate::game::{
    CellId, Direction, EnabledItems, EntityId, Equipment, PickupKind, Position, RulesConfig,
    SeatId, SpecialWeapon, SpeedLevel, Tick, World,
};
use crate::room::RoomSettings;

fn arena() -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    Arena::new(
        terrain,
        BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap()
}

fn actors() -> [ActorInit; 2] {
    [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(CellId::new(1, 1)),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        },
        ActorInit {
            id: EntityId(2),
            seat: SeatId::Two,
            position: Position::center(CellId::new(13, 11)),
            heading: Direction::West,
            speed: SpeedLevel::Base,
        },
    ]
}

fn enabled() -> EnabledItems {
    EnabledItems::from_bits(
        EnabledItems::KICK.bits() | EnabledItems::REMOTE_BOMB.bits() | EnabledItems::SHIELD.bits(),
    )
    .unwrap()
}

fn rules() -> RulesConfig {
    RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled()))
}

fn moving_remote(owner: EntityId) -> Bomb {
    let mut bomb = Bomb::new(
        EntityId(9),
        CellId::new(5, 5),
        owner,
        2,
        Tick(0),
        BombTrigger::Remote,
    )
    .unwrap();
    bomb.restore_kick_history(
        Some(BombMotion {
            direction: Direction::East,
            countdown: 0,
        }),
        true,
    );
    bomb
}

fn world_with_bomb(bomb: Bomb) -> World {
    let mut world = World::new(arena());
    world.bombs.insert(bomb.id(), bomb);
    world
}

#[test]
fn late_shield_overflow_rolls_back_kick_motion_grace_inputs_placement_and_allocator() {
    let pickup_cell = CellId::new(1, 1);
    let mut world = World::with_pickups(
        arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(pickup_cell, PickupKind::Shield)]),
    )
    .unwrap();
    let bomb = moving_remote(EntityId(2));
    world.bombs.insert(bomb.id(), bomb);
    let mut state = State::new_at_tick(rules(), world, actors(), Tick(u64::MAX - 300)).unwrap();
    state.actors[0].bomb_grace.insert(EntityId(9));
    state.actors[0]
        .inventory
        .replace_weapon(SpecialWeapon::Remote);
    let before = state.clone();
    let inputs = [
        SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        },
        SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        },
    ];

    let mut control = before.clone();
    control.world.pickups.exposed.clear();
    control.step(&inputs).unwrap();
    let moved = control.world.bombs[&EntityId(9)];
    assert_eq!(moved.cell(), CellId::new(6, 5));
    assert_eq!(moved.motion().unwrap().countdown, 9);
    assert!(moved.has_been_kicked());
    assert!(!control.actors[0].bomb_grace.contains(&EntityId(9)));
    assert_eq!(control.actors[0].held.active(), Some(Direction::East));
    assert_eq!(control.tick, Tick(u64::MAX - 299));
    assert_eq!(control.next_entity_id, 11);
    assert!(control.world.bombs.contains_key(&EntityId(10)));

    assert_eq!(
        state.step(&inputs),
        Err(StepError::EffectExpiryOverflow(EffectKind::Shield))
    );
    assert_eq!(state, before);
}

#[test]
fn a_terminal_result_freezes_an_active_moving_bomb() {
    let mut world = world_with_bomb(moving_remote(EntityId(2)));
    world.hazards.insert(CellId::new(1, 1), Tick(100));
    let mut state = State::new(rules(), world, actors()).unwrap();

    state.step(&[]).unwrap();
    assert!(state.result().is_some());
    let frozen_world = state.world.clone();
    let frozen_actors = state.actors.clone();
    let frozen_updates = state.running_updates;
    let frozen_allocator = state.next_entity_id;
    let frozen_result = state.result;
    let frozen_motion = state.world.bombs[&EntityId(9)].motion();

    let report = state
        .step(&[SeatInput {
            seat: SeatId::Two,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    assert!(report.events.is_empty());
    assert_eq!(state.world, frozen_world);
    assert_eq!(state.actors, frozen_actors);
    assert_eq!(state.running_updates, frozen_updates);
    assert_eq!(state.next_entity_id, frozen_allocator);
    assert_eq!(state.result, frozen_result);
    assert_eq!(state.world.bombs[&EntityId(9)].motion(), frozen_motion);
}

#[test]
fn bomb_motion_survives_its_kickers_death_and_equipment_cleanup() {
    let mut world = world_with_bomb(moving_remote(EntityId(1)));
    world.hazards.insert(CellId::new(1, 1), Tick(100));
    let mut state = State::new(rules(), world, actors()).unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::Kick);

    state.step(&[]).unwrap();

    assert!(!state.actors[0].alive());
    assert_eq!(state.actors[0].inventory().equipment(), None);
    let bomb = state.world.bombs[&EntityId(9)];
    assert_eq!(bomb.cell(), CellId::new(6, 5));
    assert_eq!(
        bomb.motion(),
        Some(BombMotion {
            direction: Direction::East,
            countdown: 9,
        })
    );
    assert!(bomb.has_been_kicked());
}

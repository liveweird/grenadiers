use std::collections::{BTreeMap, BTreeSet};

use super::{ActorInit, RoundResult, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::input::Direction;
use crate::game::pickups::PickupKind;
use crate::game::rules::RulesConfig;
use crate::game::test_support::add_test_bomb_with_deadline;
use crate::game::types::{CellId, EntityId, Position, SeatId, SpeedLevel, Tick};
use crate::game::world::World;

const TARGET: CellId = CellId::new(5, 5);

fn arena_with_blocks(blocks: BTreeSet<CellId>) -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
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

fn state_in(world: World) -> State {
    State::new(
        RulesConfig::classic_v1(),
        world,
        [
            actor(10, SeatId::One, CellId::new(1, 1)),
            actor(20, SeatId::Two, CellId::new(13, 11)),
        ],
    )
    .unwrap()
}

fn concealed_state() -> State {
    let world = World::with_pickups(
        arena_with_blocks(BTreeSet::from([TARGET])),
        BTreeMap::from([(TARGET, PickupKind::BombCapacity)]),
        BTreeMap::new(),
        BTreeMap::new(),
    )
    .unwrap();
    state_in(world)
}

fn add_bomb(state: &mut State, id: u32, cell: CellId, deadline: u64, range: u16) {
    add_test_bomb_with_deadline(
        state,
        EntityId(id),
        cell,
        EntityId(10),
        range,
        Tick(0),
        Tick(deadline),
    )
    .unwrap();
}

fn step_to(state: &mut State, tick: u64) {
    while state.tick().0 < tick {
        state.step(&[]).unwrap();
    }
}

#[test]
fn revealing_blast_moves_concealed_pickup_to_pending_without_burning_it() {
    let mut state = concealed_state();
    add_bomb(&mut state, 30, CellId::new(3, 5), 1, 2);

    state.step(&[]).unwrap();

    assert!(!state.world().arena().blocks().contains(&TARGET));
    assert!(state.world.pickups.concealed.is_empty());
    assert_eq!(
        state
            .world
            .pickups
            .pending
            .get(&TARGET)
            .map(|item| item.kind()),
        Some(PickupKind::BombCapacity)
    );
    assert!(state.world().exposed_pickups().is_empty());
    assert_eq!(state.world().hazards().get(&TARGET), Some(&Tick(31)));
}

#[test]
fn pending_pickup_reveals_at_the_exclusive_flame_expiry() {
    let mut state = concealed_state();
    add_bomb(&mut state, 30, CellId::new(3, 5), 1, 2);
    state.step(&[]).unwrap();

    step_to(&mut state, 30);
    assert!(state.world().exposed_pickups().is_empty());
    state.step(&[]).unwrap();

    assert!(!state.world().hazards().contains_key(&TARGET));
    assert!(state.world.pickups.pending.is_empty());
    assert_eq!(
        state
            .world()
            .exposed_pickups()
            .get(&TARGET)
            .map(|item| item.kind()),
        Some(PickupKind::BombCapacity)
    );
}

#[test]
fn renewed_overlapping_flame_delays_pending_reveal_to_the_latest_expiry() {
    let mut state = concealed_state();
    add_bomb(&mut state, 30, CellId::new(3, 5), 1, 2);
    add_bomb(&mut state, 31, CellId::new(7, 5), 20, 2);

    step_to(&mut state, 20);
    assert_eq!(state.world().hazards().get(&TARGET), Some(&Tick(50)));
    step_to(&mut state, 49);
    assert!(state.world().exposed_pickups().is_empty());
    assert!(state.world.pickups.pending.contains_key(&TARGET));
    state.step(&[]).unwrap();

    assert!(state.world.pickups.pending.is_empty());
    assert!(state.world().exposed_pickups().contains_key(&TARGET));
}

#[test]
fn reveal_followed_by_a_new_blast_in_the_same_tick_burns_the_pickup() {
    let mut state = concealed_state();
    add_bomb(&mut state, 30, CellId::new(3, 5), 1, 2);
    add_bomb(&mut state, 31, CellId::new(7, 5), 31, 2);

    step_to(&mut state, 31);

    assert!(state.world.pickups.concealed.is_empty());
    assert!(state.world.pickups.pending.is_empty());
    assert!(state.world().exposed_pickups().is_empty());
    assert_eq!(state.world().hazards().get(&TARGET), Some(&Tick(61)));
}

#[test]
fn exposed_pickup_burns_without_shielding_cells_behind_it() {
    let world = World::with_pickups(
        arena_with_blocks(BTreeSet::new()),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(TARGET, PickupKind::MovementSpeed)]),
    )
    .unwrap();
    let mut state = state_in(world);
    add_bomb(&mut state, 30, CellId::new(3, 5), 1, 4);

    state.step(&[]).unwrap();

    assert!(state.world().exposed_pickups().is_empty());
    assert!(state.world().hazards().contains_key(&CellId::new(6, 5)));
    assert!(state.world().hazards().contains_key(&CellId::new(7, 5)));
}

#[test]
fn concealed_and_pending_layers_do_not_change_public_state() {
    let blocked = arena_with_blocks(BTreeSet::from([TARGET]));
    let hidden = state_in(
        World::with_pickups(
            blocked.clone(),
            BTreeMap::from([(TARGET, PickupKind::BlastRange)]),
            BTreeMap::new(),
            BTreeMap::new(),
        )
        .unwrap(),
    );
    let no_hidden = state_in(World::new(blocked));
    assert_eq!(hidden.public_view(), no_hidden.public_view());
    assert!(!format!("{:?}", hidden.world()).contains("BlastRange"));

    let open = arena_with_blocks(BTreeSet::new());
    let pending = state_in(
        World::with_pickups(
            open.clone(),
            BTreeMap::new(),
            BTreeMap::from([(TARGET, PickupKind::BlastRange)]),
            BTreeMap::new(),
        )
        .unwrap(),
    );
    let no_pending = state_in(World::new(open));
    assert_eq!(pending.public_view(), no_pending.public_view());
}

#[test]
fn completed_result_freezes_pending_reveal_and_exposed_burn() {
    let open = arena_with_blocks(BTreeSet::new());
    let mut world = World::with_pickups(
        open,
        BTreeMap::new(),
        BTreeMap::from([(TARGET, PickupKind::BlastRange)]),
        BTreeMap::from([(CellId::new(6, 5), PickupKind::MovementSpeed)]),
    )
    .unwrap();
    world.hazards.insert(TARGET, Tick(2));
    world.hazards.insert(CellId::new(1, 1), Tick(10));
    let mut state = state_in(world);
    add_bomb(&mut state, 30, CellId::new(4, 5), 2, 2);
    state.step(&[]).unwrap();
    assert_eq!(state.result(), Some(RoundResult::Winner(SeatId::Two)));
    let frozen = state.world().clone();

    state.step(&[]).unwrap();

    assert_eq!(state.world(), &frozen);
    assert!(state.world.pickups.pending.contains_key(&TARGET));
    assert!(
        state
            .world()
            .exposed_pickups()
            .contains_key(&CellId::new(6, 5))
    );
}

use std::collections::BTreeSet;

use super::{ActorInit, InputAction, SeatInput, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::fixtures::fixed_arena;
use crate::game::input::Direction;
use crate::game::rules::RulesConfig;
use crate::game::test_support::{add_test_bomb, add_test_bomb_with_deadline};
use crate::game::types::{CellId, EntityId, Position, SeatId, SpeedLevel, Tick};
use crate::game::world::World;

fn open_world() -> World {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    World::new(
        Arena::new(
            terrain,
            BTreeSet::new(),
            [CellId::new(1, 1), CellId::new(13, 11)],
        )
        .unwrap(),
    )
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

fn state_at(one: CellId, two: CellId) -> State {
    State::new(
        RulesConfig::classic_v1(),
        open_world(),
        [actor(10, SeatId::One, one), actor(20, SeatId::Two, two)],
    )
    .unwrap()
}

fn place(seat: SeatId) -> SeatInput {
    SeatInput {
        seat,
        action: InputAction::PlaceBomb,
    }
}

fn step_to(state: &mut State, tick: u64) {
    while state.tick().0 < tick {
        state.step(&[]).unwrap();
    }
}

#[test]
fn ordinary_fuse_detonates_at_exactly_placement_plus_180() {
    let cell = CellId::new(2, 5);
    let mut state = state_at(cell, CellId::new(12, 5));
    state.step(&[place(SeatId::One)]).unwrap();
    assert_eq!(
        state
            .world()
            .bombs()
            .values()
            .next()
            .unwrap()
            .detonates_at(),
        Tick(181)
    );

    step_to(&mut state, 180);
    assert_eq!(state.world().bombs().len(), 1);
    assert!(state.world().hazards().is_empty());
    state.step(&[]).unwrap();
    assert!(state.world().bombs().is_empty());
    assert_eq!(state.world().hazards().get(&cell), Some(&Tick(211)));
}

#[test]
fn overlapping_flames_keep_the_latest_expiry_and_expire_at_the_boundary() {
    let overlap = CellId::new(5, 5);
    let mut state = state_at(CellId::new(1, 1), CellId::new(13, 11));
    add_test_bomb_with_deadline(
        &mut state,
        EntityId(30),
        CellId::new(3, 5),
        EntityId(10),
        2,
        Tick(0),
        Tick(1),
    )
    .unwrap();
    add_test_bomb_with_deadline(
        &mut state,
        EntityId(40),
        CellId::new(7, 5),
        EntityId(20),
        2,
        Tick(0),
        Tick(10),
    )
    .unwrap();

    step_to(&mut state, 10);
    assert_eq!(state.world().hazards().get(&overlap), Some(&Tick(40)));
    assert_eq!(
        state.world().hazards().get(&CellId::new(3, 5)),
        Some(&Tick(31))
    );
    step_to(&mut state, 31);
    assert!(!state.world().hazards().contains_key(&CellId::new(3, 5)));
    assert_eq!(state.world().hazards().get(&overlap), Some(&Tick(40)));
    step_to(&mut state, 39);
    assert!(state.world().hazards().contains_key(&overlap));
    state.step(&[]).unwrap();
    assert!(!state.world().hazards().contains_key(&overlap));
}

#[test]
fn bomb_placed_in_active_flame_detonates_in_the_placement_tick() {
    let cell = CellId::new(4, 5);
    let mut state = state_at(cell, CellId::new(12, 5));
    state.world.hazards.insert(cell, Tick(10));

    state.step(&[place(SeatId::One)]).unwrap();

    assert!(state.world().bombs().is_empty());
    assert_eq!(state.live_bomb_count(SeatId::One), 0);
    assert_eq!(state.world().hazards().get(&cell), Some(&Tick(31)));
}

#[test]
fn newly_placed_bomb_joins_a_same_tick_due_bomb_chain() {
    let placement_cell = CellId::new(5, 5);
    let safe_second_actor = CellId::new(12, 10);
    let mut state = state_at(placement_cell, safe_second_actor);
    add_test_bomb_with_deadline(
        &mut state,
        EntityId(30),
        CellId::new(3, 5),
        EntityId(20),
        2,
        Tick(0),
        Tick(1),
    )
    .unwrap();

    state
        .step(&[place(SeatId::One), place(SeatId::Two)])
        .unwrap();

    assert!(state.world().bombs().is_empty());
    assert_eq!(state.live_bomb_count(SeatId::One), 0);
    assert_eq!(state.live_bomb_count(SeatId::Two), 0);
    assert!(state.world().hazards().contains_key(&CellId::new(5, 7)));
    assert!(!state.world().hazards().contains_key(&safe_second_actor));
}

#[test]
fn capacity_released_by_detonation_is_available_on_the_following_tick() {
    let target = CellId::new(10, 5);
    let mut state = state_at(target, CellId::new(12, 5));
    add_test_bomb_with_deadline(
        &mut state,
        EntityId(30),
        CellId::new(2, 5),
        EntityId(10),
        2,
        Tick(0),
        Tick(1),
    )
    .unwrap();

    state.step(&[place(SeatId::One)]).unwrap();
    assert!(state.world().bombs().is_empty());
    state.step(&[place(SeatId::One)]).unwrap();
    assert_eq!(state.live_bomb_count(SeatId::One), 1);
    assert_eq!(
        state.world().bombs().values().next().unwrap().cell(),
        target
    );
}

#[test]
fn applying_a_plan_removes_hit_blocks_but_preserves_blocks_behind_them() {
    let mut state = State::new(
        RulesConfig::classic_v1(),
        World::new(fixed_arena()),
        [
            actor(10, SeatId::One, CellId::new(1, 1)),
            actor(20, SeatId::Two, CellId::new(13, 11)),
        ],
    )
    .unwrap();
    add_test_bomb_with_deadline(
        &mut state,
        EntityId(30),
        CellId::new(3, 1),
        EntityId(10),
        3,
        Tick(0),
        Tick(1),
    )
    .unwrap();

    state.step(&[]).unwrap();

    assert!(!state.world().arena().blocks().contains(&CellId::new(5, 1)));
    assert!(state.world().arena().blocks().contains(&CellId::new(6, 1)));
    assert_eq!(
        state.world().hazards().get(&CellId::new(5, 1)),
        Some(&Tick(31))
    );
    assert!(!state.world().hazards().contains_key(&CellId::new(6, 1)));
}

#[test]
fn detonation_removes_bomb_grace_from_every_actor() {
    let shared = CellId::new(5, 5);
    let mut state = state_at(shared, shared);
    add_test_bomb_with_deadline(
        &mut state,
        EntityId(30),
        shared,
        EntityId(10),
        2,
        Tick(0),
        Tick(1),
    )
    .unwrap();
    assert!(
        state
            .actors
            .iter()
            .all(|actor| actor.bomb_grace.contains(&EntityId(30)))
    );

    state.step(&[]).unwrap();
    assert!(state.actors.iter().all(|actor| actor.bomb_grace.is_empty()));
}

#[test]
fn final_tick_flame_expiry_uses_a_saturating_terminal_value() {
    let mut state = state_at(CellId::new(2, 5), CellId::new(12, 5));
    add_test_bomb(&mut state, EntityId(30), CellId::new(8, 5), EntityId(10)).unwrap();
    state
        .world
        .bombs
        .get_mut(&EntityId(30))
        .unwrap()
        .detonates_at = Tick(u64::MAX);
    state.tick = Tick(u64::MAX - 1);

    state.step(&[]).unwrap();

    assert_eq!(
        state.world().hazards().get(&CellId::new(8, 5)),
        Some(&Tick(u64::MAX))
    );
}

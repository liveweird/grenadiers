use std::collections::BTreeSet;

use super::{ActorInit, InputAction, MAX_INPUTS_PER_TICK, SeatInput, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::input::Direction;
use crate::game::pickups::{Pickup, PickupKind};
use crate::game::rules::RulesConfig;
use crate::game::test_support::add_test_bomb;
use crate::game::types::{CellId, EntityId, Position, SeatId, SpeedLevel, Tick, UNITS_PER_CELL};
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

fn actor(id: u32, seat: SeatId, position: Position, heading: Direction) -> ActorInit {
    ActorInit {
        id: EntityId(id),
        seat,
        position,
        heading,
        speed: SpeedLevel::Four,
    }
}

fn state_at(one: Position, two: Position) -> State {
    State::new(
        RulesConfig::classic_v1(),
        open_world(),
        [
            actor(10, SeatId::One, one, Direction::East),
            actor(20, SeatId::Two, two, Direction::West),
        ],
    )
    .unwrap()
}

fn input(seat: SeatId, action: InputAction) -> SeatInput {
    SeatInput { seat, action }
}

fn place(seat: SeatId) -> SeatInput {
    input(seat, InputAction::PlaceBomb)
}

#[test]
fn placement_snapshots_owner_range_and_default_deadline() {
    let one = Position::center(CellId::new(2, 5));
    let mut state = state_at(one, Position::center(CellId::new(12, 5)));
    state.step(&[place(SeatId::One)]).unwrap();

    let bomb = *state.world().bombs().values().next().unwrap();
    assert_eq!(bomb.id(), EntityId(0));
    assert_eq!(bomb.cell(), CellId::new(2, 5));
    assert_eq!(bomb.owner(), EntityId(10));
    assert_eq!(bomb.range(), 2);
    assert_eq!(bomb.placed_at(), Tick(1));
    assert_eq!(bomb.detonates_at(), Tick(181));
    assert_eq!(state.actors()[0].bomb_capacity(), 1);
    assert_eq!(state.actors()[0].bomb_range(), 2);
    assert_eq!(state.live_bomb_count(SeatId::One), 1);
}

#[test]
fn movement_precedes_placement_and_exact_boundaries_choose_east_and_south() {
    let one = Position {
        x: 3 * UNITS_PER_CELL - 85,
        y: Position::center(CellId::new(2, 5)).y,
    };
    let two = Position {
        x: Position::center(CellId::new(8, 5)).x,
        y: 6 * UNITS_PER_CELL - 85,
    };
    let mut state = State::new(
        RulesConfig::classic_v1(),
        open_world(),
        [
            actor(10, SeatId::One, one, Direction::East),
            actor(20, SeatId::Two, two, Direction::South),
        ],
    )
    .unwrap();
    state
        .step(&[
            place(SeatId::One),
            input(SeatId::One, InputAction::Press(Direction::East)),
            place(SeatId::Two),
            input(SeatId::Two, InputAction::Press(Direction::South)),
        ])
        .unwrap();

    let cells: BTreeSet<_> = state
        .world()
        .bombs()
        .values()
        .map(|bomb| bomb.cell())
        .collect();
    assert_eq!(
        cells,
        BTreeSet::from([CellId::new(3, 5), CellId::new(8, 6)])
    );
}

#[test]
fn failed_request_consumes_no_id_and_is_not_queued() {
    let one = Position::center(CellId::new(2, 5));
    let mut state = state_at(one, Position::center(CellId::new(12, 5)));
    state.step(&[place(SeatId::One)]).unwrap();
    assert_eq!(state.next_entity_id, 1);

    state.step(&[place(SeatId::One)]).unwrap();
    assert_eq!(state.next_entity_id, 1);
    state.world.bombs.clear();
    state.actors[0].bomb_grace.clear();
    state.step(&[]).unwrap();
    assert!(state.world().bombs().is_empty());
    assert_eq!(state.next_entity_id, 1);
}

#[test]
fn capacity_is_derived_and_pickups_do_not_block_placement() {
    let one = Position::center(CellId::new(2, 5));
    let mut state = state_at(one, Position::center(CellId::new(12, 5)));
    add_test_bomb(&mut state, EntityId(90), CellId::new(8, 5), EntityId(10)).unwrap();
    state
        .world
        .pickups
        .exposed
        .insert(CellId::new(2, 5), Pickup::new(PickupKind::BombCapacity));
    state.step(&[place(SeatId::One)]).unwrap();
    assert_eq!(state.world().bombs().len(), 1);

    state.world.bombs.clear();
    state.step(&[place(SeatId::One)]).unwrap();
    assert_eq!(state.world().bombs().len(), 1);
    assert_eq!(
        state.world().bombs().values().next().unwrap().cell(),
        CellId::new(2, 5)
    );
}

#[test]
fn same_cell_conflict_ignores_an_ineligible_capacity_claimant() {
    let shared = Position::center(CellId::new(5, 5));
    let mut both_eligible = state_at(shared, shared);
    both_eligible
        .step(&[place(SeatId::One), place(SeatId::Two), place(SeatId::One)])
        .unwrap();
    assert!(both_eligible.world().bombs().is_empty());

    let mut one_full = state_at(shared, shared);
    add_test_bomb(&mut one_full, EntityId(90), CellId::new(8, 5), EntityId(10)).unwrap();
    one_full
        .step(&[place(SeatId::One), place(SeatId::Two), place(SeatId::One)])
        .unwrap();
    assert_eq!(one_full.world().bombs().len(), 2);
    assert!(
        one_full
            .world()
            .bombs()
            .values()
            .any(|bomb| bomb.owner() == EntityId(20) && bomb.cell() == CellId::new(5, 5))
    );
}

#[test]
fn distinct_simultaneous_placements_ignore_input_order() {
    let mut forward = state_at(
        Position::center(CellId::new(2, 5)),
        Position::center(CellId::new(12, 5)),
    );
    let mut reverse = forward.clone();
    forward
        .step(&[place(SeatId::One), place(SeatId::Two)])
        .unwrap();
    reverse
        .step(&[place(SeatId::Two), place(SeatId::One)])
        .unwrap();
    assert_eq!(forward, reverse);
}

#[test]
fn duplicate_requests_are_one_shot_and_clear_obeys_batch_order() {
    let one = Position::center(CellId::new(2, 5));
    let two = Position::center(CellId::new(12, 5));
    let mut duplicate = state_at(one, two);
    let requests = vec![place(SeatId::One); MAX_INPUTS_PER_TICK];
    duplicate.step(&requests).unwrap();
    assert_eq!(duplicate.world().bombs().len(), 1);
    assert_eq!(duplicate.next_entity_id, 1);

    let mut cancelled = state_at(one, two);
    cancelled
        .step(&[place(SeatId::One), input(SeatId::One, InputAction::Clear)])
        .unwrap();
    assert!(cancelled.world().bombs().is_empty());
    cancelled
        .step(&[input(SeatId::One, InputAction::Clear), place(SeatId::One)])
        .unwrap();
    assert_eq!(cancelled.world().bombs().len(), 1);
}

#[test]
fn placement_grants_all_overlapping_actors_temporary_grace() {
    let shared = Position::center(CellId::new(5, 5));
    let mut state = state_at(shared, shared);
    state.step(&[place(SeatId::One)]).unwrap();
    state
        .step(&[
            input(SeatId::One, InputAction::Press(Direction::West)),
            input(SeatId::Two, InputAction::Press(Direction::West)),
        ])
        .unwrap();
    for _ in 0..9 {
        state.step(&[]).unwrap();
    }
    state
        .step(&[
            input(SeatId::One, InputAction::Press(Direction::East)),
            input(SeatId::Two, InputAction::Press(Direction::East)),
        ])
        .unwrap();
    for _ in 0..19 {
        state.step(&[]).unwrap();
    }
    for actor in state.actors() {
        assert_eq!(actor.position().x, 5 * UNITS_PER_CELL - 320);
    }
}

#[test]
fn id_and_deadline_exhaustion_reject_without_mutation() {
    let one = Position::center(CellId::new(2, 5));
    let two = Position::center(CellId::new(12, 5));
    let mut no_id = state_at(one, two);
    no_id.next_entity_id = u64::from(u32::MAX) + 1;
    let before = no_id.clone();
    no_id.step(&[place(SeatId::One)]).unwrap();
    assert!(no_id.world().bombs().is_empty());
    assert_eq!(no_id.next_entity_id, before.next_entity_id);

    let mut no_deadline = state_at(one, two);
    no_deadline.tick = Tick(u64::MAX - 180);
    no_deadline.step(&[place(SeatId::One)]).unwrap();
    assert!(no_deadline.world().bombs().is_empty());
}

#[test]
fn generated_ids_skip_actor_ids_and_start_above_fixture_ids() {
    let mut skip = State::new(
        RulesConfig::classic_v1(),
        open_world(),
        [
            actor(
                0,
                SeatId::One,
                Position::center(CellId::new(2, 5)),
                Direction::East,
            ),
            actor(
                1,
                SeatId::Two,
                Position::center(CellId::new(12, 5)),
                Direction::West,
            ),
        ],
    )
    .unwrap();
    skip.step(&[place(SeatId::One)]).unwrap();
    assert!(skip.world().bombs().contains_key(&EntityId(2)));

    let mut seeded = state_at(
        Position::center(CellId::new(2, 5)),
        Position::center(CellId::new(12, 5)),
    );
    add_test_bomb(&mut seeded, EntityId(90), CellId::new(8, 5), EntityId(20)).unwrap();
    seeded.step(&[place(SeatId::One)]).unwrap();
    assert!(seeded.world().bombs().contains_key(&EntityId(91)));
}

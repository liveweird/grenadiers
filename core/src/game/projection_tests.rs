use std::collections::BTreeSet;

use super::{ActorInit, RoundResult, State, TurnAssist};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::fixtures::fixed_arena;
use crate::game::input::Direction;
use crate::game::rules::RulesConfig;
use crate::game::test_support::add_test_bomb_with_deadline;
use crate::game::types::{CellId, EntityId, Position, SeatId, SpeedLevel, Tick};
use crate::game::world::World;

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

#[test]
fn public_view_copies_visible_round_state() {
    let mut state = state_in(World::new(fixed_arena()));
    add_test_bomb_with_deadline(
        &mut state,
        EntityId(30),
        CellId::new(8, 5),
        EntityId(10),
        3,
        Tick(0),
        Tick(99),
    )
    .unwrap();
    state.world.hazards.insert(CellId::new(1, 1), Tick(10));
    state.world.hazards.insert(CellId::new(7, 6), Tick(50));
    state.step(&[]).unwrap();

    let view = state.public_view();

    assert_eq!(view.rules_version, 1);
    assert_eq!(view.tick, Tick(1));
    assert_eq!(view.running_updates, 1);
    assert_eq!(view.remaining_ticks, 10_799);
    assert_eq!(view.arena.width, ARENA_WIDTH);
    assert_eq!(view.arena.height, ARENA_HEIGHT);
    assert_eq!(view.arena.terrain.len(), ARENA_CELL_COUNT);
    assert_eq!(view.arena.terrain[0], Terrain::Wall);
    assert_eq!(
        view.arena.terrain[usize::from(ARENA_WIDTH) + 1],
        Terrain::Floor
    );
    assert_eq!(
        view.arena.blocks,
        state
            .world()
            .arena()
            .blocks()
            .iter()
            .copied()
            .collect::<Vec<_>>()
    );
    assert_eq!(view.actors[0].id, EntityId(10));
    assert_eq!(view.actors[0].seat, SeatId::One);
    assert!(!view.actors[0].alive);
    assert_eq!(view.actors[0].bomb_capacity, 1);
    assert_eq!(view.actors[0].bomb_range, 2);
    assert_eq!(view.actors[0].live_bomb_count, 1);
    assert_eq!(view.actors[1].seat, SeatId::Two);
    assert!(view.actors[1].alive);
    assert_eq!(view.bombs.len(), 1);
    assert_eq!(view.bombs[0].id, EntityId(30));
    assert_eq!(view.bombs[0].cell, CellId::new(8, 5));
    assert_eq!(view.bombs[0].owner, EntityId(10));
    assert_eq!(view.bombs[0].range, 3);
    assert_eq!(view.bombs[0].placed_at, Tick(0));
    assert_eq!(view.bombs[0].detonates_at, Tick(99));
    assert_eq!(view.flames[0].cell, CellId::new(1, 1));
    assert_eq!(view.flames[0].expires_at, Tick(10));
    assert_eq!(view.flames[1].cell, CellId::new(7, 6));
    assert_eq!(view.result, Some(RoundResult::Winner(SeatId::Two)));
}

#[test]
fn projected_collections_have_stable_domain_ordering() {
    let mut state = state_in(open_world());
    add_test_bomb_with_deadline(
        &mut state,
        EntityId(40),
        CellId::new(8, 7),
        EntityId(10),
        2,
        Tick(0),
        Tick(99),
    )
    .unwrap();
    add_test_bomb_with_deadline(
        &mut state,
        EntityId(30),
        CellId::new(6, 7),
        EntityId(20),
        2,
        Tick(0),
        Tick(99),
    )
    .unwrap();
    state.world.hazards.insert(CellId::new(8, 6), Tick(20));
    state.world.hazards.insert(CellId::new(9, 5), Tick(20));

    let view = state.public_view();

    assert_eq!(
        view.bombs.iter().map(|bomb| bomb.id).collect::<Vec<_>>(),
        [EntityId(30), EntityId(40)]
    );
    assert_eq!(
        view.flames
            .iter()
            .map(|flame| flame.cell)
            .collect::<Vec<_>>(),
        [CellId::new(9, 5), CellId::new(8, 6)]
    );
}

#[test]
fn private_simulation_details_do_not_change_the_public_view() {
    let left = state_in(open_world());
    let mut right = left.clone();
    right.actors[0].held.press(Direction::North);
    right.actors[0].residual = 47;
    right.actors[0].pending_turn = Some(TurnAssist {
        direction: Direction::South,
        target: 1_536,
    });
    right.actors[0].bomb_grace.insert(EntityId(999));
    right.next_entity_id = 8_000;

    assert_ne!(left, right);
    assert_eq!(left.public_view(), right.public_view());
}

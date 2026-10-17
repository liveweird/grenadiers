use std::collections::BTreeSet;

use super::{ActorInit, DrawReason, RoundResult, State};
use crate::game::{
    Direction, RoundAssignment, SeatId, SpeedLevel,
    arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain},
    test_support::add_test_bomb_with_deadline,
    types::{CellId, EntityId, Position, Tick},
    world::World,
};

fn stepped(ids: [u32; 3]) -> State {
    let block = CellId::new(6, 5);
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    let arena = Arena::new(
        terrain,
        BTreeSet::from([block]),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap();
    let mut state = State::new_at_tick_with_assignment(
        crate::game::RulesConfig::classic_v1(),
        World::new(arena),
        [
            ActorInit {
                id: EntityId(100),
                seat: SeatId::One,
                position: Position::center(CellId::new(3, 5)),
                heading: Direction::East,
                speed: SpeedLevel::Base,
            },
            ActorInit {
                id: EntityId(200),
                seat: SeatId::Two,
                position: Position::center(CellId::new(5, 7)),
                heading: Direction::West,
                speed: SpeedLevel::Base,
            },
        ],
        Tick(0),
        RoundAssignment::first(),
    )
    .unwrap();
    for (id, cell, owner, deadline) in [
        (ids[0], CellId::new(3, 5), EntityId(100), Tick(1)),
        (ids[1], CellId::new(5, 5), EntityId(100), Tick(999)),
        (ids[2], CellId::new(5, 7), EntityId(200), Tick(1)),
    ] {
        add_test_bomb_with_deadline(&mut state, EntityId(id), cell, owner, 2, Tick(0), deadline)
            .unwrap();
    }
    state.step(&[]).unwrap();
    state
}

#[test]
fn complete_step_outcome_is_invariant_to_bomb_traversal_order() {
    let ascending = stepped([1, 2, 3]);
    let permuted = stepped([30, 10, 20]);

    assert_eq!(ascending.world().hazards(), permuted.world().hazards());
    assert_eq!(
        ascending.world().arena().blocks(),
        permuted.world().arena().blocks()
    );
    assert!(
        !ascending
            .world()
            .arena()
            .blocks()
            .contains(&CellId::new(6, 5))
    );
    assert!(ascending.world().bombs().is_empty());
    assert!(permuted.world().bombs().is_empty());
    let actor_facts = |state: &State| {
        state
            .actors()
            .iter()
            .map(|actor| (actor.seat(), actor.alive()))
            .collect::<Vec<_>>()
    };
    assert_eq!(actor_facts(&ascending), actor_facts(&permuted));
    assert!(ascending.actors().iter().all(|actor| !actor.alive()));
    assert_eq!(
        ascending.result(),
        Some(RoundResult::Draw(DrawReason::MutualElimination))
    );
    assert_eq!(ascending.result(), permuted.result());
}

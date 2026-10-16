use std::collections::BTreeSet;

use super::{ActorInit, InputAction, SeatInput, State, StateError};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::bombs::Bomb;
use crate::game::input::Direction;
use crate::game::rules::RulesConfig;
use crate::game::test_support::add_test_bomb;
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

fn actors(one: u32, two: u32) -> [ActorInit; 2] {
    [
        actor(one, SeatId::One, CellId::new(2, 5)),
        actor(two, SeatId::Two, CellId::new(12, 5)),
    ]
}

#[test]
fn cloned_nonempty_world_accepts_known_owner_and_seeds_allocator_above_bombs() {
    let mut source = State::new(RulesConfig::classic_v1(), open_world(), actors(10, 20)).unwrap();
    add_test_bomb(&mut source, EntityId(90), CellId::new(8, 5), EntityId(10)).unwrap();

    let mut rebuilt = State::new(
        RulesConfig::classic_v1(),
        source.world().clone(),
        actors(10, 20),
    )
    .unwrap();
    rebuilt
        .step(&[SeatInput {
            seat: SeatId::Two,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();

    assert!(rebuilt.world().bombs().contains_key(&EntityId(91)));
    assert_eq!(rebuilt.live_bomb_count(SeatId::One), 1);
    assert_eq!(rebuilt.live_bomb_count(SeatId::Two), 1);
}

#[test]
fn constructor_rejects_a_bomb_owned_by_an_unknown_actor() {
    let mut world = open_world();
    let bomb = Bomb {
        id: EntityId(90),
        cell: CellId::new(8, 5),
        owner: EntityId(99),
        range: 2,
        placed_at: Tick(1),
        detonates_at: Tick(181),
    };
    world.bombs.insert(bomb.id(), bomb);

    assert_eq!(
        State::new(RulesConfig::classic_v1(), world, actors(10, 20)),
        Err(StateError::UnknownBombOwner(EntityId(99)))
    );
}

#[test]
fn maximum_actor_id_does_not_exhaust_lower_bomb_ids() {
    let mut state =
        State::new(RulesConfig::classic_v1(), open_world(), actors(u32::MAX, 1)).unwrap();
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();

    let bomb = state.world().bombs().get(&EntityId(0)).unwrap();
    assert_eq!(bomb.owner(), EntityId(u32::MAX));
}

#[test]
fn constructor_rejects_a_bomb_whose_map_key_disagrees_with_its_id() {
    let mut world = open_world();
    world.bombs.insert(
        EntityId(89),
        Bomb {
            id: EntityId(90),
            cell: CellId::new(8, 5),
            owner: EntityId(10),
            range: 2,
            placed_at: Tick(1),
            detonates_at: Tick(181),
        },
    );

    assert_eq!(
        State::new(RulesConfig::classic_v1(), world, actors(10, 20)),
        Err(StateError::BombIdMismatch {
            key: EntityId(89),
            id: EntityId(90),
        })
    );
}

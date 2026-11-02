use std::collections::BTreeSet;

use super::super::{ActorInit, InputAction, SeatInput, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    Bomb, BombTrigger, CellId, Direction, EnabledItems, EntityId, Equipment, Position,
    PunchActivationPolicy, RulesConfig, SeatId, SpeedLevel, Tick, World,
};
use crate::room::RoomSettings;

fn arena(blocks: BTreeSet<CellId>) -> Arena {
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

fn bomb(id: u32, cell: CellId) -> Bomb {
    Bomb::new(
        EntityId(id),
        cell,
        EntityId(2),
        2,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
    )
    .unwrap()
}

fn input(direction: Direction) -> SeatInput {
    SeatInput {
        seat: SeatId::One,
        action: InputAction::Press(direction),
    }
}

#[test]
fn each_continuous_bomb_contact_stays_latched_until_its_own_separation() {
    let east = CellId::new(5, 5);
    let south = CellId::new(4, 6);
    let mut blocks = BTreeSet::new();
    blocks.extend([1, 2, 3, 6, 7, 8, 9, 10, 11, 12, 13].map(|column| CellId::new(column, 5)));
    blocks.extend([1, 2, 3, 4, 7, 8, 9, 10, 11].map(|row| CellId::new(4, row)));
    let position = Position {
        x: i32::from(east.column) * crate::game::types::UNITS_PER_CELL
            - crate::game::types::HALF_FOOTPRINT,
        y: Position::center(east).y,
    };
    let rules =
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL))
            .with_punch_activation_policy(PunchActivationPolicy::ContactV2);
    let mut state = State::new(
        rules,
        World::new(arena(blocks)),
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position,
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
        ],
    )
    .unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::Punch);
    state.world.bombs.insert(EntityId(9), bomb(9, east));
    state.world.bombs.insert(EntityId(10), bomb(10, south));

    state.step(&[input(Direction::East)]).unwrap();
    assert_eq!(
        state.actors[0].contact_punch_latches,
        BTreeSet::from([EntityId(9)])
    );

    state.actors[0].heading = Direction::South;
    state
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Release(Direction::East),
            },
            input(Direction::South),
        ])
        .unwrap();
    for _ in 1..6 {
        state.step(&[input(Direction::South)]).unwrap();
    }
    assert_eq!(
        state.actors[0].contact_punch_latches,
        BTreeSet::from([EntityId(9), EntityId(10)])
    );
    assert_eq!(state.world.bombs[&EntityId(10)].cell(), south);

    state.world.arena = arena(BTreeSet::new());
    state.actors[0].heading = Direction::West;
    state
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Release(Direction::South),
            },
            input(Direction::West),
        ])
        .unwrap();
    assert_eq!(
        state.actors[0].contact_punch_latches,
        BTreeSet::from([EntityId(10)])
    );

    state.actors[0].heading = Direction::South;
    state
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Release(Direction::West),
            },
            input(Direction::South),
        ])
        .unwrap();
    assert_eq!(state.world.bombs[&EntityId(10)].cell(), south);
    assert_eq!(
        state.actors[0].contact_punch_latches,
        BTreeSet::from([EntityId(10)])
    );

    state.actors[0].heading = Direction::North;
    state
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Release(Direction::South),
            },
            input(Direction::North),
        ])
        .unwrap();
    assert!(state.actors[0].contact_punch_latches.is_empty());
    state.actors[0].heading = Direction::South;
    state
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Release(Direction::North),
            },
            input(Direction::South),
        ])
        .unwrap();
    assert_ne!(state.world.bombs[&EntityId(10)].cell(), south);
}

use std::collections::BTreeSet;

use super::super::{ActorInit, RoundResult, State, StateError, StepError};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::{
    Bomb, BombTrigger, CellId, Direction, EnabledItems, EntityId, Position, RulesConfig, SeatId,
    SpeedLevel, Tick, World,
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

fn punched_bomb() -> Bomb {
    let mut bomb = Bomb::new(
        EntityId(9),
        CellId::new(5, 5),
        EntityId(1),
        2,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
    )
    .unwrap();
    bomb.restore_punch_history(true);
    bomb
}

#[test]
fn unsupported_punch_history_is_validated_before_terminal_short_circuit() {
    let mut world = World::new(arena());
    let bomb = punched_bomb();
    world.bombs.insert(bomb.id(), bomb);
    assert_eq!(
        State::new(RulesConfig::classic_v1(), world, actors()),
        Err(StateError::UnsupportedBombPunchHistory(EntityId(9)))
    );

    let rules = RulesConfig::expanded_v5(
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::PUNCH),
    );
    let mut world = World::new(arena());
    let bomb = punched_bomb();
    world.bombs.insert(bomb.id(), bomb);
    let mut state = State::new(rules, world, actors()).unwrap();
    state.result = Some(RoundResult::Winner(SeatId::One));
    state.rules = RulesConfig::classic_v1();
    assert_eq!(
        state.step(&[]),
        Err(StepError::UnsupportedBombPunchHistory(EntityId(9)))
    );
}

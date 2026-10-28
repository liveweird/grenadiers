use std::collections::BTreeSet;

use super::movement_collision::{ObstacleAccess, allowed_distance, position_is_free};
use super::types::{HALF_FOOTPRINT, UNITS_PER_CELL};
use super::{
    ActorInit, Arena, CellId, Direction, EntityId, InputAction, Position, RulesConfig, SeatId,
    SeatInput, SpeedLevel, State, StateError, Terrain, World,
};
use crate::room::{BoardSize, RoomSettings};

fn small_arena() -> Arena {
    let board = BoardSize::ElevenByNine;
    let mut terrain = vec![Terrain::Floor; board.cell_count()];
    for row in 0..board.height() {
        for column in 0..board.width() {
            if row == 0 || column == 0 || row == board.height() - 1 || column == board.width() - 1 {
                terrain[usize::from(row) * usize::from(board.width()) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    Arena::new_for_board(
        board,
        terrain,
        BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(9, 7)],
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
            position: Position::center(CellId::new(9, 7)),
            heading: Direction::West,
            speed: SpeedLevel::Base,
        },
    ]
}

#[test]
fn old_rules_reject_a_smaller_board_while_rules_six_accepts_it() {
    assert_eq!(
        State::new(
            RulesConfig::classic_v1(),
            World::new(small_arena()),
            actors()
        ),
        Err(StateError::UnsupportedBoardSize {
            version: 1,
            board: BoardSize::ElevenByNine,
        })
    );

    let state = State::new(
        RulesConfig::multiplayer_v6(RoomSettings::DEFAULT),
        World::new(small_arena()),
        actors(),
    )
    .unwrap();
    assert_eq!(state.public_view().arena.width, 11);
    assert_eq!(state.public_view().arena.height, 9);
    assert_eq!(state.public_view().arena.terrain.len(), 99);
}

#[test]
fn movement_and_input_processing_respect_small_board_geometry() {
    let world = World::new(small_arena());
    let empty_cells = BTreeSet::new();
    let distance = allowed_distance(
        &world,
        Position::center(CellId::new(9, 4)),
        Direction::East,
        1_000,
        &BTreeSet::new(),
        ObstacleAccess {
            can_pass: false,
            soft_grace: &empty_cells,
            oil_grace: &empty_cells,
        },
    );
    assert_eq!(distance, UNITS_PER_CELL / 2 - HALF_FOOTPRINT);
    assert!(!position_is_free(
        &world,
        Position::center(CellId::new(11, 4)),
        &BTreeSet::new(),
        false,
        &empty_cells,
        &empty_cells,
    ));

    let mut state = State::new(
        RulesConfig::multiplayer_v6(RoomSettings::DEFAULT),
        world,
        actors(),
    )
    .unwrap();
    let before = state.actors()[0].position();
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    assert!(state.actors()[0].position().x > before.x);
}

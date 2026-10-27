use std::collections::BTreeSet;
use std::num::NonZeroU32;

use super::{ActorInit, InputAction, SeatInput, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::test_support::add_test_bomb_with_deadline;
use crate::game::types::{HALF_FOOTPRINT, UNITS_PER_CELL};
use crate::game::{
    CellId, Direction, EnabledItems, EntityId, Equipment, PlayerCount, Position, RoundAssignment,
    RulesConfig, SeatId, SpeedLevel, Tick, World,
};
use crate::room::RoomSettings;

fn open_arena() -> Arena {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row * ARENA_WIDTH + column)] = Terrain::Wall;
            }
        }
    }
    Arena::new(
        terrain,
        BTreeSet::new(),
        [
            CellId::new(1, 1),
            CellId::new(13, 1),
            CellId::new(1, 11),
            CellId::new(13, 11),
        ],
    )
    .unwrap()
}

#[test]
fn three_kick_contenders_use_round_priority_for_one_shared_bomb() {
    let bomb_cell = CellId::new(7, 5);
    let center = Position::center(bomb_cell);
    let left = i32::from(bomb_cell.column) * UNITS_PER_CELL;
    let top = i32::from(bomb_cell.row) * UNITS_PER_CELL;
    let mut actors = PlayerCount::Four
        .seats()
        .iter()
        .map(|&seat| ActorInit {
            id: EntityId(10 + seat.index() as u32),
            seat,
            position: Position::center(match seat {
                SeatId::One => CellId::new(1, 1),
                SeatId::Two => CellId::new(13, 1),
                SeatId::Three => CellId::new(1, 11),
                SeatId::Four => CellId::new(13, 11),
            }),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        })
        .collect::<Vec<_>>();
    actors[SeatId::One.index()].position = Position {
        x: left - HALF_FOOTPRINT - 10,
        y: center.y,
    };
    actors[SeatId::Three.index()].position = Position {
        x: left + UNITS_PER_CELL + HALF_FOOTPRINT + 10,
        y: center.y,
    };
    actors[SeatId::Four.index()].position = Position {
        x: center.x,
        y: top - HALF_FOOTPRINT - 10,
    };
    let assignment = RoundAssignment::new(NonZeroU32::new(4).unwrap());
    let rules =
        RulesConfig::multiplayer_v6(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::KICK));
    let mut game = State::new_at_tick_with_assignment(
        rules,
        World::new(open_arena()),
        actors,
        Tick(0),
        assignment,
    )
    .unwrap();
    for seat in [SeatId::One, SeatId::Three, SeatId::Four] {
        game.replace_actor_equipment(seat, Equipment::Kick);
    }
    add_test_bomb_with_deadline(
        &mut game,
        EntityId(90),
        bomb_cell,
        EntityId(11),
        2,
        Tick(0),
        Tick(100),
    )
    .unwrap();

    game.step(&[
        SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        },
        SeatInput {
            seat: SeatId::Three,
            action: InputAction::Press(Direction::West),
        },
        SeatInput {
            seat: SeatId::Four,
            action: InputAction::Press(Direction::South),
        },
    ])
    .unwrap();

    let motion = game.world.bombs()[&EntityId(90)].motion().unwrap();
    assert_eq!(
        assignment.priority_seat_for(PlayerCount::Four),
        SeatId::Four
    );
    assert_eq!(motion.direction(), Direction::South);
}

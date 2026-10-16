use std::collections::BTreeSet;

use super::{
    ActorInit, DrawReason, InputAction, MAX_INPUTS_PER_TICK, RoundResult, SeatInput, State,
    StepError,
};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::input::Direction;
use crate::game::rules::RulesConfig;
use crate::game::test_support::add_test_bomb_with_deadline;
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

fn state() -> State {
    State::new(
        RulesConfig::classic_v1(),
        open_world(),
        [
            actor(10, SeatId::One, CellId::new(2, 5)),
            actor(20, SeatId::Two, CellId::new(12, 10)),
        ],
    )
    .unwrap()
}

#[test]
fn exact_running_update_limit_is_a_timeout_draw() {
    let mut state = state();

    for expected in 1..10_800 {
        state.step(&[]).unwrap();
        assert_eq!(state.running_updates(), expected);
        assert_eq!(state.result(), None);
        assert!(state.actors().iter().all(|actor| actor.alive()));
    }

    state.step(&[]).unwrap();

    assert_eq!(state.running_updates(), 10_800);
    assert_eq!(state.result(), Some(RoundResult::Draw(DrawReason::Timeout)));
}

#[test]
fn elimination_on_the_last_update_precedes_timeout() {
    let mut state = state();
    state.running_updates = state.rules().round_ticks() - 1;
    state.world.hazards.insert(CellId::new(2, 5), Tick(10));

    state.step(&[]).unwrap();

    assert_eq!(state.running_updates(), 10_800);
    assert_eq!(state.result(), Some(RoundResult::Winner(SeatId::Two)));
}

#[test]
fn completed_round_freezes_gameplay_but_absolute_ticks_and_reports_continue() {
    let mut state = state();
    state.world.hazards.insert(CellId::new(2, 5), Tick(100));
    add_test_bomb_with_deadline(
        &mut state,
        EntityId(30),
        CellId::new(8, 5),
        EntityId(20),
        2,
        Tick(0),
        Tick(3),
    )
    .unwrap();
    state.step(&[]).unwrap();
    assert_eq!(state.result(), Some(RoundResult::Winner(SeatId::Two)));
    let frozen_world = state.world.clone();
    let frozen_actors = state.actors.clone();
    let frozen_updates = state.running_updates();
    let frozen_result = state.result();

    for expected_tick in 2..=4 {
        let report = state
            .step(&[
                SeatInput {
                    seat: SeatId::Two,
                    action: InputAction::Press(Direction::West),
                },
                SeatInput {
                    seat: SeatId::Two,
                    action: InputAction::PlaceBomb,
                },
            ])
            .unwrap();
        assert_eq!(report.tick, Tick(expected_tick));
        assert!(report.traces.iter().all(|trace| trace.is_empty()));
    }

    assert_eq!(state.world, frozen_world);
    assert_eq!(state.actors, frozen_actors);
    assert_eq!(state.running_updates(), frozen_updates);
    assert_eq!(state.result(), frozen_result);
    assert!(state.world().bombs().contains_key(&EntityId(30)));
}

#[test]
fn frozen_state_still_validates_errors_atomically() {
    let mut state = state();
    state.world.hazards.insert(CellId::new(2, 5), Tick(10));
    state.step(&[]).unwrap();
    let too_many = vec![
        SeatInput {
            seat: SeatId::One,
            action: InputAction::Clear,
        };
        MAX_INPUTS_PER_TICK + 1
    ];
    let before = state.clone();
    assert_eq!(
        state.step(&too_many),
        Err(StepError::TooManyInputs {
            count: MAX_INPUTS_PER_TICK + 1,
            maximum: MAX_INPUTS_PER_TICK,
        })
    );
    assert_eq!(state, before);

    state.tick = Tick(u64::MAX);
    let before_overflow = state.clone();
    assert_eq!(state.step(&[]), Err(StepError::TickOverflow));
    assert_eq!(state, before_overflow);
}

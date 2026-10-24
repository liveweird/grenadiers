use std::collections::BTreeSet;

use super::super::{ActorInit, InputAction, SeatInput, State};
use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::bombs::BombTrigger;
use crate::game::test_support::add_test_bomb_with_trigger;
use crate::game::{
    CellId, Direction, EnabledItems, EntityId, Equipment, Position, RulesConfig, SeatId,
    SpeedLevel, Tick, World,
};
use crate::room::RoomSettings;

#[path = "kick_tests/motion_contracts.rs"]
mod motion_contracts;
#[path = "kick_tests/movement_contracts.rs"]
mod movement_contracts;
#[path = "kick_tests/phase_contracts.rs"]
mod phase_contracts;

fn open_world(blocks: BTreeSet<CellId>) -> World {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || row == 0 || column == ARENA_WIDTH - 1 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    World::new(Arena::new(terrain, blocks, [CellId::new(1, 1), CellId::new(13, 11)]).unwrap())
}

fn game(actor_position: Position, blocks: BTreeSet<CellId>) -> State {
    let rules =
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL));
    State::new(
        rules,
        open_world(blocks),
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position: actor_position,
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
    .unwrap()
}

fn tangent_west(cell: CellId) -> Position {
    let center = Position::center(cell);
    Position {
        x: i32::from(cell.column) * crate::game::types::UNITS_PER_CELL
            - crate::game::types::HALF_FOOTPRINT,
        y: center.y,
    }
}

fn tangent_east(cell: CellId) -> Position {
    let center = Position::center(cell);
    Position {
        x: (i32::from(cell.column) + 1) * crate::game::types::UNITS_PER_CELL
            + crate::game::types::HALF_FOOTPRINT,
        y: center.y,
    }
}

fn arm_east(state: &mut State, id: EntityId, cell: CellId) {
    add_test_bomb_with_trigger(
        state,
        id,
        cell,
        EntityId(1),
        2,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
    )
    .unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::Kick);
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
}

#[test]
fn collision_arms_then_hops_at_t_plus_one_eleven_and_twenty_one() {
    let start = CellId::new(4, 5);
    let mut state = game(tangent_west(start), BTreeSet::new());
    arm_east(&mut state, EntityId(9), start);
    let armed = state.world().bombs().values().next().unwrap();
    assert_eq!(armed.cell(), start);
    assert!(armed.has_been_kicked());
    assert_eq!(armed.motion().unwrap().countdown, 0);

    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Release(Direction::East),
        }])
        .unwrap();
    assert_eq!(
        state.world().bombs().values().next().unwrap().cell(),
        CellId::new(5, 5)
    );
    let expected = [(12, CellId::new(6, 5)), (22, CellId::new(7, 5))];
    for (tick, cell) in expected {
        while state.tick().0 < tick {
            state.step(&[]).unwrap();
        }
        assert_eq!(state.world().bombs().values().next().unwrap().cell(), cell);
    }
}

#[test]
fn blocked_first_hop_stops_but_preserves_kick_history() {
    let start = CellId::new(4, 5);
    let mut state = game(tangent_west(start), BTreeSet::from([CellId::new(5, 5)]));
    arm_east(&mut state, EntityId(9), start);
    state.step(&[]).unwrap();

    let bomb = state.world().bombs().values().next().unwrap();
    assert_eq!(bomb.cell(), start);
    assert_eq!(bomb.motion(), None);
    assert!(bomb.has_been_kicked());
}

#[test]
fn contact_bombs_do_not_block_or_arm_from_forward_movement() {
    let cell = CellId::new(4, 5);
    let mut state = game(tangent_west(cell), BTreeSet::new());
    add_test_bomb_with_trigger(
        &mut state,
        EntityId(9),
        cell,
        EntityId(1),
        2,
        Tick(0),
        BombTrigger::Contact,
    )
    .unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::Kick);
    let report = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    assert!(
        report
            .events
            .iter()
            .all(|event| !event.bomb_has_been_kicked)
    );
}

#[test]
fn living_actor_in_swept_destination_stops_motion_while_dead_actor_does_not() {
    let start = CellId::new(4, 5);
    for alive in [true, false] {
        let mut state = game(tangent_west(start), BTreeSet::new());
        state.actors[1].position = Position::center(CellId::new(5, 5));
        arm_east(&mut state, EntityId(9), start);
        state.actors[1].alive = alive;
        state.step(&[]).unwrap();
        let bomb = state.world().bombs().values().next().unwrap();
        assert_eq!(bomb.cell(), if alive { start } else { CellId::new(5, 5) });
        assert_eq!(bomb.motion().is_some(), !alive);
    }
}

#[test]
fn simultaneous_same_target_claims_both_stop_without_id_ordering() {
    let mut state = game(Position::center(CellId::new(1, 1)), BTreeSet::new());
    for (id, cell, direction) in [
        (9, CellId::new(4, 5), Direction::East),
        (10, CellId::new(6, 5), Direction::West),
    ] {
        add_test_bomb_with_trigger(
            &mut state,
            EntityId(id),
            cell,
            EntityId(1),
            2,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(100),
            },
        )
        .unwrap();
        state
            .world
            .bombs
            .get_mut(&EntityId(id))
            .unwrap()
            .restore_kick_history(
                Some(crate::game::BombMotion {
                    direction,
                    countdown: 0,
                }),
                true,
            );
    }

    state.step(&[]).unwrap();
    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(4, 5));
    assert_eq!(state.world.bombs[&EntityId(10)].cell(), CellId::new(6, 5));
    assert_eq!(state.world.bombs[&EntityId(9)].motion(), None);
    assert_eq!(state.world.bombs[&EntityId(10)].motion(), None);
}

#[test]
fn remote_bomb_hops_before_same_tick_manual_trigger() {
    let mut state = game(Position::center(CellId::new(1, 1)), BTreeSet::new());
    add_test_bomb_with_trigger(
        &mut state,
        EntityId(9),
        CellId::new(4, 5),
        EntityId(1),
        2,
        Tick(0),
        BombTrigger::Remote,
    )
    .unwrap();
    state
        .world
        .bombs
        .get_mut(&EntityId(9))
        .unwrap()
        .restore_kick_history(
            Some(crate::game::BombMotion {
                direction: Direction::East,
                countdown: 0,
            }),
            true,
        );

    let report = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    let detonated = report
        .events
        .iter()
        .find_map(|event| match event.kind {
            crate::game::StepEventKind::BombDetonated(bomb) => Some(bomb),
            _ => None,
        })
        .unwrap();
    assert_eq!(detonated.cell, CellId::new(5, 5));
    assert!(report.events.iter().any(|event| event.bomb_has_been_kicked));
}

#[test]
fn same_bomb_requests_use_round_priority_without_loser_fallback() {
    let cell = CellId::new(5, 5);
    let mut state = game(tangent_west(cell), BTreeSet::new());
    state.actors[1].position = tangent_east(cell);
    state.actors[1].heading = Direction::West;
    add_test_bomb_with_trigger(
        &mut state,
        EntityId(9),
        cell,
        EntityId(1),
        2,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
    )
    .unwrap();
    state.replace_actor_equipment(SeatId::One, Equipment::Kick);
    state.replace_actor_equipment(SeatId::Two, Equipment::Kick);

    state
        .step(&[
            SeatInput {
                seat: SeatId::Two,
                action: InputAction::Press(Direction::West),
            },
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Press(Direction::East),
            },
        ])
        .unwrap();
    assert_eq!(
        state.world.bombs[&EntityId(9)]
            .motion()
            .unwrap()
            .direction(),
        Direction::East
    );
}

#[test]
fn source_fire_holds_motion_and_target_fire_ignites_after_the_hop() {
    for source_burning in [true, false] {
        let source = CellId::new(4, 5);
        let target = CellId::new(5, 5);
        let mut state = game(Position::center(CellId::new(1, 1)), BTreeSet::new());
        add_test_bomb_with_trigger(
            &mut state,
            EntityId(9),
            source,
            EntityId(1),
            2,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(100),
            },
        )
        .unwrap();
        state
            .world
            .bombs
            .get_mut(&EntityId(9))
            .unwrap()
            .restore_kick_history(
                Some(crate::game::BombMotion {
                    direction: Direction::East,
                    countdown: 0,
                }),
                true,
            );
        state
            .world
            .hazards
            .insert(if source_burning { source } else { target }, Tick(10));

        let report = state.step(&[]).unwrap();
        let cell = report
            .events
            .iter()
            .find_map(|event| match event.kind {
                crate::game::StepEventKind::BombDetonated(bomb) => Some(bomb.cell),
                _ => None,
            })
            .unwrap();
        assert_eq!(cell, if source_burning { source } else { target });
    }
}

use super::*;
use crate::game::EffectKind;
use crate::game::types::{HALF_FOOTPRINT, UNITS_PER_CELL};

fn moving_bomb(state: &mut State, id: u32, cell: CellId, direction: Direction, countdown: u8) {
    add_test_bomb_with_trigger(
        state,
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
                countdown,
            }),
            true,
        );
}

fn west_tangent(cell: CellId) -> Position {
    Position {
        x: i32::from(cell.column) * UNITS_PER_CELL - HALF_FOOTPRINT,
        ..Position::center(cell)
    }
}

#[test]
fn every_living_source_overlap_blocks_including_owner_frozen_and_hidden() {
    for condition in ["owner", "frozen", "hidden"] {
        let source = CellId::new(4, 5);
        let mut state = game(Position::center(CellId::new(1, 1)), BTreeSet::new());
        moving_bomb(&mut state, 9, source, Direction::East, 0);
        let blocker = if condition == "owner" { 0 } else { 1 };
        state.actors[blocker].position = Position::center(source);
        match condition {
            "owner" => {}
            "frozen" => {
                state.actors[0]
                    .effects
                    .apply(EffectKind::Stopwatch, Tick(0))
                    .unwrap();
            }
            "hidden" => {
                state.actors[1]
                    .effects
                    .apply(EffectKind::InvisibilityCloak, Tick(0))
                    .unwrap();
            }
            _ => unreachable!(),
        }

        state.step(&[]).unwrap();

        let bomb = state.world.bombs[&EntityId(9)];
        assert_eq!(bomb.cell(), source, "{condition}");
        assert_eq!(bomb.motion(), None, "{condition}");
    }
}

#[test]
fn swept_cell_tangency_is_legal_but_one_unit_of_overlap_blocks() {
    let source = CellId::new(4, 5);
    for overlap in [0, 1] {
        let tangent = west_tangent(source);
        let position = Position {
            x: tangent.x + overlap,
            ..tangent
        };
        let mut state = game(position, BTreeSet::new());
        moving_bomb(&mut state, 9, source, Direction::East, 0);

        state.step(&[]).unwrap();

        let bomb = state.world.bombs[&EntityId(9)];
        assert_eq!(
            bomb.cell(),
            if overlap == 0 {
                CellId::new(5, 5)
            } else {
                source
            }
        );
    }
}

#[test]
fn walls_soft_blocks_and_contact_bombs_stop_a_due_hop() {
    let cases = [
        (CellId::new(1, 5), Direction::West, BTreeSet::new(), false),
        (
            CellId::new(4, 5),
            Direction::East,
            BTreeSet::from([CellId::new(5, 5)]),
            false,
        ),
        (CellId::new(4, 5), Direction::East, BTreeSet::new(), true),
    ];
    for (source, direction, blocks, contact) in cases {
        let mut state = game(Position::center(CellId::new(1, 1)), blocks);
        moving_bomb(&mut state, 9, source, direction, 0);
        if contact {
            add_test_bomb_with_trigger(
                &mut state,
                EntityId(10),
                CellId::new(5, 5),
                EntityId(2),
                2,
                Tick(0),
                BombTrigger::Contact,
            )
            .unwrap();
        }

        state.step(&[]).unwrap();
        assert_eq!(state.world.bombs[&EntityId(9)].cell(), source);
        assert_eq!(state.world.bombs[&EntityId(9)].motion(), None);
    }
}

#[test]
fn immutable_start_cells_block_exchange_and_following_a_moving_bomb() {
    for directions in [
        (Direction::East, Direction::East),
        (Direction::East, Direction::West),
    ] {
        let mut state = game(Position::center(CellId::new(1, 1)), BTreeSet::new());
        moving_bomb(&mut state, 9, CellId::new(4, 5), directions.0, 0);
        moving_bomb(&mut state, 10, CellId::new(5, 5), directions.1, 0);

        state.step(&[]).unwrap();

        assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(4, 5));
        assert_eq!(state.world.bombs[&EntityId(9)].motion(), None);
        if directions.1 == Direction::West {
            assert_eq!(state.world.bombs[&EntityId(10)].cell(), CellId::new(5, 5));
            assert_eq!(state.world.bombs[&EntityId(10)].motion(), None);
        } else {
            assert_eq!(state.world.bombs[&EntityId(10)].cell(), CellId::new(6, 5));
        }
    }
}

#[test]
fn same_target_stops_all_and_disjoint_hops_commit_under_id_permutations() {
    for swapped in [false, true] {
        let ids = if swapped { [10, 9] } else { [9, 10] };
        let mut conflict = game(Position::center(CellId::new(1, 1)), BTreeSet::new());
        moving_bomb(&mut conflict, ids[0], CellId::new(4, 5), Direction::East, 0);
        moving_bomb(&mut conflict, ids[1], CellId::new(6, 5), Direction::West, 0);
        conflict.step(&[]).unwrap();
        assert_eq!(
            conflict.world.bombs[&EntityId(ids[0])].cell(),
            CellId::new(4, 5)
        );
        assert_eq!(
            conflict.world.bombs[&EntityId(ids[1])].cell(),
            CellId::new(6, 5)
        );

        let mut disjoint = game(Position::center(CellId::new(1, 1)), BTreeSet::new());
        moving_bomb(&mut disjoint, ids[0], CellId::new(4, 5), Direction::East, 0);
        moving_bomb(&mut disjoint, ids[1], CellId::new(9, 5), Direction::West, 0);
        disjoint.step(&[]).unwrap();
        assert_eq!(
            disjoint.world.bombs[&EntityId(ids[0])].cell(),
            CellId::new(5, 5)
        );
        assert_eq!(
            disjoint.world.bombs[&EntityId(ids[1])].cell(),
            CellId::new(8, 5)
        );
    }
}

#[test]
fn waiting_bomb_is_solid_and_cannot_be_redirected() {
    let source = CellId::new(5, 5);
    let start = west_tangent(source);
    let mut state = game(start, BTreeSet::new());
    moving_bomb(&mut state, 9, source, Direction::East, 1);
    state.replace_actor_equipment(SeatId::One, Equipment::Kick);

    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();

    let bomb = state.world.bombs[&EntityId(9)];
    assert_eq!(state.actors[0].position, start);
    assert_eq!(bomb.motion().unwrap().direction(), Direction::East);
    assert_eq!(bomb.motion().unwrap().countdown, 0);
}

#[test]
fn actor_entering_a_future_sweep_blocks_the_next_due_hop() {
    let source = CellId::new(4, 5);
    let target = CellId::new(5, 5);
    let target_east_tangent = Position {
        x: (i32::from(target.column) + 1) * UNITS_PER_CELL + HALF_FOOTPRINT,
        ..Position::center(target)
    };
    let start = Position {
        x: target_east_tangent.x + 50,
        ..target_east_tangent
    };
    let mut state = game(start, BTreeSet::new());
    state.actors[0].heading = Direction::West;
    moving_bomb(&mut state, 9, source, Direction::East, 1);

    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::West),
        }])
        .unwrap();
    assert!(crate::game::movement::position_overlaps_cell(
        state.actors[0].position,
        target
    ));
    state.step(&[]).unwrap();

    assert_eq!(state.world.bombs[&EntityId(9)].cell(), source);
    assert_eq!(state.world.bombs[&EntityId(9)].motion(), None);
}

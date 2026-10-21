use super::*;
use crate::game::types::{HALF_FOOTPRINT, UNITS_PER_CELL};
use crate::game::{EffectKind, Pickup, PickupKind, StepEventKind};

fn west_tangent(cell: CellId) -> Position {
    Position {
        x: i32::from(cell.column) * UNITS_PER_CELL - HALF_FOOTPRINT,
        ..Position::center(cell)
    }
}

fn east_tangent(cell: CellId) -> Position {
    Position {
        x: (i32::from(cell.column) + 1) * UNITS_PER_CELL + HALF_FOOTPRINT,
        ..Position::center(cell)
    }
}

fn add_motion(
    state: &mut State,
    id: u32,
    cell: CellId,
    direction: Direction,
    countdown: u8,
    deadline: Tick,
) {
    add_test_bomb_with_trigger(
        state,
        EntityId(id),
        cell,
        EntityId(1),
        2,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: deadline,
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

fn press(direction: Direction) -> SeatInput {
    SeatInput {
        seat: SeatId::One,
        action: InputAction::Press(direction),
    }
}

#[test]
fn moving_at_start_cannot_redirect_after_stopping_or_becoming_due() {
    let source = CellId::new(4, 5);
    let mut stopped = game(west_tangent(source), BTreeSet::from([CellId::new(5, 5)]));
    add_motion(&mut stopped, 9, source, Direction::East, 0, Tick(100));
    stopped.replace_actor_equipment(SeatId::One, Equipment::Kick);
    stopped.step(&[press(Direction::East)]).unwrap();
    assert_eq!(stopped.world.bombs[&EntityId(9)].cell(), source);
    assert_eq!(stopped.world.bombs[&EntityId(9)].motion(), None);

    let mut due = game(west_tangent(source), BTreeSet::new());
    add_motion(&mut due, 9, source, Direction::East, 0, Tick(1));
    due.replace_actor_equipment(SeatId::One, Equipment::Kick);
    let report = due.step(&[press(Direction::East)]).unwrap();
    let event = report
        .events
        .iter()
        .find(|event| matches!(event.kind, StepEventKind::BombDetonated(_)))
        .unwrap();
    assert_eq!(event.bomb_motion.unwrap().direction(), Direction::East);
    assert!(event.bomb_has_been_kicked);
}

#[test]
fn stopwatch_extends_a_due_timed_bomb_before_motion_without_pausing_motion() {
    let source = CellId::new(4, 5);
    let mut state = game(Position::center(CellId::new(1, 1)), BTreeSet::new());
    add_motion(&mut state, 9, source, Direction::East, 0, Tick(1));
    state.actors[0]
        .effects
        .apply(EffectKind::Stopwatch, Tick(0))
        .unwrap();

    state.step(&[]).unwrap();

    let bomb = state.world.bombs[&EntityId(9)];
    assert_eq!(bomb.cell(), CellId::new(5, 5));
    assert_eq!(bomb.detonates_at(), Some(Tick(2)));
    assert_eq!(bomb.motion().unwrap().countdown, 9);
}

#[test]
fn movement_uses_postmotion_world_vacated_source_and_solid_destination() {
    let source = CellId::new(4, 5);
    let target = CellId::new(5, 5);

    let source_start = west_tangent(source);
    let mut enters_source = game(source_start, BTreeSet::new());
    enters_source.actors[0].heading = Direction::East;
    add_motion(&mut enters_source, 9, source, Direction::East, 0, Tick(100));
    enters_source.step(&[press(Direction::East)]).unwrap();
    assert!(enters_source.actors[0].position.x > source_start.x);
    assert_eq!(enters_source.world.bombs[&EntityId(9)].cell(), target);

    let target_start = east_tangent(target);
    let mut blocked_at_target = game(target_start, BTreeSet::new());
    blocked_at_target.actors[0].heading = Direction::West;
    add_motion(
        &mut blocked_at_target,
        9,
        source,
        Direction::East,
        0,
        Tick(100),
    );
    blocked_at_target.step(&[press(Direction::West)]).unwrap();
    assert_eq!(blocked_at_target.actors[0].position, target_start);
}

#[test]
fn motion_does_not_free_capacity_but_a_preexisting_spare_allows_placement() {
    // A legal actor cannot reach the physical source cell in the same update: any initial
    // overlap blocks the full-cell sweep, and one update's maximum walk is below the 384-unit
    // tangent-to-cell boundary. These paired cases prove the independently observable rule:
    // vacating a cell never frees owner capacity, while an existing spare remains usable.
    for spare in [false, true] {
        let actor_cell = CellId::new(2, 5);
        let source = CellId::new(8, 5);
        let mut state = game(Position::center(actor_cell), BTreeSet::new());
        if spare {
            state
                .world
                .pickups
                .exposed
                .insert(actor_cell, Pickup::new(PickupKind::BombCapacity));
            state.step(&[]).unwrap();
            assert_eq!(state.actors[0].bomb_capacity(), 2);
        }
        add_motion(&mut state, 9, source, Direction::East, 0, Tick(100));

        state
            .step(&[SeatInput {
                seat: SeatId::One,
                action: InputAction::PlaceBomb,
            }])
            .unwrap();

        assert!(!state.world.bombs.values().any(|bomb| bomb.cell() == source));
        assert_eq!(state.world.bombs.len(), if spare { 2 } else { 1 });
        assert_eq!(
            state
                .world
                .bombs
                .values()
                .filter(|bomb| bomb.cell() == actor_cell)
                .count(),
            usize::from(spare)
        );
    }
}

#[test]
fn same_tick_placement_is_not_retroactively_kicked() {
    let cell = CellId::new(5, 5);
    let mut state = game(Position::center(cell), BTreeSet::new());
    state.replace_actor_equipment(SeatId::One, Equipment::Kick);

    state
        .step(&[
            press(Direction::East),
            SeatInput {
                seat: SeatId::One,
                action: InputAction::PlaceBomb,
            },
        ])
        .unwrap();

    let bomb = state.world.bombs.values().next().unwrap();
    assert_eq!(bomb.motion(), None);
    assert!(!bomb.has_been_kicked());
}

#[test]
fn replacing_wall_pass_with_kick_preserves_only_exit_grace() {
    let block = CellId::new(5, 5);
    let mut state = game(Position::center(CellId::new(4, 5)), BTreeSet::from([block]));
    state.actors[0].position = Position::center(block);
    state.replace_actor_equipment(SeatId::One, Equipment::WallPass);
    state.replace_actor_equipment(SeatId::One, Equipment::Kick);
    assert!(state.actors[0].soft_wall_grace.contains(&block));

    state.step(&[press(Direction::East)]).unwrap();
    for _ in 0..25 {
        state.step(&[]).unwrap();
    }

    assert_eq!(
        state.actors[0].inventory().equipment(),
        Some(Equipment::Kick)
    );
    assert!(state.actors[0].soft_wall_grace.is_empty());
    assert!(state.actors[0].position.x > Position::center(block).x);
}

#[test]
fn teleporter_commits_after_unrelated_bomb_motion() {
    let one = CellId::new(2, 5);
    let two = CellId::new(12, 5);
    let mut state = game(Position::center(one), BTreeSet::new());
    state.actors[1].position = Position::center(two);
    state
        .world
        .pickups
        .exposed
        .insert(one, Pickup::new(PickupKind::Teleporter));
    add_motion(
        &mut state,
        9,
        CellId::new(7, 8),
        Direction::East,
        0,
        Tick(100),
    );

    state.step(&[]).unwrap();

    assert_eq!(state.world.bombs[&EntityId(9)].cell(), CellId::new(8, 8));
    assert_eq!(state.actors[0].position, Position::center(two));
    assert_eq!(state.actors[1].position, Position::center(one));
    assert_eq!(state.last_teleport_tick, Some(Tick(1)));
}

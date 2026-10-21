use grenadiers_core::game::{CellId, Direction, EnabledItems, Equipment, SeatId};

use super::replay;
use crate::game::rooms::match_runtime::{MatchAction, MatchRuntime, start_optional_runtime};

fn move_cells(runtime: &mut MatchRuntime, seat: SeatId, direction: Direction, cells: usize) {
    let epoch = runtime.lifecycle().round_attempt();
    runtime
        .action(seat, epoch, MatchAction::Movement(Some(direction)))
        .unwrap();
    for update in 0..cells * 20 {
        if update > 0 && update.is_multiple_of(10) {
            runtime
                .action(seat, epoch, MatchAction::Movement(Some(direction)))
                .unwrap();
        }
        runtime.tick().unwrap();
    }
    runtime
        .action(seat, epoch, MatchAction::Movement(None))
        .unwrap();
    runtime.tick().unwrap();
}

fn plant(runtime: &mut MatchRuntime, seat: SeatId, action_id: u32) {
    runtime
        .action(
            seat,
            runtime.lifecycle().round_attempt(),
            MatchAction::Bomb(action_id),
        )
        .unwrap();
    runtime.tick().unwrap();
}

fn wait_clear(runtime: &mut MatchRuntime) {
    for _ in 0..=180 {
        let state = &runtime.sim().unwrap().state;
        if state.world().bombs().is_empty() && state.world().hazards().is_empty() {
            return;
        }
        runtime.tick().unwrap();
    }
    panic!("seed 31 bomb and flame window did not clear");
}

fn assert_cell(runtime: &MatchRuntime, seat: SeatId, expected: CellId) {
    assert_eq!(
        runtime.sim().unwrap().state.actors()[seat.index()]
            .position()
            .cell(),
        Some(expected)
    );
}

fn collect_bracelet_and_clear_top(runtime: &mut MatchRuntime) {
    move_cells(runtime, SeatId::One, Direction::East, 1);
    plant(runtime, SeatId::One, 1);
    move_cells(runtime, SeatId::One, Direction::West, 1);
    move_cells(runtime, SeatId::One, Direction::South, 1);
    wait_clear(runtime);
    move_cells(runtime, SeatId::One, Direction::North, 1);
    move_cells(runtime, SeatId::One, Direction::East, 2);
    assert_cell(runtime, SeatId::One, CellId::new(3, 1));
    assert_eq!(
        runtime.sim().unwrap().state.actors()[0]
            .inventory()
            .equipment(),
        Some(Equipment::Bracelet)
    );

    for (action_id, west, east, corner_escape, expected) in [
        (2, 2, 3, true, CellId::new(4, 1)),
        (3, 3, 4, false, CellId::new(5, 1)),
        (4, 3, 6, false, CellId::new(8, 1)),
        (5, 3, 4, false, CellId::new(9, 1)),
        (6, 3, 4, false, CellId::new(10, 1)),
    ] {
        plant(runtime, SeatId::One, action_id);
        move_cells(runtime, SeatId::One, Direction::West, west);
        if corner_escape {
            move_cells(runtime, SeatId::One, Direction::South, 1);
        }
        wait_clear(runtime);
        if corner_escape {
            move_cells(runtime, SeatId::One, Direction::North, 1);
        }
        move_cells(runtime, SeatId::One, Direction::East, east);
        assert_cell(runtime, SeatId::One, expected);
    }
}

fn clear_right_and_stage_victim(runtime: &mut MatchRuntime) {
    move_cells(runtime, SeatId::Two, Direction::North, 1);
    plant(runtime, SeatId::Two, 1);
    move_cells(runtime, SeatId::Two, Direction::South, 1);
    move_cells(runtime, SeatId::Two, Direction::West, 1);
    wait_clear(runtime);
    move_cells(runtime, SeatId::Two, Direction::East, 1);
    move_cells(runtime, SeatId::Two, Direction::North, 4);
    assert_cell(runtime, SeatId::Two, CellId::new(13, 7));

    plant(runtime, SeatId::Two, 2);
    move_cells(runtime, SeatId::Two, Direction::South, 3);
    wait_clear(runtime);
    move_cells(runtime, SeatId::Two, Direction::North, 6);
    assert_cell(runtime, SeatId::Two, CellId::new(13, 4));

    for (action_id, south, north, expected) in [
        (3, 3, 4, CellId::new(13, 3)),
        (4, 3, 4, CellId::new(13, 2)),
        (5, 3, 4, CellId::new(13, 1)),
    ] {
        plant(runtime, SeatId::Two, action_id);
        move_cells(runtime, SeatId::Two, Direction::South, south);
        wait_clear(runtime);
        move_cells(runtime, SeatId::Two, Direction::North, north);
        assert_cell(runtime, SeatId::Two, expected);
    }
    plant(runtime, SeatId::Two, 6);
    move_cells(runtime, SeatId::Two, Direction::South, 3);
    wait_clear(runtime);
    move_cells(runtime, SeatId::Two, Direction::North, 3);
    move_cells(runtime, SeatId::Two, Direction::West, 2);
    assert_cell(runtime, SeatId::Two, CellId::new(11, 1));
}

#[test]
fn generated_bracelet_shove_and_private_history_replay_exactly() {
    let mut runtime = start_optional_runtime(EnabledItems::BRACELET, 31);
    collect_bracelet_and_clear_top(&mut runtime);
    clear_right_and_stage_victim(&mut runtime);
    assert_cell(&runtime, SeatId::One, CellId::new(10, 1));
    if runtime.replay_inspection().reports.len() == 2 {
        runtime.tick().unwrap();
    }

    let victim_start = runtime.sim().unwrap().state.actors()[1].position();
    let epoch = runtime.lifecycle().round_attempt();
    runtime
        .action(
            SeatId::One,
            epoch,
            MatchAction::Movement(Some(Direction::East)),
        )
        .unwrap();
    for _ in 0..20 {
        runtime.tick().unwrap();
        if runtime.sim().unwrap().state.actors()[1].position() != victim_start {
            break;
        }
    }
    runtime
        .action(SeatId::One, epoch, MatchAction::Movement(None))
        .unwrap();

    let victim_end = runtime.sim().unwrap().state.actors()[1].position();
    assert_eq!(victim_end.x - victim_start.x, 1_024);
    assert_eq!(victim_end.y, victim_start.y);
    assert_cell(&runtime, SeatId::Two, CellId::new(12, 1));
    let inspection = runtime.replay_inspection();
    let private = format!("{inspection:?}");
    assert_eq!(inspection.reports.len(), 1);
    assert!(private.contains("forced_movement: Some(ForcedMovement { seat: Two"));
    assert_eq!(
        private.matches("last_successful_shove_tick: Some").count(),
        1
    );
    assert_eq!(replay(runtime.replay_journal()).unwrap(), inspection);
}

use grenadiers_core::game::{CellId, Direction, EnabledItems, Equipment, SeatId};

use super::replay;
use crate::game::rooms::match_runtime::{
    MatchAction, collect_generated_pickup, move_cells, plant, start_optional_runtime_with_fuse,
    wait_clear,
};

fn assert_replay(runtime: &super::super::match_runtime::MatchRuntime) {
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}

fn assert_private_motion_absent(frame: &str) {
    for private in ["motion", "countdown", "hasBeenKicked", "direction"] {
        assert!(!frame.contains(private));
    }
}

#[test]
fn generated_kick_motion_resume_publication_and_replay_preserve_active_and_stopped_history() {
    let mut runtime = start_optional_runtime_with_fuse(EnabledItems::KICK, 31, 180);
    collect_generated_pickup(&mut runtime);
    assert_eq!(
        runtime.sim().unwrap().state.actors()[0]
            .inventory()
            .equipment(),
        Some(Equipment::Kick)
    );

    plant(&mut runtime, 2);
    move_cells(&mut runtime, Direction::West, 2);
    move_cells(&mut runtime, Direction::South, 1);
    wait_clear(&mut runtime);
    move_cells(&mut runtime, Direction::North, 1);
    move_cells(&mut runtime, Direction::East, 2);
    plant(&mut runtime, 3);
    move_cells(&mut runtime, Direction::East, 1);

    let epoch = runtime.lifecycle().round_attempt();
    runtime
        .action(
            SeatId::One,
            epoch,
            MatchAction::Movement(Some(Direction::West)),
        )
        .unwrap();
    for _ in 0..40 {
        runtime.tick().unwrap();
        if runtime
            .sim()
            .unwrap()
            .state
            .world()
            .bombs()
            .values()
            .next()
            .unwrap()
            .has_been_kicked()
        {
            break;
        }
    }
    let active = runtime
        .sim()
        .unwrap()
        .state
        .world()
        .bombs()
        .values()
        .next()
        .unwrap();
    assert_eq!(active.cell(), CellId::new(3, 1));
    assert_eq!(active.motion().unwrap().countdown(), 0);
    assert_replay(&runtime);

    for seat in SeatId::DUEL {
        let admission = runtime.prepare_resume(seat).unwrap();
        let frame = admission.notice.frame(seat).unwrap();
        assert_private_motion_absent(&frame);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&frame).unwrap()["state"]["bombs"][0]["cell"],
            serde_json::json!({"column": 3, "row": 1})
        );
        runtime.commit_admission(admission);
    }

    runtime
        .action(SeatId::One, epoch, MatchAction::Movement(None))
        .unwrap();
    runtime.tick().unwrap();
    let first_hop = runtime
        .sim()
        .unwrap()
        .state
        .world()
        .bombs()
        .values()
        .next()
        .unwrap();
    assert_eq!(first_hop.cell(), CellId::new(2, 1));
    assert_eq!(first_hop.motion().unwrap().countdown(), 9);
    assert_replay(&runtime);

    for _ in 0..20 {
        runtime.tick().unwrap();
    }
    let stopped = runtime
        .sim()
        .unwrap()
        .state
        .world()
        .bombs()
        .values()
        .next()
        .unwrap();
    assert_eq!(stopped.cell(), CellId::new(1, 1));
    assert_eq!(stopped.motion(), None);
    assert!(stopped.has_been_kicked());
    assert_replay(&runtime);

    let admission = runtime.prepare_resume(SeatId::One).unwrap();
    let frame = admission.notice.frame(SeatId::One).unwrap();
    assert_private_motion_absent(&frame);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&frame).unwrap()["state"]["bombs"][0]["cell"],
        serde_json::json!({"column": 1, "row": 1})
    );
}

use grenadiers_core::game::{CellId, Direction, EnabledItems, Equipment, SeatId};

use super::replay;
use crate::game::rooms::match_runtime::{
    collect_generated_pickup, move_cells, start_optional_runtime,
};

#[test]
fn generated_wall_pass_holder_inside_block_resumes_and_replays_without_private_wire_facts() {
    let mut runtime = start_optional_runtime(EnabledItems::WALL_PASS, 31);
    collect_generated_pickup(&mut runtime);
    assert_eq!(
        runtime.sim().unwrap().state.actors()[0]
            .inventory()
            .equipment(),
        Some(Equipment::WallPass)
    );

    let block = CellId::new(3, 2);
    assert!(
        runtime
            .sim()
            .unwrap()
            .state
            .world()
            .arena()
            .blocks()
            .contains(&block)
    );
    move_cells(&mut runtime, Direction::South, 1);
    assert_eq!(
        runtime.sim().unwrap().state.actors()[0].position().cell(),
        Some(block)
    );

    for seat in SeatId::ALL {
        let admission = runtime.prepare_resume(seat).unwrap();
        let frame = admission
            .notice
            .frame(seat)
            .expect("running admission snapshot");
        assert!(!frame.contains("softWall"));
        assert!(!frame.contains("wallGrace"));
        let value: serde_json::Value = serde_json::from_str(&frame).unwrap();
        assert_eq!(
            value["state"]["actors"][0]["inventory"]["equipment"],
            "wallPass"
        );
        assert!(
            value["state"]["arena"]["blocks"]
                .as_array()
                .unwrap()
                .iter()
                .any(|cell| cell == &serde_json::json!({"column": 3, "row": 2}))
        );
        runtime.commit_admission(admission);
    }
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    move_cells(&mut runtime, Direction::North, 1);
    move_cells(&mut runtime, Direction::South, 1);
    assert_eq!(
        runtime.sim().unwrap().state.actors()[0].position().cell(),
        Some(block)
    );
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}

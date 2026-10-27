use grenadiers_core::game::{CellId, Direction, EnabledItems, OilBarrelState, SeatId};

use super::replay;
use crate::game::rooms::match_runtime::{move_cells, plant, start_optional_runtime, wait_clear};

#[test]
fn concealed_reveal_resume_and_materialized_oil_replay_deterministically() {
    let mut runtime = start_optional_runtime(EnabledItems::OIL_BARREL, 31);
    assert!(
        runtime
            .sim()
            .unwrap()
            .state
            .public_view()
            .oil_barrels
            .is_empty()
    );
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    move_cells(&mut runtime, Direction::East, 1);
    plant(&mut runtime, 1);
    move_cells(&mut runtime, Direction::West, 1);
    move_cells(&mut runtime, Direction::South, 1);
    for _ in 0..=180 {
        let state = &runtime.sim().unwrap().state;
        if state.world().bombs().is_empty() && !state.world().hazards().is_empty() {
            break;
        }
        runtime.tick().unwrap();
    }
    let pending = &runtime.sim().unwrap().state;
    assert!(pending.world().bombs().is_empty());
    assert!(!pending.world().hazards().is_empty());
    assert!(
        !pending
            .world()
            .arena()
            .blocks()
            .contains(&CellId::new(3, 1))
    );
    assert!(pending.public_view().oil_barrels.is_empty());
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    wait_clear(&mut runtime);
    assert!(
        runtime
            .sim()
            .unwrap()
            .state
            .public_view()
            .oil_barrels
            .iter()
            .any(|oil| { oil.cell == CellId::new(3, 1) && oil.state == OilBarrelState::Unlit })
    );

    for seat in SeatId::DUEL {
        let admission = runtime.prepare_resume(seat).unwrap();
        let frame = admission.notice.frame(seat).expect("running Oil snapshot");
        let value: serde_json::Value = serde_json::from_str(&frame).unwrap();
        assert!(
            value["state"]["oilBarrels"]
                .as_array()
                .unwrap()
                .iter()
                .any(|oil| oil
                    == &serde_json::json!({
                        "cell":{"column":3,"row":1},
                        "state":"unlit"
                    }))
        );
        for private in [
            "concealed",
            "pending",
            "oilGrace",
            "generationRecord",
            "seed",
        ] {
            assert!(!frame.contains(private));
        }
        runtime.commit_admission(admission);
    }
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    move_cells(&mut runtime, Direction::North, 1);
    move_cells(&mut runtime, Direction::East, 1);
    plant(&mut runtime, 2);
    move_cells(&mut runtime, Direction::West, 1);
    move_cells(&mut runtime, Direction::South, 1);
    for _ in 0..=180 {
        if runtime
            .sim()
            .unwrap()
            .state
            .public_view()
            .oil_barrels
            .iter()
            .any(|oil| oil.cell == CellId::new(3, 1) && oil.state == OilBarrelState::Burning)
        {
            break;
        }
        runtime.tick().unwrap();
    }
    let burning = &runtime.sim().unwrap().state;
    assert!(!burning.world().hazards().is_empty());
    assert!(
        burning
            .public_view()
            .oil_barrels
            .iter()
            .any(|oil| { oil.cell == CellId::new(3, 1) && oil.state == OilBarrelState::Burning })
    );
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    wait_clear(&mut runtime);
    let settled = &runtime.sim().unwrap().state;
    assert!(settled.world().hazards().is_empty());
    assert!(
        settled
            .public_view()
            .oil_barrels
            .iter()
            .any(|oil| { oil.cell == CellId::new(3, 1) && oil.state == OilBarrelState::Burning })
    );
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}

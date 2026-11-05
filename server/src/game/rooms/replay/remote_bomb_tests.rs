use grenadiers_core::game::{
    BlastKind, BombTrigger, CellId, Direction, EnabledItems, SeatId, SpecialWeapon,
};
use grenadiers_core::{
    protocol::{Counter, v13::ResumeToken},
    room::DepartureReason,
};

use super::replay;
use crate::game::rooms::match_runtime::{
    ResumeKind, collect_generated_pickup, move_cells, plant, start_optional_runtime, wait_clear,
};

#[test]
fn generated_remote_charge_survives_resume_and_replay_until_manual_detonation() {
    let mut runtime = start_optional_runtime(EnabledItems::REMOTE_BOMB, 31);
    collect_generated_pickup(&mut runtime);
    assert_eq!(
        runtime.sim().unwrap().state.actors()[0]
            .inventory()
            .weapon(),
        Some(SpecialWeapon::Remote)
    );
    move_cells(&mut runtime, Direction::West, 2);
    move_cells(&mut runtime, Direction::South, 1);
    plant(&mut runtime, 2);
    let bomb = *runtime
        .sim()
        .unwrap()
        .state
        .world()
        .bombs()
        .values()
        .next()
        .unwrap();
    assert_eq!(bomb.cell(), CellId::new(1, 2));
    assert_eq!(bomb.trigger(), BombTrigger::Remote);
    assert_eq!(bomb.blast(), BlastKind::Standard);
    assert_eq!(
        runtime.sim().unwrap().state.actors()[0]
            .inventory()
            .weapon(),
        None
    );
    move_cells(&mut runtime, Direction::North, 1);
    move_cells(&mut runtime, Direction::East, 2);
    runtime
        .settle_boundary([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    for _ in 0..=183 {
        runtime.tick().unwrap();
    }
    assert_eq!(
        runtime.sim().unwrap().state.world().bombs().get(&bomb.id()),
        Some(&bomb)
    );
    let restoration = runtime.prepare_resume(SeatId::One).unwrap();
    assert_eq!(
        restoration.last_bomb_action_id,
        Some(Counter::new(2).unwrap())
    );
    let frame = restoration
        .encode(
            Counter::new(9).unwrap(),
            ResumeToken::new("c".repeat(64)).unwrap(),
        )
        .unwrap();
    let admitted: serde_json::Value = serde_json::from_str(&frame).unwrap();
    assert_eq!(admitted["type"], "admitted");
    assert_eq!(admitted["sequence"], 9);
    assert_eq!(admitted["youSeat"], "one");
    assert_eq!(admitted["lastActionId"], 2);
    assert!(admitted.get("events").is_none());
    let resumed_bomb = admitted["state"]["bombs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|candidate| candidate["id"] == u64::from(bomb.id().0))
        .unwrap();
    assert_eq!(resumed_bomb["trigger"]["type"], "remote");
    assert_eq!(runtime.commit_admission(restoration), ResumeKind::Restore);
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    plant(&mut runtime, 3);
    assert!(
        !runtime
            .sim()
            .unwrap()
            .state
            .world()
            .bombs()
            .contains_key(&bomb.id())
    );
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    wait_clear(&mut runtime);
    plant(&mut runtime, 4);
    let ordinary = runtime
        .sim()
        .unwrap()
        .state
        .world()
        .bombs()
        .values()
        .next()
        .unwrap();
    assert!(matches!(ordinary.trigger(), BombTrigger::Timed { .. }));
    assert_eq!(ordinary.blast(), BlastKind::Standard);
}

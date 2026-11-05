use grenadiers_core::{
    game::{CellId, Direction, EffectKind, EnabledItems, SeatId},
    protocol::{Counter, v13::ResumeToken},
    room::DepartureReason,
};

use super::replay;
use crate::game::rooms::match_runtime::{
    MatchAction, ResumeKind, collect_generated_pickup, move_cells, plant, start_optional_runtime,
    wait_clear,
};

fn remaining_effect(state: &serde_json::Value, kind: &str) -> u64 {
    state["actors"][0]["effects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|effect| effect["kind"] == kind)
        .and_then(|effect| effect["remainingTicks"].as_u64())
        .unwrap()
}

fn remaining_bomb(state: &serde_json::Value, id: u64) -> (u64, bool) {
    let trigger = &state["bombs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|candidate| candidate["id"] == id)
        .unwrap()["trigger"];
    (
        trigger["remainingTicks"].as_u64().unwrap(),
        trigger["paused"].as_bool().unwrap(),
    )
}

#[test]
fn stopwatch_freeze_action_floor_thaw_and_natural_bomb_replay_deterministically() {
    let mut runtime = start_optional_runtime(EnabledItems::STOPWATCH, 31);
    collect_generated_pickup(&mut runtime);
    assert!(
        runtime.sim().unwrap().state.actors()[0]
            .effects()
            .active(EffectKind::Stopwatch, runtime.sim().unwrap().state.tick())
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
    move_cells(&mut runtime, Direction::North, 1);
    move_cells(&mut runtime, Direction::East, 2);

    let epoch = runtime.lifecycle().round_attempt();
    let frozen_start = runtime.sim().unwrap().state.actors()[1].position();
    runtime
        .action(
            SeatId::Two,
            epoch,
            MatchAction::Movement(Some(Direction::South)),
        )
        .unwrap();
    runtime
        .action(SeatId::Two, epoch, MatchAction::Bomb(1))
        .unwrap();
    runtime.tick().unwrap();
    assert_eq!(
        runtime.sim().unwrap().state.actors()[1].position(),
        frozen_start
    );
    assert_eq!(runtime.sim().unwrap().state.world().bombs().len(), 1);
    let prepared = runtime.prepare_resume(SeatId::Two).unwrap();
    assert_eq!(prepared.last_bomb_action_id, Some(Counter::new(1).unwrap()));

    let before_loss = runtime.prepare_resume(SeatId::One).unwrap();
    assert_eq!(
        before_loss.last_bomb_action_id,
        Some(Counter::new(2).unwrap())
    );
    let before_loss: serde_json::Value = serde_json::from_str(
        &before_loss
            .encode(
                Counter::new(8).unwrap(),
                ResumeToken::new("b".repeat(64)).unwrap(),
            )
            .unwrap(),
    )
    .unwrap();
    let initial_effect = remaining_effect(&before_loss["state"], "stopwatch");
    let (initial_fuse, initially_paused) =
        remaining_bomb(&before_loss["state"], u64::from(bomb.id().0));
    assert!(initially_paused);

    runtime
        .settle_boundary([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    for _ in 0..3 {
        runtime.tick().unwrap();
    }
    let restore_tick = runtime.sim().unwrap().state.tick();
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
    assert_eq!(
        remaining_effect(&admitted["state"], "stopwatch"),
        initial_effect - 4
    );
    assert_eq!(
        remaining_bomb(&admitted["state"], u64::from(bomb.id().0)),
        (initial_fuse, true)
    );
    assert_eq!(runtime.commit_admission(restoration), ResumeKind::Restore);
    assert_eq!(runtime.sim().unwrap().state.tick(), restore_tick);
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    let mut frozen_ticks = 0;
    while runtime.sim().unwrap().state.voluntary_actions_frozen()[1] {
        if frozen_ticks % 10 == 0 {
            runtime
                .action(
                    SeatId::Two,
                    epoch,
                    MatchAction::Movement(Some(Direction::South)),
                )
                .unwrap();
        }
        runtime.tick().unwrap();
        frozen_ticks += 1;
    }
    runtime.tick().unwrap();
    assert_ne!(
        runtime.sim().unwrap().state.actors()[1].position(),
        frozen_start
    );
    assert_eq!(runtime.sim().unwrap().state.world().bombs().len(), 1);
    runtime
        .action(SeatId::Two, epoch, MatchAction::Movement(None))
        .unwrap();
    while runtime
        .sim()
        .unwrap()
        .state
        .world()
        .bombs()
        .contains_key(&bomb.id())
    {
        runtime.tick().unwrap();
    }
    assert!(
        runtime
            .sim()
            .unwrap()
            .state
            .world()
            .hazards()
            .contains_key(&CellId::new(1, 2))
    );
    wait_clear(&mut runtime);
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}

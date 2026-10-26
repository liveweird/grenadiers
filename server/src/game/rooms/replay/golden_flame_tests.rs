use grenadiers_core::{
    game::{Direction, EffectKind, EnabledItems, SeatId},
    protocol::{Counter, Seat, v12::ResumeToken},
    room::{DepartureReason, MatchPhase},
};

use super::replay;
use crate::game::rooms::match_runtime::{
    MatchRuntime, collect_generated_pickup, move_cells, plant, start_optional_runtime, wait_clear,
};

fn personal_state(runtime: &MatchRuntime) -> serde_json::Value {
    let prepared = runtime.prepare_resume(SeatId::One).unwrap();
    let encoded = crate::game::rooms::match_runtime::prepare_expanded_admission!(
        &prepared.room,
        prepared.state_epoch,
        prepared.capture.as_ref(),
        prepared.generation,
        Seat::One,
        prepared.last_bomb_action_id,
        Counter::new(1).unwrap(),
        ResumeToken::new("c".repeat(64)).unwrap(),
    )
    .unwrap();
    let frame: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert!(frame.get("events").is_none());
    frame["state"].clone()
}

#[test]
fn generated_golden_flame_captures_range_resumes_expires_and_replays_next_round() {
    let mut runtime = start_optional_runtime(EnabledItems::GOLDEN_FLAME, 31);
    collect_generated_pickup(&mut runtime);
    let baseline = personal_state(&runtime);
    assert_eq!(baseline["actors"][0]["effects"][0]["kind"], "goldenFlame");
    let remaining = baseline["actors"][0]["effects"][0]["remainingTicks"]
        .as_u64()
        .unwrap();
    assert!((1..=600).contains(&remaining));
    assert_eq!(baseline["actors"][0]["bombRange"], 2);
    let replacement = runtime.prepare_resume(SeatId::One).unwrap();
    runtime.commit_admission(replacement);
    runtime
        .settle_boundary([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    let restored = personal_state(&runtime);
    assert_eq!(restored["actors"][0]["effects"][0]["kind"], "goldenFlame");
    assert!(
        restored["actors"][0]["effects"][0]["remainingTicks"]
            .as_u64()
            .unwrap()
            <= remaining
    );
    let restoration = runtime.prepare_resume(SeatId::One).unwrap();
    runtime.commit_admission(restoration);

    plant(&mut runtime, 2);
    assert_eq!(runtime.sim().unwrap().state.world().bombs().len(), 1);
    assert_eq!(
        runtime
            .sim()
            .unwrap()
            .state
            .world()
            .bombs()
            .values()
            .next()
            .unwrap()
            .range(),
        8
    );
    move_cells(&mut runtime, Direction::West, 2);
    move_cells(&mut runtime, Direction::South, 1);
    wait_clear(&mut runtime);
    assert!(runtime.sim().unwrap().state.actors()[0].alive());
    for _ in 0..=600 {
        let state = &runtime.sim().unwrap().state;
        if !state.actors()[0]
            .effects()
            .active(EffectKind::GoldenFlame, state.tick())
        {
            break;
        }
        runtime.tick().unwrap();
    }
    assert_eq!(
        personal_state(&runtime)["actors"][0]["effects"],
        serde_json::json!([])
    );
    plant(&mut runtime, 3);
    assert_eq!(runtime.sim().unwrap().state.world().bombs().len(), 1);
    assert_eq!(
        runtime
            .sim()
            .unwrap()
            .state
            .world()
            .bombs()
            .values()
            .next()
            .unwrap()
            .range(),
        2
    );
    for _ in 0..=180 {
        if !runtime.sim().unwrap().state.actors()[0].alive() {
            break;
        }
        runtime.tick().unwrap();
    }
    assert!(!runtime.sim().unwrap().state.actors()[0].alive());
    for _ in 0..1000 {
        if runtime.lifecycle().round_attempt() == 2
            && runtime.lifecycle().phase() == MatchPhase::Running
        {
            break;
        }
        runtime.tick().unwrap();
    }
    assert_eq!(runtime.lifecycle().round_attempt(), 2);
    assert_eq!(runtime.lifecycle().phase(), MatchPhase::Running);
    assert_eq!(
        runtime.lifecycle().settings().enabled_items(),
        EnabledItems::GOLDEN_FLAME
    );
    assert_eq!(
        personal_state(&runtime)["actors"][0]["effects"],
        serde_json::json!([])
    );
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}

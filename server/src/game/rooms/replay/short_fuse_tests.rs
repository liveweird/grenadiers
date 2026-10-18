use grenadiers_core::{
    game::{BombTrigger, Direction, EnabledItems, SeatId},
    protocol::{Counter, Seat, v8::ResumeToken},
    room::{DepartureReason, MatchPhase},
};

use super::replay;
use crate::game::rooms::match_runtime::{
    MatchRuntime, collect_generated_pickup, move_cells, plant, start_optional_runtime, wait_clear,
};

fn personal_state(runtime: &MatchRuntime) -> serde_json::Value {
    let prepared = runtime.prepare_resume(SeatId::One).unwrap();
    let encoded = grenadiers_core::protocol::v8::prepare_expanded_admission(
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

fn remaining(runtime: &MatchRuntime) -> u64 {
    personal_state(runtime)["actors"][0]["effects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|effect| effect["kind"] == "shortFuse")
        .and_then(|effect| effect["remainingTicks"].as_u64())
        .unwrap_or(0)
}

fn captured_fuse(runtime: &MatchRuntime) -> u64 {
    let bombs = runtime.sim().unwrap().state.world().bombs();
    assert_eq!(bombs.len(), 1);
    let bomb = bombs.values().next().unwrap();
    let BombTrigger::Timed { detonates_at } = bomb.trigger() else {
        panic!("ordinary timed bomb expected")
    };
    detonates_at.0 - bomb.placed_at().0
}

#[test]
fn generated_short_fuse_preserves_schedules_through_expiry_resume_and_next_round_replay() {
    let enabled = EnabledItems::SHORT_FUSE;
    let mut runtime = start_optional_runtime(enabled, 31);
    collect_generated_pickup(&mut runtime);
    assert!((1..=600).contains(&remaining(&runtime)));
    let original_remaining = remaining(&runtime);
    let expires_at = runtime
        .sim()
        .unwrap()
        .state
        .tick()
        .0
        .checked_add(original_remaining)
        .unwrap();
    let replacement = runtime.prepare_resume(SeatId::One).unwrap();
    runtime.commit_admission(replacement);
    runtime
        .settle_boundary([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    let restored = personal_state(&runtime);
    assert_eq!(restored["actors"][0]["effects"][0]["kind"], "shortFuse");
    assert!(
        restored["actors"][0]["effects"][0]["remainingTicks"]
            .as_u64()
            .unwrap()
            <= original_remaining
    );
    let restoration = runtime.prepare_resume(SeatId::One).unwrap();
    runtime.commit_admission(restoration);

    plant(&mut runtime, 2);
    assert_eq!(captured_fuse(&runtime), 90);
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
    move_cells(&mut runtime, Direction::West, 2);
    move_cells(&mut runtime, Direction::South, 1);
    wait_clear(&mut runtime);
    assert!(runtime.sim().unwrap().state.actors()[0].alive());

    for _ in 0..600 {
        if expires_at.saturating_sub(runtime.sim().unwrap().state.tick().0) <= 20 {
            break;
        }
        runtime.tick().unwrap();
    }
    assert!((1..=20).contains(&remaining(&runtime)));
    plant(&mut runtime, 3);
    assert_eq!(captured_fuse(&runtime), 90);
    move_cells(&mut runtime, Direction::North, 1);
    move_cells(&mut runtime, Direction::East, 1);
    assert_eq!(remaining(&runtime), 0);
    assert_eq!(captured_fuse(&runtime), 90);
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
    wait_clear(&mut runtime);
    assert!(runtime.sim().unwrap().state.actors()[0].alive());
    assert_eq!(
        personal_state(&runtime)["actors"][0]["effects"],
        serde_json::json!([])
    );

    plant(&mut runtime, 4);
    assert_eq!(captured_fuse(&runtime), 180);
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
    assert_eq!(runtime.lifecycle().settings().enabled_items(), enabled);
    assert_eq!(remaining(&runtime), 0);
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}

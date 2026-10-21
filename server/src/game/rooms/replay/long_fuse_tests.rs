use grenadiers_core::{
    game::{BombTrigger, Direction, EnabledItems, SeatId},
    protocol::{Counter, Seat, v10::ResumeToken},
    room::{DepartureReason, MatchPhase},
};

use super::replay;
use crate::game::rooms::match_runtime::{
    MatchRuntime, collect_generated_pickup, move_cells, plant, start_optional_runtime_with_fuse,
    wait_clear,
};

fn personal_state(runtime: &MatchRuntime, seat: SeatId) -> serde_json::Value {
    let prepared = runtime.prepare_resume(seat).unwrap();
    let encoded = grenadiers_core::protocol::v10::prepare_expanded_admission(
        &prepared.room,
        prepared.state_epoch,
        prepared.capture.as_ref(),
        prepared.generation,
        match seat {
            SeatId::One => Seat::One,
            SeatId::Two => Seat::Two,
        },
        prepared.last_bomb_action_id,
        Counter::new(1).unwrap(),
        ResumeToken::new("c".repeat(64)).unwrap(),
    )
    .unwrap();
    serde_json::from_str::<serde_json::Value>(&encoded).unwrap()["state"].clone()
}

fn remaining(runtime: &MatchRuntime) -> u64 {
    personal_state(runtime, SeatId::One)["actors"][0]["effects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|effect| effect["kind"] == "longFuse")
        .and_then(|effect| effect["remainingTicks"].as_u64())
        .unwrap_or(0)
}

fn captured_fuse(runtime: &MatchRuntime) -> u64 {
    let bomb = runtime
        .sim()
        .unwrap()
        .state
        .world()
        .bombs()
        .values()
        .next()
        .unwrap();
    let BombTrigger::Timed { detonates_at } = bomb.trigger() else {
        panic!("timed bomb expected")
    };
    detonates_at.0 - bomb.placed_at().0
}

#[test]
fn generated_long_fuse_collects_resumes_expires_and_replays_exactly() {
    let mut runtime = start_optional_runtime_with_fuse(EnabledItems::LONG_FUSE, 31, 120);
    collect_generated_pickup(&mut runtime);
    let initial = remaining(&runtime);
    assert!((1..=600).contains(&initial));
    let replacement = runtime.prepare_resume(SeatId::One).unwrap();
    runtime.commit_admission(replacement);
    runtime
        .settle_boundary([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    assert!((1..=initial).contains(&remaining(&runtime)));
    let restoration = runtime.prepare_resume(SeatId::One).unwrap();
    runtime.commit_admission(restoration);
    plant(&mut runtime, 2);
    assert_eq!(captured_fuse(&runtime), 240);
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
    move_cells(&mut runtime, Direction::West, 2);
    move_cells(&mut runtime, Direction::South, 1);
    wait_clear(&mut runtime);
    while remaining(&runtime) > 120 {
        runtime.tick().unwrap();
    }
    assert!((1..=120).contains(&remaining(&runtime)));
    plant(&mut runtime, 3);
    assert_eq!(captured_fuse(&runtime), 240);
    let historical_deadline = runtime
        .sim()
        .unwrap()
        .state
        .world()
        .bombs()
        .values()
        .next()
        .unwrap()
        .detonates_at();
    move_cells(&mut runtime, Direction::North, 1);
    move_cells(&mut runtime, Direction::East, 2);
    while remaining(&runtime) != 0 {
        runtime.tick().unwrap();
    }
    assert_eq!(remaining(&runtime), 0);
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
            .detonates_at(),
        historical_deadline
    );
    for seat in [SeatId::One, SeatId::Two] {
        let state = personal_state(&runtime, seat);
        assert_eq!(state["bombs"][0]["trigger"]["paused"], false);
        assert!(
            state["bombs"][0]["trigger"]["remainingTicks"]
                .as_u64()
                .unwrap()
                > 0
        );
    }
    let historical_resume = runtime.prepare_resume(SeatId::One).unwrap();
    runtime.commit_admission(historical_resume);
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
    wait_clear(&mut runtime);
    assert!(runtime.sim().unwrap().state.world().bombs().is_empty());
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
    plant(&mut runtime, 4);
    assert_eq!(captured_fuse(&runtime), 120);
    for _ in 0..=120 {
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
    assert_eq!(remaining(&runtime), 0);
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}

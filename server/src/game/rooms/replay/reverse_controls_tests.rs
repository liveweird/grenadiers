use grenadiers_core::{
    game::{Direction, EnabledItems, SeatId},
    protocol::{Counter, Seat, v10::ResumeToken},
    room::{DepartureReason, MatchPhase},
};

use super::replay;
use crate::game::rooms::match_runtime::{
    MatchAction, MatchRuntime, collect_generated_pickup, plant, start_optional_runtime,
};

fn personal_state(runtime: &MatchRuntime, seat: SeatId) -> serde_json::Value {
    let prepared = runtime.prepare_resume(seat).unwrap();
    let encoded = grenadiers_core::protocol::v10::prepare_expanded_admission(
        &prepared.room,
        prepared.state_epoch,
        prepared.capture.as_ref(),
        prepared.generation,
        if seat == SeatId::One {
            Seat::One
        } else {
            Seat::Two
        },
        prepared.last_bomb_action_id,
        Counter::new(1).unwrap(),
        ResumeToken::new("c".repeat(64)).unwrap(),
    )
    .unwrap();
    serde_json::from_str::<serde_json::Value>(&encoded).unwrap()["state"].clone()
}

fn remaining(runtime: &MatchRuntime, seat: SeatId) -> u64 {
    personal_state(runtime, seat)["actors"][0]["effects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|effect| effect["kind"] == "reverseControls")
        .and_then(|effect| effect["remainingTicks"].as_u64())
        .unwrap_or(0)
}

fn x(runtime: &MatchRuntime) -> i32 {
    runtime.sim().unwrap().state.actors()[0].position().x
}

#[test]
fn generated_reverse_controls_preserves_raw_input_across_replay_and_lifecycle() {
    let mut runtime = start_optional_runtime(EnabledItems::REVERSE_CONTROLS, 31);
    collect_generated_pickup(&mut runtime);
    let collected = remaining(&runtime, SeatId::One);
    assert!((1..=600).contains(&collected));
    assert_eq!(remaining(&runtime, SeatId::Two), collected);

    runtime
        .action(SeatId::One, 1, MatchAction::Movement(Some(Direction::East)))
        .unwrap();
    let before_reversed = x(&runtime);
    runtime.tick().unwrap();
    assert!(x(&runtime) < before_reversed);
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    let replacement = runtime.prepare_resume(SeatId::One).unwrap();
    runtime.commit_admission(replacement);
    runtime
        .settle_boundary([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    let after_loss = x(&runtime);
    runtime.tick().unwrap();
    assert_eq!(x(&runtime), after_loss, "transport loss clears held input");
    let restoration = runtime.prepare_resume(SeatId::One).unwrap();
    runtime.commit_admission(restoration);
    assert_eq!(
        remaining(&runtime, SeatId::One),
        remaining(&runtime, SeatId::Two)
    );

    runtime
        .action(SeatId::One, 1, MatchAction::Movement(Some(Direction::East)))
        .unwrap();
    let mut updates = 0;
    while remaining(&runtime, SeatId::One) > 1 {
        if updates % 10 == 0 {
            runtime
                .action(SeatId::One, 1, MatchAction::Movement(Some(Direction::East)))
                .unwrap();
        }
        runtime.tick().unwrap();
        updates += 1;
    }
    runtime
        .action(SeatId::One, 1, MatchAction::Movement(Some(Direction::East)))
        .unwrap();
    let before_expiry = x(&runtime);
    runtime.tick().unwrap();
    assert_eq!(remaining(&runtime, SeatId::One), 0);
    assert!(
        x(&runtime) > before_expiry,
        "expiry tick uses the raw east input: {before_expiry} -> {}, alive={}",
        x(&runtime),
        runtime.sim().unwrap().state.actors()[0].alive()
    );
    runtime
        .action(SeatId::One, 1, MatchAction::Movement(None))
        .unwrap();
    runtime.tick().unwrap();
    let released = x(&runtime);
    runtime.tick().unwrap();
    assert_eq!(x(&runtime), released);
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    plant(&mut runtime, 2);
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
    assert_eq!(remaining(&runtime, SeatId::One), 0);
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}

#[test]
fn death_and_next_round_clear_generated_reverse_controls() {
    let mut runtime = start_optional_runtime(EnabledItems::REVERSE_CONTROLS, 31);
    collect_generated_pickup(&mut runtime);
    assert!(remaining(&runtime, SeatId::One) > 0);
    plant(&mut runtime, 2);
    for _ in 0..=180 {
        if !runtime.sim().unwrap().state.actors()[0].alive() {
            break;
        }
        runtime.tick().unwrap();
    }
    assert!(!runtime.sim().unwrap().state.actors()[0].alive());
    assert_eq!(remaining(&runtime, SeatId::One), 0);
    for _ in 0..1000 {
        if runtime.lifecycle().round_attempt() == 2
            && runtime.lifecycle().phase() == MatchPhase::Running
        {
            break;
        }
        runtime.tick().unwrap();
    }
    assert_eq!(runtime.lifecycle().round_attempt(), 2);
    assert_eq!(remaining(&runtime, SeatId::One), 0);
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}

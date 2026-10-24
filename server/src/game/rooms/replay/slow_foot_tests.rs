use grenadiers_core::{
    game::{Direction, EnabledItems, SeatId},
    protocol::{Counter, Seat, v11::ResumeToken},
    room::{DepartureReason, MatchPhase},
};

use super::replay;
use crate::game::rooms::match_runtime::{
    MatchAction, MatchRuntime, collect_generated_pickup, plant, start_optional_runtime,
};

fn personal_state(runtime: &MatchRuntime, seat: SeatId) -> serde_json::Value {
    let prepared = runtime.prepare_resume(seat).unwrap();
    let encoded = grenadiers_core::protocol::v11::prepare_expanded_admission(
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

fn slow_foot(runtime: &MatchRuntime, seat: SeatId) -> Vec<u64> {
    personal_state(runtime, seat)["actors"][0]["effects"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|effect| effect["kind"] == "slowFoot")
        .map(|effect| effect["remainingTicks"].as_u64().unwrap())
        .collect()
}

fn x(runtime: &MatchRuntime) -> i32 {
    runtime.sim().unwrap().state.actors()[0].position().x
}

#[test]
fn generated_slow_foot_preserves_timer_movement_reconnect_and_replay() {
    let mut runtime = start_optional_runtime(EnabledItems::SLOW_FOOT, 31);
    collect_generated_pickup(&mut runtime);
    let collected = slow_foot(&runtime, SeatId::One);
    assert_eq!(collected.len(), 1);
    assert!((1..=600).contains(&collected[0]));
    assert_eq!(slow_foot(&runtime, SeatId::Two), collected);

    runtime
        .action(SeatId::One, 1, MatchAction::Movement(Some(Direction::East)))
        .unwrap();
    let before = x(&runtime);
    runtime.tick().unwrap();
    assert!((24..=27).contains(&(x(&runtime) - before)));
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    runtime
        .settle_boundary([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    let stopped = x(&runtime);
    runtime.tick().unwrap();
    assert_eq!(x(&runtime), stopped);
    let restoration = runtime.prepare_resume(SeatId::One).unwrap();
    runtime.commit_admission(restoration);
    assert_eq!(
        slow_foot(&runtime, SeatId::One),
        slow_foot(&runtime, SeatId::Two)
    );

    runtime
        .action(SeatId::One, 1, MatchAction::Movement(Some(Direction::West)))
        .unwrap();
    while slow_foot(&runtime, SeatId::One)
        .first()
        .copied()
        .unwrap_or(0)
        > 1
    {
        runtime.tick().unwrap();
    }
    runtime
        .action(SeatId::One, 1, MatchAction::Movement(Some(Direction::East)))
        .unwrap();
    let before_expiry = x(&runtime);
    runtime.tick().unwrap();
    assert!(slow_foot(&runtime, SeatId::One).is_empty());
    assert!((50..=52).contains(&(x(&runtime) - before_expiry)));
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    runtime
        .action(SeatId::One, 1, MatchAction::Movement(None))
        .unwrap();
    plant(&mut runtime, 2);
    for _ in 0..=180 {
        if !runtime.sim().unwrap().state.actors()[0].alive() {
            break;
        }
        runtime.tick().unwrap();
    }
    assert!(!runtime.sim().unwrap().state.actors()[0].alive());
    assert!(slow_foot(&runtime, SeatId::One).is_empty());
    for _ in 0..1000 {
        if runtime.lifecycle().round_attempt() == 2
            && runtime.lifecycle().phase() == MatchPhase::Running
        {
            break;
        }
        runtime.tick().unwrap();
    }
    assert_eq!(runtime.lifecycle().round_attempt(), 2);
    assert!(slow_foot(&runtime, SeatId::One).is_empty());
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}

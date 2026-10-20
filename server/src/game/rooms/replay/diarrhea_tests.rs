use grenadiers_core::{
    game::{BombTrigger, Direction, EnabledItems, SeatId},
    protocol::{Counter, Seat, v8::ResumeToken},
    room::{DepartureReason, MatchPhase},
};

use super::replay;
use crate::game::rooms::match_runtime::{
    MatchAction, MatchRuntime, collect_generated_pickup, move_cells, plant,
    start_optional_runtime_with_fuse,
};

fn admission(runtime: &MatchRuntime, seat: SeatId) -> (serde_json::Value, Option<Counter>) {
    let prepared = runtime.prepare_resume(seat).unwrap();
    let action_id = prepared.last_bomb_action_id;
    let encoded = grenadiers_core::protocol::v8::prepare_expanded_admission(
        &prepared.room,
        prepared.state_epoch,
        prepared.capture.as_ref(),
        prepared.generation,
        match seat {
            SeatId::One => Seat::One,
            SeatId::Two => Seat::Two,
        },
        action_id,
        Counter::new(1).unwrap(),
        ResumeToken::new("c".repeat(64)).unwrap(),
    )
    .unwrap();
    (
        serde_json::from_str::<serde_json::Value>(&encoded).unwrap()["state"].clone(),
        action_id,
    )
}

fn remaining(runtime: &MatchRuntime, seat: SeatId) -> u64 {
    admission(runtime, seat).0["actors"][0]["effects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|effect| effect["kind"] == "diarrhea")
        .and_then(|effect| effect["remainingTicks"].as_u64())
        .unwrap_or(0)
}

fn collect_with_long_fuse(runtime: &mut MatchRuntime) {
    move_cells(runtime, Direction::East, 1);
    plant(runtime, 1);
    move_cells(runtime, Direction::West, 1);
    move_cells(runtime, Direction::South, 1);
    for _ in 0..360 {
        let world = runtime.sim().unwrap().state.world();
        if world.bombs().is_empty() && world.hazards().is_empty() {
            break;
        }
        runtime.tick().unwrap();
    }
    move_cells(runtime, Direction::North, 1);
    move_cells(runtime, Direction::East, 2);
}

#[test]
fn generated_diarrhea_auto_places_for_reserved_actor_and_replays_through_expiry() {
    let mut runtime = start_optional_runtime_with_fuse(EnabledItems::DIARRHEA, 31, 300);
    collect_with_long_fuse(&mut runtime);
    let collected = remaining(&runtime, SeatId::One);
    assert!((1..=600).contains(&collected));
    assert_eq!(remaining(&runtime, SeatId::Two), collected);
    assert_eq!(
        admission(&runtime, SeatId::One).1,
        Some(Counter::new(1).unwrap())
    );

    let replacement = runtime.prepare_resume(SeatId::One).unwrap();
    runtime.commit_admission(replacement);
    runtime
        .settle_boundary([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    runtime.tick().unwrap();
    let bomb = runtime
        .sim()
        .unwrap()
        .state
        .world()
        .bombs()
        .values()
        .next()
        .unwrap();
    assert_eq!(bomb.owner(), runtime.sim().unwrap().state.actors()[0].id());
    let BombTrigger::Timed { detonates_at } = bomb.trigger() else {
        panic!("timed bomb expected")
    };
    assert_eq!(detonates_at.0 - bomb.placed_at().0, 300);
    assert_eq!(
        admission(&runtime, SeatId::One).1,
        Some(Counter::new(1).unwrap())
    );
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
    let restoration = runtime.prepare_resume(SeatId::One).unwrap();
    runtime.commit_admission(restoration);

    move_cells(&mut runtime, Direction::West, 2);
    move_cells(&mut runtime, Direction::South, 1);
    while !runtime.sim().unwrap().state.world().bombs().is_empty() {
        runtime.tick().unwrap();
    }
    assert!(runtime.sim().unwrap().state.actors()[0].alive());
    assert_eq!(runtime.lifecycle().phase(), MatchPhase::Running);
    runtime.tick().unwrap();
    assert_eq!(runtime.sim().unwrap().state.live_bomb_count(SeatId::One), 1);
    while !runtime.sim().unwrap().state.world().hazards().is_empty() {
        runtime.tick().unwrap();
    }
    move_cells(&mut runtime, Direction::North, 1);
    move_cells(&mut runtime, Direction::East, 2);
    while remaining(&runtime, SeatId::One) != 0 {
        runtime.tick().unwrap();
    }
    assert_eq!(remaining(&runtime, SeatId::Two), 0);
    assert_eq!(runtime.sim().unwrap().state.world().bombs().len(), 1);
    while !runtime.sim().unwrap().state.world().bombs().is_empty() {
        runtime.tick().unwrap();
    }
    assert!(runtime.sim().unwrap().state.world().bombs().is_empty());
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );

    runtime
        .action(SeatId::One, 1, MatchAction::Bomb(2))
        .unwrap();
    runtime.tick().unwrap();
    assert_eq!(
        admission(&runtime, SeatId::One).1,
        Some(Counter::new(2).unwrap())
    );
}

#[test]
fn diarrhea_death_and_next_round_clear_effect_without_synthetic_action_id() {
    let mut runtime = start_optional_runtime_with_fuse(EnabledItems::DIARRHEA, 31, 180);
    collect_generated_pickup(&mut runtime);
    runtime.tick().unwrap();
    assert_eq!(runtime.sim().unwrap().state.live_bomb_count(SeatId::One), 1);
    assert_eq!(
        admission(&runtime, SeatId::One).1,
        Some(Counter::new(1).unwrap())
    );
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
    assert_eq!(admission(&runtime, SeatId::One).1, None);
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}

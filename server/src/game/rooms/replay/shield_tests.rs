use grenadiers_core::{
    game::{EffectKind, SeatId},
    protocol::{Counter, Seat, v13::ResumeToken},
    room::DepartureReason,
};

use super::replay;
use crate::game::rooms::match_runtime::{
    MatchAction, collect_generated_pickup, start_shield_runtime, wait_clear,
};

fn personal_state(runtime: &super::super::match_runtime::MatchRuntime) -> serde_json::Value {
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
    let message: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert!(message.get("events").is_none());
    message["state"].clone()
}

#[test]
fn generated_shield_survives_resume_absorbs_then_expires_and_replays_exactly() {
    let mut runtime = start_shield_runtime();
    collect_generated_pickup(&mut runtime);
    let state = &runtime.sim().unwrap().state;
    assert!(
        state.actors()[0]
            .effects()
            .active(EffectKind::Shield, state.tick())
    );
    let original_expiry = personal_state(&runtime)["actors"][0]["effects"][0]["remainingTicks"]
        .as_u64()
        .unwrap();
    assert!((1..=300).contains(&original_expiry));

    let replacement = runtime.prepare_resume(SeatId::One).unwrap();
    runtime.commit_admission(replacement);
    runtime
        .settle_boundary([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    let baseline = personal_state(&runtime);
    assert!(
        baseline["actors"][0]["effects"][0]["remainingTicks"]
            .as_u64()
            .unwrap()
            <= original_expiry
    );
    let restoration = runtime.prepare_resume(SeatId::One).unwrap();
    runtime.commit_admission(restoration);

    runtime
        .action(SeatId::One, 1, MatchAction::Bomb(2))
        .unwrap();
    runtime.tick().unwrap();
    assert_eq!(runtime.sim().unwrap().state.world().bombs().len(), 1);
    wait_clear(&mut runtime);
    assert!(runtime.sim().unwrap().state.actors()[0].alive());
    for _ in 0..300 {
        let state = &runtime.sim().unwrap().state;
        if !state.actors()[0]
            .effects()
            .active(EffectKind::Shield, state.tick())
        {
            break;
        }
        runtime.tick().unwrap();
    }
    assert_eq!(
        personal_state(&runtime)["actors"][0]["effects"],
        serde_json::json!([])
    );
    runtime
        .action(SeatId::One, 1, MatchAction::Bomb(3))
        .unwrap();
    for _ in 0..=180 {
        runtime.tick().unwrap();
        if !runtime.sim().unwrap().state.actors()[0].alive() {
            break;
        }
    }
    assert!(!runtime.sim().unwrap().state.actors()[0].alive());
    assert_eq!(
        replay(runtime.replay_journal()).unwrap(),
        runtime.replay_inspection()
    );
}

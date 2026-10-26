use grenadiers_core::{
    game::SeatId,
    protocol::{Counter, Seat, v12::ResumeToken},
    room::DepartureReason,
};

use super::ResumeKind;
use super::test_support::{
    absorb_generated_heart, collect_generated_heart, expanded_heart_runtime,
    start_expanded_heart_round,
};

fn personal_state(prepared: &super::PreparedRuntimeAdmission) -> serde_json::Value {
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

fn assert_consumed_heart_baseline(state: &serde_json::Value) {
    assert!(state["actors"][0]["inventory"]["equipment"].is_null());
    assert!(
        state["actors"][0]["damageGraceRemainingTicks"]
            .as_u64()
            .is_some_and(|ticks| ticks > 0)
    );
}

fn assert_collected_heart_baseline(state: &serde_json::Value) {
    assert_eq!(state["actors"][0]["inventory"]["equipment"], "heart");
    assert_eq!(state["actors"][0]["damageGraceRemainingTicks"], 0);
}

#[test]
fn generated_heart_inventory_and_grace_survive_replacement_and_restore() {
    let mut runtime = expanded_heart_runtime();
    start_expanded_heart_round(&mut runtime);
    collect_generated_heart(&mut runtime);

    let replacement = runtime.prepare_resume(SeatId::One).unwrap();
    assert_collected_heart_baseline(&personal_state(&replacement));
    assert_eq!(runtime.commit_admission(replacement), ResumeKind::Replace);

    absorb_generated_heart(&mut runtime);
    runtime.tick().unwrap();
    runtime
        .settle_boundary([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    let restoration = runtime.prepare_resume(SeatId::One).unwrap();
    assert_consumed_heart_baseline(&personal_state(&restoration));
    assert_eq!(runtime.commit_admission(restoration), ResumeKind::Restore);
}

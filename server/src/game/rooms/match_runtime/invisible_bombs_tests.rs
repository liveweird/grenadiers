use grenadiers_core::{
    game::{EnabledItems, SeatId},
    protocol::{Counter, Seat, v11::ResumeToken},
};

use super::test_support::{collect_generated_pickup, plant, start_optional_runtime};

fn admission(prepared: &super::PreparedRuntimeAdmission, recipient: Seat) -> serde_json::Value {
    let encoded = grenadiers_core::protocol::v11::prepare_expanded_admission(
        &prepared.room,
        prepared.state_epoch,
        prepared.capture.as_ref(),
        prepared.generation,
        recipient,
        prepared.last_bomb_action_id,
        Counter::new(1).unwrap(),
        ResumeToken::new("c".repeat(64)).unwrap(),
    )
    .unwrap();
    serde_json::from_str(&encoded).unwrap()
}

#[test]
fn live_publication_and_quiet_resume_filter_only_the_afflicted_recipient() {
    let mut runtime = start_optional_runtime(EnabledItems::INVISIBLE_BOMBS, 31);
    collect_generated_pickup(&mut runtime);
    plant(&mut runtime, 2);

    let notice = runtime.current_notice().unwrap();
    let blind: serde_json::Value =
        serde_json::from_str(notice.frame(SeatId::One).unwrap().as_ref()).unwrap();
    let sighted: serde_json::Value =
        serde_json::from_str(notice.frame(SeatId::Two).unwrap().as_ref()).unwrap();
    assert!(blind["state"]["bombs"].as_array().unwrap().is_empty());
    assert_eq!(blind["state"]["actors"][0]["liveBombCount"], 1);
    assert_eq!(sighted["state"]["bombs"].as_array().unwrap().len(), 1);
    let remaining = blind["state"]["actors"][0]["effects"][0]["remainingTicks"]
        .as_u64()
        .unwrap();

    let owner = runtime.prepare_resume(SeatId::One).unwrap();
    let owner_frame = admission(&owner, Seat::One);
    assert!(owner_frame.get("events").is_none());
    assert!(owner_frame["state"]["bombs"].as_array().unwrap().is_empty());
    runtime.commit_admission(owner);

    let opponent = runtime.prepare_resume(SeatId::Two).unwrap();
    let opponent_frame = admission(&opponent, Seat::Two);
    assert!(opponent_frame.get("events").is_none());
    assert_eq!(
        opponent_frame["state"]["bombs"].as_array().unwrap().len(),
        1
    );
    runtime.commit_admission(opponent);

    let replacement = runtime.prepare_resume(SeatId::One).unwrap();
    let replacement_frame = admission(&replacement, Seat::One);
    assert_eq!(
        replacement_frame["state"]["actors"][0]["effects"][0]["remainingTicks"],
        remaining
    );
    assert!(
        replacement_frame["state"]["bombs"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

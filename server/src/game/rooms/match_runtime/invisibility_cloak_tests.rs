use grenadiers_core::{
    game::{EnabledItems, SeatId},
    protocol::{Counter, Seat, v13::ResumeToken},
};

use super::test_support::{collect_generated_pickup, start_optional_runtime};

fn admission(prepared: &super::PreparedRuntimeAdmission, recipient: Seat) -> serde_json::Value {
    let encoded = crate::game::rooms::match_runtime::prepare_expanded_admission!(
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
fn asymmetric_live_publication_and_quiet_replacements_keep_the_physical_deadline() {
    let mut runtime = start_optional_runtime(EnabledItems::INVISIBILITY_CLOAK, 31);
    collect_generated_pickup(&mut runtime);

    let notice = runtime.current_notice().unwrap();
    let one: serde_json::Value =
        serde_json::from_str(notice.frame(SeatId::One).unwrap().as_ref()).unwrap();
    let two: serde_json::Value =
        serde_json::from_str(notice.frame(SeatId::Two).unwrap().as_ref()).unwrap();
    assert_eq!(one["state"]["actors"][0]["type"], "visible");
    assert_eq!(one["state"]["actors"][0]["voluntaryActionsFrozen"], false);
    let remaining = one["state"]["actors"][0]["effects"][0]["remainingTicks"]
        .as_u64()
        .unwrap();
    assert_eq!(two["state"]["actors"][0]["type"], "hidden");
    assert_eq!(two["state"]["actors"][0].as_object().unwrap().len(), 4);

    let owner = runtime.prepare_resume(SeatId::One).unwrap();
    let owner_frame = admission(&owner, Seat::One);
    assert!(owner_frame.get("events").is_none());
    assert_eq!(owner_frame["state"]["actors"][0]["type"], "visible");
    assert_eq!(
        owner_frame["state"]["actors"][0]["voluntaryActionsFrozen"],
        false
    );
    runtime.commit_admission(owner);

    let opponent = runtime.prepare_resume(SeatId::Two).unwrap();
    let opponent_frame = admission(&opponent, Seat::Two);
    assert!(opponent_frame.get("events").is_none());
    assert_eq!(opponent_frame["state"]["actors"][0]["type"], "hidden");
    runtime.commit_admission(opponent);

    let replacement = runtime.prepare_resume(SeatId::One).unwrap();
    let replacement_frame = admission(&replacement, Seat::One);
    let after = replacement_frame["state"]["actors"][0]["effects"][0]["remainingTicks"]
        .as_u64()
        .unwrap();
    assert_eq!(after, remaining);
}

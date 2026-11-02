use super::*;
use crate::protocol::Counter;

#[test]
fn checked_client_corpus_accepts_bomb_and_rejects_retired_equipment() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/fixtures/gameplay-v20-client-conformance.json"
    )))
    .unwrap();
    for case in corpus["validMessages"].as_array().unwrap() {
        let encoded = serde_json::to_string(&case["value"]).unwrap();
        assert!(parse_client_message(&encoded).is_ok(), "{}", case["caseId"]);
    }
    for case in corpus["invalidMessages"].as_array().unwrap() {
        let encoded = serde_json::to_string(&case["value"]).unwrap();
        assert!(
            parse_client_message(&encoded).is_err(),
            "{}",
            case["caseId"]
        );
    }
}

#[test]
fn gameplay20_identity_keeps_the_v19_shaped_server_body() {
    assert_eq!(
        serde_json::to_value(GameServerMessage::Authenticated {
            protocol_version: GameplayVersion,
        })
        .unwrap(),
        serde_json::json!({"type": "authenticated", "protocolVersion": 20})
    );
    let mut shared =
        serde_json::to_value(crate::protocol::v19::server_conformance_tests::corpus()).unwrap();
    shared["producer"] = serde_json::Value::String(
        "Rust gameplay v20 typed projection; ContactV2 is a closed transport identity".into(),
    );
    shared["validMessages"][0]["caseId"] = serde_json::Value::String("v20-authenticated".into());
    shared["validMessages"][0]["value"]["protocolVersion"] = serde_json::json!(20);
    let expected: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/fixtures/gameplay-v20-server-conformance.json"
    )))
    .unwrap();
    assert_eq!(shared, expected);
}

#[test]
fn authoritative_observation_round_trips_the_closed_contact_activation() {
    use crate::{
        cpu::WorldId,
        game::{PunchActivationPolicy, SpikePitPhase},
        protocol::Counter,
    };

    let (state, generation) =
        crate::protocol::v19::server_conformance_tests::runtime_with_activation(
            false,
            false,
            SpikePitPhase::Idle,
            PunchActivationPolicy::ContactV2,
        );
    let capture = capture_expanded(&state, &[]).unwrap();
    let room = crate::protocol::v19::server_conformance_tests::room(
        state.tick().0,
        RoomPhase::Running,
        false,
    );
    let publication = bind_expanded(&room, Counter::new(1).unwrap(), &capture, generation).unwrap();
    let observation = observe_cpu_recipient(
        &publication,
        Seat::One,
        WorldId {
            room_instance: 20,
            physical_round: 1,
        },
    )
    .unwrap();

    assert_eq!(
        observation.parts().rules.punch_activation_policy,
        PunchActivationPolicy::ContactV2
    );
}

#[test]
fn profile_binder_closes_generator_and_pit_layout_as_one_identity() {
    use crate::game::{ArenaConstructionProfile, PunchActivationPolicy, SpikePitPhase};

    let construction = ArenaConstructionProfile::LairClearanceV5;
    let (state, generation) = crate::protocol::v19::server_conformance_tests::runtime_with_profile(
        true,
        false,
        SpikePitPhase::Idle,
        PunchActivationPolicy::ContactV2,
        construction,
    );
    let capture = capture_expanded(&state, &[]).unwrap();
    let room = crate::protocol::v19::server_conformance_tests::room(
        state.tick().0,
        RoomPhase::Running,
        true,
    );

    assert!(bind_expanded(&room, Counter::new(1).unwrap(), &capture, generation).is_err());
    bind_expanded_with_profile(
        &room,
        Counter::new(1).unwrap(),
        &capture,
        generation,
        construction,
    )
    .unwrap();
    let personal = || PersonalAdmission {
        recipient: Seat::One,
        last_action_id: None,
        sequence: Counter::new(2).unwrap(),
        resume_token: ResumeToken::new(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .unwrap(),
    };
    let projection = || AdmissionProjection {
        room: &room,
        state_epoch: Some(Counter::new(1).unwrap()),
        capture: Some(&capture),
        generation: Some(generation),
    };
    assert!(prepare_admission(projection(), personal()).is_err());
    prepare_admission_with_profile(projection(), personal(), construction).unwrap();

    let (legacy_state, legacy_generation) =
        crate::protocol::v19::server_conformance_tests::runtime_with_activation(
            true,
            false,
            SpikePitPhase::Idle,
            PunchActivationPolicy::ContactV2,
        );
    let legacy_capture = capture_expanded(&legacy_state, &[]).unwrap();
    assert!(
        bind_expanded_with_profile(
            &room,
            Counter::new(1).unwrap(),
            &capture,
            legacy_generation,
            construction,
        )
        .is_err()
    );
    assert!(
        bind_expanded_with_profile(
            &room,
            Counter::new(1).unwrap(),
            &legacy_capture,
            generation,
            construction,
        )
        .is_err()
    );
}

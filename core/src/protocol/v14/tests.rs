use super::*;
use crate::room::{BoardSize as DomainBoardSize, CpuDifficulty, SeatControl};
use jsonschema::validator_for;
use serde::Deserialize;
use serde_json::Value;

#[test]
fn checked_configuration_requires_explicit_null_and_domain_invariants() {
    let valid = r#"{"type":"createRoom","sequence":1,"configuration":{"controllers":["human","cpu","human","cpu"],"cpuDifficulty":"standard","board":{"width":15,"height":13}}}"#;
    let GameClientMessage::CreateRoom { configuration, .. } = parse_client_message(valid).unwrap()
    else {
        panic!("wrong message")
    };
    assert_eq!(
        configuration.domain().controllers(),
        &[
            SeatControl::Human,
            SeatControl::Cpu,
            SeatControl::Human,
            SeatControl::Cpu
        ]
    );
    assert_eq!(
        configuration.domain().cpu_difficulty(),
        Some(CpuDifficulty::Standard)
    );
    assert_eq!(
        configuration.domain().board(),
        DomainBoardSize::FifteenByThirteen
    );

    for invalid in [
        r#"{"type":"createRoom","sequence":1,"configuration":{"controllers":["human","human"],"board":{"width":15,"height":13}}}"#,
        r#"{"type":"createRoom","sequence":1,"configuration":{"controllers":["cpu","human"],"cpuDifficulty":"easy","board":{"width":15,"height":13}}}"#,
        r#"{"type":"createRoom","sequence":1,"configuration":{"controllers":["human","cpu"],"cpuDifficulty":null,"board":{"width":15,"height":13}}}"#,
        r#"{"type":"createRoom","sequence":1,"configuration":{"controllers":["human","human"],"cpuDifficulty":"easy","board":{"width":15,"height":13}}}"#,
        r#"{"type":"createRoom","sequence":1,"configuration":{"controllers":["human","human"],"cpuDifficulty":null,"board":{"width":12,"height":10}}}"#,
    ] {
        assert_eq!(
            parse_client_message(invalid),
            Err(ParseClientMessageError::InvalidMessage)
        );
    }
}

#[test]
fn all_supported_lineups_and_boards_round_trip() {
    let mut lineups = Vec::new();
    for count in 2..=4 {
        for suffix in 0..(1 << (count - 1)) {
            let mut controllers = vec!["human"];
            for index in 0..count - 1 {
                controllers.push(if suffix & (1 << index) == 0 {
                    "human"
                } else {
                    "cpu"
                });
            }
            if suffix == 0 {
                lineups.push((controllers, None));
            } else {
                lineups.push((controllers.clone(), Some("easy")));
                lineups.push((controllers, Some("standard")));
            }
        }
    }
    for (controllers, difficulty) in lineups {
        for (width, height) in [(11, 9), (13, 11), (15, 13)] {
            let value = serde_json::json!({"type":"createRoom", "sequence":1, "configuration":{"controllers":controllers, "cpuDifficulty":difficulty, "board":{"width":width,"height":height}}});
            assert!(parse_client_message(&value.to_string()).is_ok(), "{value}");
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Conformance {
    valid_create_rooms: Vec<Case>,
    invalid_create_rooms: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    case_id: String,
    value: Value,
}

#[test]
fn parser_and_schema_agree_on_v14_configuration_fixture() {
    let corpus: Conformance = serde_json::from_str(include_str!(
        "../../../../api/fixtures/gameplay-v14-client-conformance.json"
    ))
    .unwrap();
    let mut schema: Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v14.schema.json")).unwrap();
    schema["$ref"] = serde_json::json!("#/$defs/GameClientMessage");
    let validator = validator_for(&schema).unwrap();
    for case in corpus.valid_create_rooms {
        assert!(
            parse_client_message(&case.value.to_string()).is_ok(),
            "{}",
            case.case_id
        );
        assert!(validator.is_valid(&case.value), "{}", case.case_id);
    }
    for case in corpus.invalid_create_rooms {
        assert!(
            parse_client_message(&case.value.to_string()).is_err(),
            "{}",
            case.case_id
        );
        assert!(!validator.is_valid(&case.value), "{}", case.case_id);
    }
}

#[test]
fn v14_message_limits_are_independent_and_exact() {
    assert_eq!(MAX_CLIENT_MESSAGE_BYTES, 4_096);
    assert_eq!(MAX_SERVER_MESSAGE_BYTES, 98_304);
    assert_eq!(super::super::MAX_SERVER_MESSAGE_BYTES, 65_536);
    let exact = "x".repeat(MAX_SERVER_MESSAGE_BYTES);
    assert_eq!(enforce_server_message_size(exact.clone()), Ok(exact));
    assert_eq!(
        enforce_server_message_size("x".repeat(MAX_SERVER_MESSAGE_BYTES + 1)),
        Err(super::super::EncodeServerMessageError::MessageTooLarge),
    );
    assert!(parse_client_message(&"x".repeat(MAX_CLIENT_MESSAGE_BYTES + 1)).is_err());
}

#[test]
fn v14_event_and_bomb_bounds_do_not_widen_archived_contracts() {
    assert_eq!(MAX_SNAPSHOT_EVENTS, 192);
    assert_eq!(MAX_EVENT_INDEX, 191);
    assert!(EventIndex::new(191).is_ok());
    assert!(EventIndex::new(192).is_err());
    assert_eq!(crate::protocol::v11::MAX_BOMBS, 16);
    assert_eq!(MAX_BOMBS, 32);
}

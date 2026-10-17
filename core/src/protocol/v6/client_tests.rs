use jsonschema::{Validator, validator_for};
use serde_json::{Value, json};

use super::{GameClientMessage, ParseClientMessageError, ResumeToken, parse_client_message};

const SESSION_TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const RESUME_TOKEN: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
const ROOM_ID: &str = "0123456789abcdef0123456789abcdef";

fn client_schema() -> Validator {
    let mut schema: Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v6.schema.json")).unwrap();
    schema["$ref"] = json!("#/$defs/GameClientMessage");
    validator_for(&schema).unwrap()
}

fn assert_valid(value: &Value) -> GameClientMessage {
    assert!(client_schema().is_valid(value), "schema rejected {value}");
    parse_client_message(&value.to_string()).unwrap()
}

fn assert_invalid(value: &Value) {
    assert!(!client_schema().is_valid(value), "schema accepted {value}");
    assert_eq!(
        parse_client_message(&value.to_string()),
        Err(ParseClientMessageError::InvalidMessage)
    );
}

fn messages() -> [Value; 10] {
    [
        json!({"type":"authenticate", "protocolVersion":6, "token":SESSION_TOKEN}),
        json!({"type":"createRoom", "sequence":1}),
        json!({"type":"joinRoom", "sequence":2, "roomCode":"ABCDEFGH"}),
        json!({"type":"resumeRoom", "sequence":3, "roomId":ROOM_ID, "resumeToken":RESUME_TOKEN}),
        json!({"type":"ready", "sequence":4, "roomId":ROOM_ID, "roundId":1, "ready":true, "settingsRevision":1}),
        json!({"type":"movement", "sequence":5, "roomId":ROOM_ID, "roundId":1, "direction":null}),
        json!({"type":"placeBomb", "sequence":6, "roomId":ROOM_ID, "roundId":1, "actionId":1}),
        json!({"type":"leaveRoom", "sequence":7, "roomId":ROOM_ID, "roundId":1}),
        json!({"type":"rematch", "sequence":8, "roomId":ROOM_ID, "roundId":1, "ready":false}),
        json!({"type":"updateSettings", "sequence":9, "roomId":ROOM_ID, "roundId":1, "settingsRevision":1, "settings":{"startingSpeedLevel":0,"turnAssistUnits":256,"winsToFinish":3,"bombFuseTicks":180}}),
    ]
}

#[test]
fn parses_every_closed_client_variant() {
    for value in messages() {
        assert_valid(&value);
        for field in value.as_object().unwrap().keys() {
            let mut missing = value.clone();
            missing.as_object_mut().unwrap().remove(field);
            assert_invalid(&missing);
        }
        let mut extra = value.clone();
        extra["extra"] = json!(true);
        assert_invalid(&extra);
    }
}

#[test]
fn accepts_bounded_cross_version_authentication_input() {
    for version in [0, 3, 4, u16::MAX] {
        assert_valid(
            &json!({"type":"authenticate", "protocolVersion":version, "token":SESSION_TOKEN}),
        );
    }
    for version in [json!(-1), json!(65_536), json!(1.5), json!(null)] {
        assert_invalid(
            &json!({"type":"authenticate", "protocolVersion":version, "token":SESSION_TOKEN}),
        );
    }
}

#[test]
fn resume_capability_is_strict_and_redacted() {
    for token in [
        "a".repeat(63),
        "a".repeat(65),
        "A".repeat(64),
        "g".repeat(64),
    ] {
        assert_invalid(
            &json!({"type":"resumeRoom", "sequence":1, "roomId":ROOM_ID, "resumeToken":token}),
        );
    }
    let token = ResumeToken::new(RESUME_TOKEN).unwrap();
    assert_eq!(token.as_str(), RESUME_TOKEN);
    assert_eq!(format!("{token:?}"), "ResumeToken([REDACTED])");
    assert!(!format!("{token:?}").contains(RESUME_TOKEN));
    let message = assert_valid(
        &json!({"type":"resumeRoom", "sequence":1, "roomId":ROOM_ID, "resumeToken":RESUME_TOKEN}),
    );
    assert!(!format!("{message:?}").contains(RESUME_TOKEN));
}

#[test]
fn counters_nullable_direction_and_size_limit_remain_strict() {
    for field in ["sequence", "roundId"] {
        for invalid in [json!(0), json!(2_147_483_648_u64), json!(1.5), json!(null)] {
            let mut message = messages()[8].clone();
            message[field] = invalid;
            assert_invalid(&message);
        }
    }
    let mut missing = messages()[5].clone();
    missing.as_object_mut().unwrap().remove("direction");
    assert_invalid(&missing);
    let base = "{\"type\":\"createRoom\",\"sequence\":1}";
    let at_limit = format!("{base}{}", " ".repeat(4_096 - base.len()));
    assert_eq!(at_limit.len(), 4_096);
    assert!(parse_client_message(&at_limit).is_ok());
    assert_eq!(
        parse_client_message(&(at_limit + " ")),
        Err(ParseClientMessageError::InvalidMessage)
    );
}

use jsonschema::{Validator, validator_for};
use serde_json::{Value, json};

use super::{GameClientMessage, ParseClientMessageError, parse_client_message};

const TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const ROOM_ID: &str = "0123456789abcdef0123456789abcdef";

fn client_schema() -> Validator {
    let mut schema: Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v3.schema.json")).unwrap();
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

fn messages() -> [Value; 8] {
    [
        json!({"type":"authenticate", "protocolVersion":3, "token":TOKEN}),
        json!({"type":"createRoom", "sequence":1}),
        json!({"type":"joinRoom", "sequence":2, "roomCode":"ABCDEFGH"}),
        json!({"type":"ready", "sequence":3, "roomId":ROOM_ID, "roundId":1, "ready":true}),
        json!({"type":"movement", "sequence":4, "roomId":ROOM_ID, "roundId":1, "direction":null}),
        json!({"type":"placeBomb", "sequence":5, "roomId":ROOM_ID, "roundId":1, "actionId":1}),
        json!({"type":"leaveRoom", "sequence":6, "roomId":ROOM_ID, "roundId":1}),
        json!({"type":"rematch", "sequence":7, "roomId":ROOM_ID, "roundId":1, "ready":false}),
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
    for version in [0, 2, 3, u16::MAX] {
        assert_valid(&json!({"type":"authenticate", "protocolVersion":version, "token":TOKEN}));
    }
    for version in [json!(-1), json!(65_536), json!(1.5), json!(null)] {
        assert_invalid(&json!({"type":"authenticate", "protocolVersion":version, "token":TOKEN}));
    }
}

#[test]
fn counters_and_required_nullable_direction_remain_strict() {
    for field in ["sequence", "roundId"] {
        for invalid in [json!(0), json!(2_147_483_648_u64), json!(1.5), json!(null)] {
            let mut message = messages()[7].clone();
            message[field] = invalid;
            assert_invalid(&message);
        }
    }
    let mut missing = messages()[4].clone();
    missing.as_object_mut().unwrap().remove("direction");
    assert_invalid(&missing);
}

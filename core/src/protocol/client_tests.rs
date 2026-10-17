use jsonschema::{Validator, validator_for};
use serde_json::{Value, json};

use super::{
    Counter, Direction, GameClientMessage, MAX_CLIENT_MESSAGE_BYTES, ParseClientMessageError,
    parse_client_message,
};

const TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const ROOM_ID: &str = "0123456789abcdef0123456789abcdef";

fn client_schema() -> Validator {
    let mut schema: Value =
        serde_json::from_str(include_str!("../../../api/gameplay.schema.json")).unwrap();
    schema["$ref"] = json!("#/$defs/GameClientMessage");
    validator_for(&schema).unwrap()
}

fn assert_valid(raw: &str) -> GameClientMessage {
    let value: Value = serde_json::from_str(raw).unwrap();
    assert!(client_schema().is_valid(&value), "schema rejected {raw}");
    parse_client_message(raw).unwrap()
}

fn assert_invalid(raw: &str) {
    let value: Value = serde_json::from_str(raw).unwrap();
    assert!(!client_schema().is_valid(&value), "schema accepted {raw}");
    assert_eq!(
        parse_client_message(raw),
        Err(ParseClientMessageError::InvalidMessage)
    );
}

fn valid_messages() -> [Value; 7] {
    [
        json!({"type":"authenticate", "protocolVersion":1, "token":TOKEN}),
        json!({"type":"createRoom", "sequence":1}),
        json!({"type":"joinRoom", "sequence":2, "roomCode":"ABCDEFGH"}),
        json!({
            "type":"ready", "sequence":3, "roomId":ROOM_ID, "roundId":1, "ready":true
        }),
        json!({
            "type":"movement", "sequence":4, "roomId":ROOM_ID, "roundId":1,
            "direction":"north"
        }),
        json!({
            "type":"placeBomb", "sequence":5, "roomId":ROOM_ID, "roundId":1,
            "actionId":1
        }),
        json!({"type":"leaveRoom", "sequence":6, "roomId":ROOM_ID, "roundId":1}),
    ]
}

#[test]
fn parses_every_closed_client_variant_against_the_schema() {
    for value in valid_messages() {
        let raw = value.to_string();
        assert_valid(&raw);

        let mut missing = value.clone();
        missing.as_object_mut().unwrap().remove("type");
        assert_invalid(&missing.to_string());

        let mut extra = value.clone();
        extra["extra"] = json!(true);
        assert_invalid(&extra.to_string());
    }
}

#[test]
fn every_variant_rejects_missing_and_wrong_typed_payload_fields() {
    let cases = [
        (0, "token"),
        (1, "sequence"),
        (2, "roomCode"),
        (3, "ready"),
        (4, "direction"),
        (5, "actionId"),
        (6, "roundId"),
    ];
    for (index, field) in cases {
        let mut missing = valid_messages()[index].clone();
        missing.as_object_mut().unwrap().remove(field);
        assert_invalid(&missing.to_string());

        let mut wrong = valid_messages()[index].clone();
        wrong[field] = json!([]);
        assert_invalid(&wrong.to_string());
    }
}

#[test]
fn bounded_numbers_accept_json_integer_forms_and_reject_other_values() {
    for raw in [
        format!(r#"{{"type":"authenticate","protocolVersion":0,"token":"{TOKEN}"}}"#),
        format!(r#"{{"type":"authenticate","protocolVersion":65535,"token":"{TOKEN}"}}"#),
        format!(r#"{{"type":"authenticate","protocolVersion":2,"token":"{TOKEN}"}}"#),
        r#"{"type":"createRoom","sequence":1.0}"#.to_owned(),
        r#"{"type":"createRoom","sequence":1e0}"#.to_owned(),
        r#"{"type":"createRoom","sequence":2147483647}"#.to_owned(),
    ] {
        assert_valid(&raw);
    }

    for raw in [
        format!(r#"{{"type":"authenticate","protocolVersion":-1,"token":"{TOKEN}"}}"#),
        format!(r#"{{"type":"authenticate","protocolVersion":65536,"token":"{TOKEN}"}}"#),
        format!(r#"{{"type":"authenticate","protocolVersion":1.5,"token":"{TOKEN}"}}"#),
        format!(r#"{{"type":"authenticate","protocolVersion":true,"token":"{TOKEN}"}}"#),
        r#"{"type":"createRoom","sequence":0}"#.to_owned(),
        r#"{"type":"createRoom","sequence":2147483648}"#.to_owned(),
        r#"{"type":"createRoom","sequence":1.5}"#.to_owned(),
        r#"{"type":"createRoom","sequence":"1"}"#.to_owned(),
    ] {
        assert_invalid(&raw);
    }
}

#[test]
fn movement_direction_is_required_but_nullable() {
    let null = format!(
        r#"{{"type":"movement","sequence":1,"roomId":"{ROOM_ID}","roundId":1,"direction":null}}"#
    );
    assert!(matches!(
        assert_valid(&null),
        GameClientMessage::Movement {
            direction: None,
            ..
        }
    ));

    let missing = format!(r#"{{"type":"movement","sequence":1,"roomId":"{ROOM_ID}","roundId":1}}"#);
    assert_invalid(&missing);
}

#[test]
fn canonical_secret_room_and_invitation_strings_are_enforced() {
    for invalid_token in [
        &TOKEN[..63],
        "g123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "0123456789ABCDEF0123456789abcdef0123456789abcdef0123456789abcdef",
    ] {
        assert_invalid(
            &json!({"type":"authenticate", "protocolVersion":1, "token":invalid_token}).to_string(),
        );
    }
    for room_id in [
        "0123456789abcdef0123456789abcde",
        "G123456789abcdef0123456789abcdef",
    ] {
        assert_invalid(
            &json!({
                "type":"leaveRoom", "sequence":1, "roomId":room_id, "roundId":1
            })
            .to_string(),
        );
    }
    for room_code in ["ABCDEFGI", "ABCDEFG1", "abcdefgh"] {
        assert_invalid(&json!({"type":"joinRoom", "sequence":1, "roomCode":room_code}).to_string());
    }
}

#[test]
fn older_version_is_structurally_available_for_compatibility_handling() {
    let parsed = assert_valid(&format!(
        r#"{{"type":"authenticate","protocolVersion":1,"token":"{TOKEN}"}}"#
    ));
    assert!(matches!(
        parsed,
        GameClientMessage::Authenticate {
            protocol_version,
            ..
        } if protocol_version.get() == 1
    ));
}

#[test]
fn token_debug_and_generic_parse_errors_never_expose_payloads() {
    let message = assert_valid(&format!(
        r#"{{"type":"authenticate","protocolVersion":1,"token":"{TOKEN}"}}"#
    ));
    assert!(!format!("{message:?}").contains(TOKEN));

    let malformed = format!(r#"{{"type":"createRoom","sequence":"{TOKEN}"}}"#);
    let error = parse_client_message(&malformed).unwrap_err();
    assert!(!format!("{error:?} {error}").contains(TOKEN));
    assert_eq!(error, ParseClientMessageError::InvalidMessage);

    let valid = format!(r#"{{"type":"authenticate","protocolVersion":1,"token":"{TOKEN}"}}"#);
    let at_limit = format!(
        "{}{}",
        " ".repeat(MAX_CLIENT_MESSAGE_BYTES - valid.len()),
        valid
    );
    assert!(parse_client_message(&at_limit).is_ok());
    let oversized = format!(" {at_limit}");
    assert_eq!(
        parse_client_message(&oversized),
        Err(ParseClientMessageError::InvalidMessage)
    );
}

#[test]
fn parsed_counter_and_direction_values_are_typed() {
    let message = assert_valid(r#"{"type":"createRoom","sequence":7}"#);
    assert_eq!(
        message,
        GameClientMessage::CreateRoom {
            sequence: Counter::new(7).unwrap()
        }
    );
    let message = assert_valid(&format!(
        r#"{{"type":"movement","sequence":1,"roomId":"{ROOM_ID}","roundId":1,"direction":"west"}}"#
    ));
    assert!(matches!(
        message,
        GameClientMessage::Movement {
            direction: Some(Direction::West),
            ..
        }
    ));
}

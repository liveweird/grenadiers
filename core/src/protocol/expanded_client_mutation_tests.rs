use std::panic::{AssertUnwindSafe, catch_unwind};

use jsonschema::Validator;
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};
use serde_json::{Value, json};

use super::expanded_client_contract_tests::{client_schemas, corpus, parse_v8, parsers_accept};

fn assert_rejected(case_id: &str, value: &Value, schemas: &[Validator]) {
    let raw = serde_json::to_string(value).unwrap();
    for accepted in parsers_accept(&raw) {
        assert!(!accepted, "{case_id}");
    }
    for schema in schemas {
        assert!(!schema.is_valid(value), "{case_id}");
    }
}

fn incompatible_value(value: &Value) -> Value {
    match value {
        Value::Null => json!(false),
        Value::Bool(_) => json!(0),
        Value::Number(_) => json!("wrong-type"),
        Value::String(_) => json!(0),
        Value::Array(_) => json!({}),
        Value::Object(_) => json!([]),
    }
}

#[test]
fn every_corpus_variant_rejects_each_missing_required_field() {
    let corpus = corpus();
    let schemas = client_schemas();
    for case in &corpus.valid_cases {
        for field in case.value.as_object().unwrap().keys() {
            let mut changed = case.value.clone();
            changed.as_object_mut().unwrap().remove(field);
            assert_rejected(&case.case_id, &changed, &schemas);
        }
        if let Some(settings) = case.value.get("settings").and_then(Value::as_object) {
            for field in settings.keys() {
                let mut changed = case.value.clone();
                changed["settings"].as_object_mut().unwrap().remove(field);
                assert_rejected(&case.case_id, &changed, &schemas);
            }
        }
    }
}

#[test]
fn every_corpus_variant_rejects_unknown_fields_and_wrong_field_types() {
    let corpus = corpus();
    let schemas = client_schemas();
    for case in &corpus.valid_cases {
        let mut extra = case.value.clone();
        extra["unexpected"] = json!(true);
        assert_rejected(&case.case_id, &extra, &schemas);

        for (field, original) in case.value.as_object().unwrap() {
            let mut changed = case.value.clone();
            changed[field] = incompatible_value(original);
            assert_rejected(&case.case_id, &changed, &schemas);
        }

        if let Some(settings) = case.value.get("settings").and_then(Value::as_object) {
            let mut extra_setting = case.value.clone();
            extra_setting["settings"]["unexpected"] = json!(true);
            assert_rejected(&case.case_id, &extra_setting, &schemas);
            for (field, original) in settings {
                let mut changed = case.value.clone();
                changed["settings"][field] = incompatible_value(original);
                assert_rejected(&case.case_id, &changed, &schemas);
            }
        }
    }
}

#[test]
fn numeric_and_settings_boundaries_are_exercised_on_both_sides() {
    let schemas = client_schemas();
    let room_id = "0123456789abcdef0123456789abcdef";
    let base = json!({
        "type": "updateSettings",
        "sequence": 1,
        "roomId": room_id,
        "roundId": 2,
        "settingsRevision": 3,
        "settings": {
            "startingSpeedLevel": 0,
            "turnAssistUnits": 256,
            "winsToFinish": 1,
            "bombFuseTicks": 60,
            "enabledItems": []
        }
    });
    for (field, accepted) in [
        ("startingSpeedLevel", json!(4)),
        ("turnAssistUnits", json!(64)),
        ("winsToFinish", json!(9)),
        ("bombFuseTicks", json!(300)),
    ] {
        let mut changed = base.clone();
        changed["settings"][field] = accepted;
        let raw = serde_json::to_string(&changed).unwrap();
        assert!(
            parsers_accept(&raw).into_iter().all(|result| result),
            "{field}"
        );
        assert!(
            schemas.iter().all(|schema| schema.is_valid(&changed)),
            "{field}"
        );
    }
    for (field, rejected) in [
        ("startingSpeedLevel", json!(5)),
        ("turnAssistUnits", json!(63)),
        ("winsToFinish", json!(10)),
        ("bombFuseTicks", json!(301)),
    ] {
        let mut changed = base.clone();
        changed["settings"][field] = rejected;
        assert_rejected(field, &changed, &schemas);
    }
}

#[test]
fn legal_json_representations_preserve_the_parsed_command() {
    let room = "0123456789abcdef0123456789abcdef";
    let canonical = format!(
        r#"{{"type":"movement","sequence":1,"roomId":"{room}","roundId":2,"direction":"west"}}"#
    );
    let expected = parse_v8(&canonical).unwrap();
    let representations = [
        format!(
            r#"{{"direction":"west","roundId":2,"roomId":"{room}","sequence":1,"type":"movement"}}"#
        ),
        format!(
            " {{ \"type\" : \"movement\", \"sequence\" : 1, \"roomId\" : \"{room}\", \"roundId\" : 2, \"direction\" : \"west\" }} \n"
        ),
        format!(
            r#"{{"\u0074ype":"\u006dovement","sequence":1,"roomId":"{room}","roundId":2,"direction":"\u0077est"}}"#
        ),
        format!(
            r#"{{"type":"movement","sequence":1.0,"roomId":"{room}","roundId":2e0,"direction":"west"}}"#
        ),
    ];
    let schemas = client_schemas();
    for raw in representations {
        assert_eq!(parse_v8(&raw).unwrap(), expected, "legal-representation");
        assert_eq!(super::v9::parse_client_message(&raw).unwrap(), expected);
        assert_eq!(super::v10::parse_client_message(&raw).unwrap(), expected);
        assert_eq!(super::v11::parse_client_message(&raw).unwrap(), expected);
        let value: Value = serde_json::from_str(&raw).unwrap();
        assert!(schemas.iter().all(|schema| schema.is_valid(&value)));
    }
}

#[test]
fn raw_input_guards_cover_duplicates_truncation_trailing_data_and_bytes() {
    let duplicated = r#"{"type":"createRoom","sequence":1,"sequence":2}"#;
    let truncated = r#"{"type":"createRoom","sequence":1"#;
    let trailing = r#"{"type":"createRoom","sequence":1}x"#;
    for raw in [duplicated, truncated, trailing] {
        assert!(parsers_accept(raw).into_iter().all(|accepted| !accepted));
    }

    let command = r#"{"type":"createRoom","sequence":1}"#;
    let at_limit = format!(
        "{command}{}",
        " ".repeat(super::MAX_CLIENT_MESSAGE_BYTES - command.len())
    );
    assert_eq!(at_limit.len(), super::MAX_CLIENT_MESSAGE_BYTES);
    assert!(
        parsers_accept(&at_limit)
            .into_iter()
            .all(|accepted| accepted)
    );
    let over_limit = format!("{at_limit} ");
    assert_eq!(over_limit.len(), super::MAX_CLIENT_MESSAGE_BYTES + 1);
    assert!(
        parsers_accept(&over_limit)
            .into_iter()
            .all(|accepted| !accepted)
    );
}

#[test]
fn deeply_nested_invalid_input_is_rejected_without_panicking() {
    // Client commands have no recursive field, so this is no-panic evidence for deeply nested
    // invalid input, not an isolated assertion about serde_json's recursion-depth guard.
    let deep = format!("{}0{}", "[".repeat(160), "]".repeat(160));
    let outcome = catch_unwind(AssertUnwindSafe(|| parsers_accept(&deep)));
    assert_eq!(outcome.unwrap(), [false; 4]);
}

#[test]
fn fixed_seed_string_and_byte_perturbations_never_panic() {
    let mut rng = ChaCha20Rng::from_seed([0x73; 32]);
    for _ in 0..96 {
        let length = (rng.next_u32() as usize) % 4_200;
        let mut bytes = vec![0_u8; length];
        rng.fill_bytes(&mut bytes);
        let arbitrary = String::from_utf8_lossy(&bytes);
        let outcome = catch_unwind(AssertUnwindSafe(|| parsers_accept(&arbitrary)));
        assert!(outcome.is_ok(), "fixed-seed-perturbation");
    }
}

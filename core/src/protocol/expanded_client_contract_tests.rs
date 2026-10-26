use std::collections::BTreeSet;

use jsonschema::{Validator, validator_for};
use serde::Deserialize;
use serde_json::{Value, json};

use super::v8::GameClientMessage;

const CORPUS: &str =
    include_str!("../../../api/fixtures/gameplay-expanded-client-conformance.json");

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Corpus {
    pub(super) valid_cases: Vec<ValidCase>,
    pub(super) invalid_cases: Vec<InvalidCase>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ValidCase {
    pub(super) case_id: String,
    pub(super) variant: String,
    pub(super) value: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct InvalidCase {
    pub(super) case_id: String,
    pub(super) value: Value,
}

pub(super) fn corpus() -> Corpus {
    serde_json::from_str(CORPUS).expect("expanded client conformance corpus must be valid JSON")
}

pub(super) fn client_schemas() -> Vec<Validator> {
    [
        include_str!("../../../api/gameplay-v8.schema.json"),
        include_str!("../../../api/gameplay-v9.schema.json"),
        include_str!("../../../api/gameplay-v10.schema.json"),
        include_str!("../../../api/gameplay-v11.schema.json"),
    ]
    .into_iter()
    .map(|raw| {
        let mut schema: Value = serde_json::from_str(raw).unwrap();
        schema["$ref"] = json!("#/$defs/GameClientMessage");
        validator_for(&schema).unwrap()
    })
    .collect()
}

pub(super) fn parsers_accept(raw: &str) -> [bool; 4] {
    [
        super::v8::parse_client_message(raw).is_ok(),
        super::v9::parse_client_message(raw).is_ok(),
        super::v10::parse_client_message(raw).is_ok(),
        super::v11::parse_client_message(raw).is_ok(),
    ]
}

pub(super) fn parse_v8(raw: &str) -> Result<GameClientMessage, ()> {
    super::v8::parse_client_message(raw).map_err(|_| ())
}

#[test]
fn shared_corpus_covers_the_actual_expanded_client_union() {
    let corpus = corpus();
    let actual = corpus
        .valid_cases
        .iter()
        .map(|case| case.variant.as_str())
        .collect::<BTreeSet<_>>();
    let expected = BTreeSet::from([
        "authenticate",
        "createRoom",
        "joinRoom",
        "leaveRoom",
        "manualAction",
        "movement",
        "ready",
        "rematch",
        "resumeRoom",
        "updateSettings",
    ]);
    assert_eq!(actual, expected);
    for case in &corpus.valid_cases {
        assert_eq!(case.value["type"].as_str(), Some(case.variant.as_str()));
    }
}

#[test]
fn shared_corpus_agrees_across_all_rust_parsers_and_schemas() {
    let corpus = corpus();
    let schemas = client_schemas();
    for case in &corpus.valid_cases {
        let raw = serde_json::to_string(&case.value).unwrap();
        for accepted in parsers_accept(&raw) {
            assert!(accepted, "{}", case.case_id);
        }
        for schema in &schemas {
            assert!(schema.is_valid(&case.value), "{}", case.case_id);
        }
    }
    for case in &corpus.invalid_cases {
        let raw = serde_json::to_string(&case.value).unwrap();
        for accepted in parsers_accept(&raw) {
            assert!(!accepted, "{}", case.case_id);
        }
        for schema in &schemas {
            assert!(!schema.is_valid(&case.value), "{}", case.case_id);
        }
    }
}

#[test]
fn retired_place_bomb_case_is_complete_under_the_historical_v6_contract() {
    let corpus = corpus();
    let case = corpus
        .invalid_cases
        .iter()
        .find(|case| case.case_id == "retired-place-bomb-command")
        .unwrap();
    let raw = serde_json::to_string(&case.value).unwrap();
    assert!(
        super::v6::parse_client_message(&raw).is_ok(),
        "{}",
        case.case_id
    );

    let mut schema: Value =
        serde_json::from_str(include_str!("../../../api/gameplay-v6.schema.json")).unwrap();
    schema["$ref"] = json!("#/$defs/GameClientMessage");
    assert!(
        validator_for(&schema).unwrap().is_valid(&case.value),
        "{}",
        case.case_id
    );
}

#[test]
fn every_retained_schema_has_the_same_client_variant_set() {
    let expected = BTreeSet::from([
        "#/$defs/Authenticate",
        "#/$defs/CreateRoom",
        "#/$defs/JoinRoom",
        "#/$defs/LeaveRoom",
        "#/$defs/ManualAction",
        "#/$defs/Movement",
        "#/$defs/Ready",
        "#/$defs/Rematch",
        "#/$defs/ResumeRoom",
        "#/$defs/UpdateSettingsCommand",
    ]);
    for schema in [
        include_str!("../../../api/gameplay-v8.schema.json"),
        include_str!("../../../api/gameplay-v9.schema.json"),
        include_str!("../../../api/gameplay-v10.schema.json"),
        include_str!("../../../api/gameplay-v11.schema.json"),
    ] {
        let schema: Value = serde_json::from_str(schema).unwrap();
        let variants = schema["$defs"]["GameClientMessage"]["oneOf"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["$ref"].as_str().unwrap())
            .collect::<BTreeSet<_>>();
        assert_eq!(variants, expected);
    }
}

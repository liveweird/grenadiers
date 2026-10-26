#[path = "expanded_server_contract_tests/fixtures.rs"]
mod fixtures;

use std::collections::{BTreeMap, BTreeSet};

use fixtures::{Corpus, FrameCase, VersionApi};
use jsonschema::{Validator, validator_for};
use serde_json::{Value, json};

use super::{v8, v9, v10, v11};

const CORPUS: &str =
    include_str!("../../../api/fixtures/gameplay-expanded-server-conformance.json");

fn apis() -> [VersionApi; 4] {
    [
        VersionApi {
            version: 8,
            project_room: v8::project_expanded_room,
            capture: v8::capture_expanded,
            bind: v8::bind_expanded,
            prepare_pair: v8::prepare_snapshot_pair,
            prepare_admission: v8::prepare_admission,
        },
        VersionApi {
            version: 9,
            project_room: v9::project_expanded_room,
            capture: v9::capture_expanded,
            bind: v9::bind_expanded,
            prepare_pair: v9::prepare_snapshot_pair,
            prepare_admission: v9::prepare_admission,
        },
        VersionApi {
            version: 10,
            project_room: v10::project_expanded_room,
            capture: v10::capture_expanded,
            bind: v10::bind_expanded,
            prepare_pair: v10::prepare_snapshot_pair,
            prepare_admission: v10::prepare_admission,
        },
        VersionApi {
            version: 11,
            project_room: v11::project_expanded_room,
            capture: v11::capture_expanded,
            bind: v11::bind_expanded,
            prepare_pair: v11::prepare_snapshot_pair,
            prepare_admission: v11::prepare_admission,
        },
    ]
}

fn actual_cases() -> Vec<FrameCase> {
    let [v8, v9, v10, v11] = apis();
    let mut cases = vec![fixtures::waiting_case(
        v8,
        crate::game::EnabledItems::INVISIBILITY_CLOAK,
    )];
    cases.extend(fixtures::privacy_pair(
        v8,
        crate::game::EnabledItems::INVISIBILITY_CLOAK,
        "runningPrivacy",
    ));
    cases.push(fixtures::waiting_case(
        v9,
        crate::game::EnabledItems::STOPWATCH,
    ));
    cases.extend(fixtures::frozen_pair(v9));
    cases.push(fixtures::waiting_case(
        v10,
        crate::game::EnabledItems::TELEPORTER,
    ));
    cases.extend(fixtures::teleport_pairs(v10));
    cases.push(fixtures::waiting_case(
        v11,
        crate::game::EnabledItems::OIL_BARREL,
    ));
    cases.extend(fixtures::oil_pairs(v11));
    cases
}

fn corpus() -> Corpus {
    serde_json::from_str(CORPUS).expect("expanded server conformance corpus must be valid JSON")
}

fn server_schema(version: u16) -> Validator {
    let raw = match version {
        8 => include_str!("../../../api/gameplay-v8.schema.json"),
        9 => include_str!("../../../api/gameplay-v9.schema.json"),
        10 => include_str!("../../../api/gameplay-v10.schema.json"),
        11 => include_str!("../../../api/gameplay-v11.schema.json"),
        _ => panic!("unsupported fixture version {version}"),
    };
    let mut schema: Value = serde_json::from_str(raw).unwrap();
    schema["$ref"] = json!("#/$defs/GameServerMessage");
    validator_for(&schema).unwrap()
}

#[derive(Debug, PartialEq)]
struct SemanticCase {
    version: u16,
    scenario: String,
    recipient: Option<super::Seat>,
    frame: Value,
}

fn semantic_frames(cases: Vec<FrameCase>) -> BTreeMap<String, SemanticCase> {
    let mut frames = BTreeMap::new();
    for case in cases {
        let semantic = SemanticCase {
            version: case.version,
            scenario: case.scenario,
            recipient: case.recipient,
            frame: serde_json::from_str(&case.frame).unwrap(),
        };
        assert!(
            frames.insert(case.case_id.clone(), semantic).is_none(),
            "duplicate expanded server fixture case ID: {}",
            case.case_id
        );
    }
    frames
}

#[test]
fn checked_corpus_matches_actual_archived_projection_and_encoding() {
    let expected = semantic_frames(corpus().cases);
    let actual = semantic_frames(actual_cases());
    assert_eq!(actual, expected);
}

#[test]
fn every_checked_frame_satisfies_its_retained_server_schema() {
    for case in corpus().cases {
        let value: Value = serde_json::from_str(&case.frame).unwrap();
        assert!(
            server_schema(case.version).is_valid(&value),
            "{}",
            case.case_id
        );
    }
}

#[test]
fn corpus_covers_waiting_and_both_running_recipients_per_version() {
    let cases = corpus().cases;
    for version in 8..=11 {
        assert!(
            cases
                .iter()
                .any(|case| { case.version == version && case.scenario == "waitingAdmission" })
        );
        let recipients = cases
            .iter()
            .filter(|case| case.version == version && case.scenario.starts_with("running"))
            .filter_map(|case| case.recipient.map(|seat| format!("{seat:?}")))
            .collect::<BTreeSet<_>>();
        assert_eq!(recipients, BTreeSet::from(["One".into(), "Two".into()]));
    }
}

#[test]
fn changed_physical_state_is_rejected_by_the_preceding_archive() {
    for (scenario, prior) in [
        ("runningFrozen", 8),
        ("runningTeleported", 9),
        ("runningOilMaterialized", 10),
    ] {
        for case in corpus()
            .cases
            .iter()
            .filter(|case| case.scenario == scenario)
        {
            let value: Value = serde_json::from_str(&case.frame).unwrap();
            assert!(!server_schema(prior).is_valid(&value), "{}", case.case_id);
        }
    }
}

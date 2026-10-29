use crate::game::{
    ActorInit, CellId, Direction, EntityId, Position, RoundAssignment, RulesConfig, SeatId,
    SimulationSeed, SpeedLevel, State, StormCloudConfig, Tick, World, fixed_arena,
};

use super::*;

fn state(cloud: StormCloudConfig) -> State {
    let actors = [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(CellId::new(7, 6)),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        },
        ActorInit {
            id: EntityId(2),
            seat: SeatId::Two,
            position: Position::center(CellId::new(13, 11)),
            heading: Direction::West,
            speed: SpeedLevel::Base,
        },
    ];
    State::new_at_tick_with_assignment_and_seed(
        RulesConfig::environment_v7(crate::room::RoomSettings::DEFAULT, cloud),
        World::new(fixed_arena()),
        actors,
        Tick(0),
        RoundAssignment::first(),
        SimulationSeed::new([7; 32]),
    )
    .unwrap()
}

#[test]
fn rules_seven_capture_publishes_cloud_and_actual_remaining_timer() {
    let mut state = state(StormCloudConfig::Single);
    let report = state.step(&[]).unwrap();
    let capture = capture_expanded(&state, &[report]).unwrap();
    assert!(capture.0.world.storm_cloud.is_some());
    assert_eq!(capture.0.world.actors[0].storm_slow_remaining_ticks, 180);
    assert_eq!(capture.0.world.actors[1].storm_slow_remaining_ticks, 0);
}

#[test]
fn disabled_capture_requires_null_cloud_and_zero_timers() {
    let capture = capture_expanded(&state(StormCloudConfig::Disabled), &[]).unwrap();
    assert!(capture.0.world.storm_cloud.is_none());
    assert!(
        capture
            .0
            .world
            .actors
            .iter()
            .all(|actor| actor.storm_slow_remaining_ticks == 0)
    );
}

#[test]
fn public_v14_capture_still_rejects_rules_seven() {
    assert_eq!(
        crate::protocol::v14::capture_expanded(&state(StormCloudConfig::Single), &[]),
        Err(ProjectionError::InvalidRules)
    );
}

#[test]
fn client_configuration_requires_strict_environment_shape() {
    let base = r#"{"type":"createRoom","sequence":1,"configuration":{"controllers":["human","human"],"cpuDifficulty":null,"board":{"width":15,"height":13},"environment":{"stormCloud":true}}}"#;
    assert!(parse_client_message(base).is_ok());
    assert!(
        parse_client_message(
            &base.replace("\"stormCloud\":true", "\"stormCloud\":true,\"seed\":7")
        )
        .is_err()
    );
    assert!(
        parse_client_message(&base.replace(",\"environment\":{\"stormCloud\":true}", "")).is_err()
    );
}

#[test]
fn settings_wrapper_requires_environment_without_widening_legacy_settings() {
    let message = r#"{"type":"updateSettings","sequence":1,"roomId":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","roundId":1,"settingsRevision":1,"settings":{"startingSpeedLevel":0,"turnAssistUnits":256,"winsToFinish":3,"bombFuseTicks":180,"enabledItems":[],"environment":{"stormCloud":false}}}"#;
    assert!(parse_client_message(message).is_ok());
    assert!(
        parse_client_message(&message.replace(",\"environment\":{\"stormCloud\":false}", ""))
            .is_err()
    );
    assert!(serde_json::from_str::<crate::protocol::v14::GameClientMessage>(message).is_err());
}

#[test]
fn parser_and_schema_agree_on_v15_client_fixture() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../api/fixtures/gameplay-v15-client-conformance.json"
    ))
    .unwrap();
    let mut schema: serde_json::Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v15.schema.json")).unwrap();
    schema["$ref"] = serde_json::json!("#/$defs/GameClientMessage");
    let validator = jsonschema::validator_for(&schema).unwrap();
    for case in corpus["validMessages"].as_array().unwrap() {
        let value = &case["value"];
        assert!(
            parse_client_message(&value.to_string()).is_ok(),
            "{}",
            case["caseId"]
        );
        assert!(validator.is_valid(value), "{}", case["caseId"]);
    }
    for case in corpus["invalidMessages"].as_array().unwrap() {
        let value = &case["value"];
        assert!(
            parse_client_message(&value.to_string()).is_err(),
            "{}",
            case["caseId"]
        );
        assert!(!validator.is_valid(value), "{}", case["caseId"]);
    }
}

use crate::game::{
    ActorInit, CellId, Direction, EntityId, Position, RoundAssignment, RulesConfig, SeatId,
    SimulationSeed, SpeedLevel, State, StormCloudConfig, Tick, World, fixed_arena,
};
use crate::room::{LairConfig, LairSettings, RoomSettings};

use super::*;

fn state(lairs: LairConfig) -> State {
    let actors = [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(CellId::new(1, 1)),
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
        RulesConfig::environment_v8(RoomSettings::DEFAULT, StormCloudConfig::Disabled, lairs),
        World::new(fixed_arena()),
        actors,
        Tick(0),
        RoundAssignment::first(),
        SimulationSeed::new([16; 32]),
    )
    .unwrap()
}

#[test]
fn parser_retains_lair_numbers_while_disabled_and_is_strict() {
    let valid = r#"{"type":"createRoom","sequence":1,"configuration":{"controllers":["human","human"],"cpuDifficulty":null,"board":{"width":15,"height":13},"environment":{"stormCloud":false,"lairs":{"enabled":false,"spawnSeconds":37,"maxLivingMobs":6}}}}"#;
    let parsed = parse_client_message(valid).unwrap();
    let GameClientMessage::CreateRoom {
        ref configuration, ..
    } = parsed
    else {
        panic!("expected create room")
    };
    assert!(!configuration.domain().lairs().enabled());
    assert_eq!(
        configuration.domain().lairs().settings().spawn_seconds(),
        37
    );
    let encoded = serde_json::to_string(configuration).unwrap();
    assert!(encoded.contains(r#""lairs":{"enabled":false,"spawnSeconds":37,"maxLivingMobs":6}"#));
    let round_trip: RoomConfiguration = serde_json::from_str(&encoded).unwrap();
    assert_eq!(round_trip.domain(), configuration.domain());
    assert!(parse_client_message(&valid.replace("37", "4")).is_err());
    assert!(parse_client_message(&valid.replace("\"lairs\":{", "\"seed\":7,\"lairs\":{")).is_err());
}

#[test]
fn capture_uses_real_rules_eight_lair_state_without_widening_v15() {
    let disabled = state(LairConfig::Disabled);
    let capture = capture_expanded(&disabled, &[]).unwrap();
    assert!(capture.lairs.is_empty());
    assert!(capture.mobs.is_empty());
    assert!(matches!(
        crate::protocol::v15::capture_expanded(&disabled, &[]),
        Err(ProjectionError::InvalidRules)
    ));

    let enabled = state(LairConfig::Enabled(LairSettings::new(5, 2).unwrap()));
    let capture = capture_expanded(&enabled, &[]).unwrap();
    assert_eq!(capture.lairs.len(), 2);
    assert_eq!(capture.lairs[0].id, 0);
    assert_eq!(capture.lairs[1].id, 1);
    assert_eq!(capture.lairs[0].spawn_remaining_ticks, 300);
    assert!(capture.mobs.is_empty());
}

#[test]
fn parser_and_schema_agree_on_v16_client_corpus() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../api/fixtures/gameplay-v16-client-conformance.json"
    ))
    .unwrap();
    let mut schema: serde_json::Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v16.schema.json")).unwrap();
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

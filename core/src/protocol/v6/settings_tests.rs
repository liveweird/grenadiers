use jsonschema::validator_for;
use serde_json::{Value, json};

use super::projection_test_support::{metadata, occupied};
use super::{GameSettings, RoomProjectionError, validate_room_view};

fn settings_schema() -> jsonschema::Validator {
    let mut schema: Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v6.schema.json")).unwrap();
    schema["$ref"] = json!("#/$defs/GameSettings");
    validator_for(&schema).unwrap()
}

#[test]
fn settings_parser_and_schema_agree_on_bounds_steps_and_numeric_types() {
    let defaults =
        json!({"startingSpeedLevel":0,"turnAssistUnits":256,"winsToFinish":3,"bombFuseTicks":180});
    let validator = settings_schema();
    for (field, valid, invalid) in [
        ("startingSpeedLevel", vec![0, 4], vec![-1, 5]),
        (
            "turnAssistUnits",
            vec![64, 128, 192, 256],
            vec![-64, 0, 1, 255, 320],
        ),
        ("winsToFinish", vec![1, 9], vec![0, 10]),
        (
            "bombFuseTicks",
            vec![60, 90, 180, 300],
            vec![0, 59, 61, 301],
        ),
    ] {
        for (accepted, values) in [(true, valid), (false, invalid)] {
            for value in values {
                let mut candidate = defaults.clone();
                candidate[field] = json!(value);
                assert_eq!(validator.is_valid(&candidate), accepted, "{candidate}");
                let parsed = serde_json::from_value::<GameSettings>(candidate.clone());
                assert_eq!(parsed.is_ok(), accepted, "{candidate}");
                if let Ok(parsed) = parsed {
                    assert_eq!(serde_json::to_value(parsed).unwrap(), candidate);
                }
            }
        }
        for invalid in [Value::Null, json!("1"), json!(1.5), json!(true)] {
            let mut candidate = defaults.clone();
            candidate[field] = invalid;
            assert!(!validator.is_valid(&candidate));
            assert!(serde_json::from_value::<GameSettings>(candidate).is_err());
        }
    }
    // JSON Schema integer accepts integral JSON numbers written with a decimal point.
    let integral = r#"{"startingSpeedLevel":0.0,"turnAssistUnits":256.0,"winsToFinish":3.0,"bombFuseTicks":180.0}"#;
    assert!(validator.is_valid(&serde_json::from_str(integral).unwrap()));
    assert!(serde_json::from_str::<GameSettings>(integral).is_ok());
    let mut extra = defaults.clone();
    extra["seed"] = json!(1);
    assert!(!validator.is_valid(&extra));
    assert!(serde_json::from_value::<GameSettings>(extra).is_err());
}

#[test]
fn room_projection_binds_configurable_settings_and_target_to_lifecycle() {
    let lifecycle = occupied();
    let mut metadata = metadata();
    metadata.rules_version = crate::protocol::RulesVersion::ConfigurableV4;
    metadata.settings = GameSettings::from_domain(lifecycle.settings());
    let mut room = super::project_room_view(metadata.clone(), &lifecycle).unwrap();
    metadata.settings = GameSettings::from_domain(
        crate::room::settings::RoomSettings::new(crate::game::SpeedLevel::Four, 64, 9, 60).unwrap(),
    );
    assert_eq!(
        super::project_room_view(metadata, &lifecycle),
        Err(RoomProjectionError::InvalidSettings)
    );
    room.scores.one = super::Wins::new(4).unwrap();
    assert_eq!(
        validate_room_view(&room),
        Err(RoomProjectionError::InvalidSettings)
    );
    room.scores.one = super::Wins::new(0).unwrap();
    room.settings_editor = Some(crate::protocol::Seat::One);
    room.seats[0].occupied = false;
    room.seats[0].connection = super::ConnectionStatus::Vacant;
    assert_eq!(
        validate_room_view(&room),
        Err(RoomProjectionError::InvalidSettings)
    );
}

#[test]
fn configurable_round_identity_includes_starting_speed_and_match_target() {
    use crate::game::{RulesConfig, SpeedLevel};
    use crate::room::RoomSettings;

    let defaults = RoomSettings::default();
    let configured = RulesConfig::configurable_v4(defaults);
    assert_eq!(configured.room_settings(), Some(defaults));
    assert_eq!(RulesConfig::classic_v3().room_settings(), None);
    for settings in [
        RoomSettings::new(SpeedLevel::Four, 256, 3, 180).unwrap(),
        RoomSettings::new(SpeedLevel::Base, 256, 9, 180).unwrap(),
    ] {
        // Speed/target do not alter the fuse or turn fields, but still identify the round.
        let other = RulesConfig::configurable_v4(settings);
        assert_eq!(other.bomb_fuse_ticks(), configured.bomb_fuse_ticks());
        assert_eq!(other.turn_assist(), configured.turn_assist());
        assert_ne!(other, configured);
    }
}

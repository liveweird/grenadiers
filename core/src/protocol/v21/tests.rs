use super::*;
use crate::protocol::Counter;

#[test]
fn closed_client_corpus_checks_required_bounded_drop_multiplier() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/fixtures/gameplay-v21-client-conformance.json"
    )))
    .unwrap();
    for case in corpus["validMessages"].as_array().unwrap() {
        let encoded = serde_json::to_string(&case["value"]).unwrap();
        assert!(parse_client_message(&encoded).is_ok(), "{}", case["caseId"]);
    }
    for case in corpus["invalidMessages"].as_array().unwrap() {
        let encoded = serde_json::to_string(&case["value"]).unwrap();
        assert!(
            parse_client_message(&encoded).is_err(),
            "{}",
            case["caseId"]
        );
    }

    for case in corpus["validMessages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["value"]["type"] == "updateSettings")
    {
        let encoded = serde_json::to_string(&case["value"]).unwrap();
        assert!(crate::protocol::v20::parse_client_message(&encoded).is_err());
    }
}

#[test]
fn server_fixture_is_the_v20_shape_with_v21_settings_identity() {
    let mut shared =
        serde_json::to_value(crate::protocol::v19::server_conformance_tests::corpus()).unwrap();
    shared["producer"] = serde_json::Value::String(
        "Rust gameplay v21 typed projection; ContactV2 is a closed transport identity with explicit Bomb Capacity drop weighting".into(),
    );
    shared["validMessages"][0]["caseId"] = serde_json::Value::String("v21-authenticated".into());
    shared["validMessages"][0]["value"]["protocolVersion"] = serde_json::json!(21);
    insert_drop_multiplier(&mut shared);
    let expected: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../api/fixtures/gameplay-v21-server-conformance.json"
    )))
    .unwrap();
    assert_eq!(shared, expected);
    assert_eq!(
        serde_json::to_value(GameServerMessage::Authenticated {
            protocol_version: GameplayVersion,
        })
        .unwrap(),
        serde_json::json!({"type": "authenticated", "protocolVersion": 21})
    );
}

fn insert_drop_multiplier(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Array(values) => {
            for value in values {
                insert_drop_multiplier(value);
            }
        }
        serde_json::Value::Object(fields) => {
            if fields.contains_key("startingSpeedLevel")
                && fields.contains_key("bombFuseTicks")
                && fields.contains_key("enabledItems")
                && fields.contains_key("environment")
            {
                fields.insert("bombCapacityDropMultiplier".into(), serde_json::json!(5));
            }
            for value in fields.values_mut() {
                insert_drop_multiplier(value);
            }
        }
        _ => {}
    }
}

#[test]
fn weighted_binding_accepts_typed_settings_and_old_v20_rejects_them() {
    use crate::game::{
        ActorInit, ArenaGenerationConfig, EnabledItems, PlayerCount, Position, RoundAssignment,
        RulesConfig, SeatId, SpeedLevel, StormCloudConfig, World,
        generate_weighted_multiplayer_arena,
    };
    use crate::room::{
        BoardSize as DomainBoard, BombCapacityDropMultiplier, RespawnLairConfig, RoomSettings,
        SpikePitConfig,
    };

    let multiplier = BombCapacityDropMultiplier::DEFAULT;
    let generated = generate_weighted_multiplayer_arena(
        [21; 32],
        EnabledItems::EMPTY,
        ArenaGenerationConfig {
            player_count: PlayerCount::Two,
            board: DomainBoard::FifteenByThirteen,
            assignment: RoundAssignment::first(),
        },
        multiplier,
    )
    .unwrap();
    let generation = generated.record();
    let (world, _) = World::from_generated(generated).unwrap();
    let spawns = world.arena().spawns().to_vec();
    let actors = SeatId::DUEL.map(|seat| ActorInit {
        id: crate::game::EntityId(21 + seat.index() as u32),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: crate::game::Direction::East,
        speed: SpeedLevel::Base,
    });
    let domain_settings = RoomSettings::DEFAULT.with_bomb_capacity_drop_multiplier(multiplier);
    let rules = RulesConfig::environment_v10(
        domain_settings,
        StormCloudConfig::Disabled,
        RespawnLairConfig::Disabled,
        SpikePitConfig::Disabled,
    )
    .with_punch_activation_policy(crate::game::PunchActivationPolicy::ContactV2);
    let state = crate::game::State::new(rules, world, actors).unwrap();
    let capture = capture_expanded(&state, &[]).unwrap();
    let legacy = crate::protocol::v19::server_conformance_tests::room(
        state.tick().0,
        RoomPhase::Running,
        false,
    );
    let settings = super::RoomSettings::new(
        legacy.settings.base,
        legacy.settings.environment,
        multiplier,
    );
    let room = RoomView::from_v20(legacy, settings);

    bind_expanded(&room, Counter::new(1).unwrap(), &capture, generation).unwrap();
    assert!(
        crate::protocol::v20::bind_expanded(
            &room.legacy_view(),
            Counter::new(1).unwrap(),
            &capture,
            generation,
        )
        .is_err()
    );
}

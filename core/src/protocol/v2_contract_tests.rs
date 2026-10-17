use super::*;
use crate::protocol::{PickupKind, ProtocolVersionInput};

#[test]
fn map_and_rules_variants_conform_and_unknown_values_fail() {
    for rules in [RulesVersion::ClassicV1, RulesVersion::ClassicV2] {
        assert!(definition_schema("RulesVersion").is_valid(&serde_json::to_value(rules).unwrap()));
        assert_eq!(RulesVersion::try_from(rules.get()), Ok(rules));
    }
    for value in [0, 3, u16::MAX] {
        assert!(RulesVersion::try_from(value).is_err());
        assert!(!definition_schema("RulesVersion").is_valid(&json!(value)));
    }
    for map in [MapId::FixedV1, MapId::GeneratedV1] {
        assert!(definition_schema("MapId").is_valid(&serde_json::to_value(map).unwrap()));
    }
    assert!(!definition_schema("MapId").is_valid(&json!("private_seed_map")));
}

#[test]
fn version_rejection_can_describe_older_and_future_servers() {
    for version in [0, 1, 2, 3, u16::MAX] {
        let value = assert_server_valid(&GameServerMessage::IncompatibleProtocol {
            supported_version: ProtocolVersionInput::new(version),
        });
        assert_eq!(value["supportedVersion"], version);
    }
    for version in [-1, 65536] {
        assert!(
            !server_schema()
                .is_valid(&json!({"type":"incompatibleProtocol","supportedVersion":version}))
        );
    }
    assert!(!server_schema().is_valid(&json!({"type":"authenticated","protocolVersion":1})));
}

#[test]
fn collection_event_kinds_are_closed_and_cells_remain_bounded() {
    for kind in [
        PickupKind::BombCapacity,
        PickupKind::BlastRange,
        PickupKind::MovementSpeed,
    ] {
        let event = crate::protocol::EventKind::PickupCollected {
            id: EntityId::new(10),
            seat: Seat::One,
            cell: Cell::new(5, 5).unwrap(),
            kind,
        };
        let value = serde_json::to_value(event).unwrap();
        let schema = definition_schema("EventKind");
        assert!(schema.is_valid(&value));
        for key in ["type", "id", "seat", "cell", "kind"] {
            let mut missing = value.clone();
            missing.as_object_mut().unwrap().remove(key);
            assert!(!schema.is_valid(&missing));
        }
        for (key, invalid) in [
            ("seed", json!("secret")),
            ("kind", json!("unknown")),
            ("cell", json!({"column":15,"row":5})),
        ] {
            let mut changed = value.clone();
            changed[key] = invalid;
            assert!(!schema.is_valid(&changed));
        }
    }
}

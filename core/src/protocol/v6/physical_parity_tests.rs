use serde_json::Value;

#[test]
fn v6_duplicates_unchanged_v3_physical_definitions_exactly() {
    let v6: Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v6.schema.json")).unwrap();
    let v3: Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v3.schema.json")).unwrap();
    for name in [
        "MapId",
        "PickupKind",
        "EntityId",
        "Tick",
        "Seat",
        "Direction",
        "Cell",
        "Position",
        "RoundResult",
        "WinnerResult",
        "DrawResult",
        "Terrain",
        "GameState",
        "PickupView",
        "ArenaView",
        "ActorView",
        "BombView",
        "FlameView",
        "WireEvent",
        "EventKind",
        "BombPlacedEvent",
        "BombDetonatedEvent",
        "ActorEliminatedEvent",
        "RoundEndedEvent",
        "PickupCollectedEventKind",
    ] {
        assert_eq!(v6["$defs"][name], v3["$defs"][name], "{name} drifted");
    }
}

#[test]
fn v6_adds_forgiving_rules_without_changing_archived_rules_contracts() {
    let v6: Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v6.schema.json")).unwrap();
    let v4: Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v4.schema.json")).unwrap();
    assert_eq!(
        v6["$defs"]["RulesVersion"]["enum"],
        serde_json::json!([1, 2, 3, 4])
    );
    assert_eq!(
        v4["$defs"]["RulesVersion"]["enum"],
        serde_json::json!([1, 2])
    );
    assert_eq!(v6["$defs"]["GameplayVersion"]["const"], 6);
    assert_eq!(v4["$defs"]["GameplayVersion"]["const"], 4);
}

#[test]
fn v6_schema_references_only_its_local_definitions() {
    fn visit(value: &Value) {
        match value {
            Value::Object(object) => {
                if let Some(reference) = object.get("$ref") {
                    assert!(reference.as_str().unwrap().starts_with("#/$defs/"));
                }
                object.values().for_each(visit);
            }
            Value::Array(values) => values.iter().for_each(visit),
            _ => {}
        }
    }
    let schema: Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v6.schema.json")).unwrap();
    visit(&schema);
}

#[test]
fn every_v6_object_definition_is_closed_and_requires_all_properties() {
    let schema: Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v6.schema.json")).unwrap();
    for (name, definition) in schema["$defs"].as_object().unwrap() {
        if definition["type"] != "object" {
            continue;
        }
        assert_eq!(definition["additionalProperties"], false, "{name} is open");
        let properties = definition["properties"].as_object().unwrap();
        let required = definition["required"].as_array().unwrap();
        assert_eq!(
            required.len(),
            properties.len(),
            "{name} has optional fields"
        );
        for property in properties.keys() {
            assert!(
                required.iter().any(|required| required == property),
                "{name}.{property} is optional"
            );
        }
    }
}

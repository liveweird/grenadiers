use jsonschema::validator_for;
use serde::Deserialize;
use serde_json::{Value, json};

use super::{
    EffectValidationError, EffectViews, GameClientMessage, GameSettings, ItemMask, ItemSet,
    SettingsContextError, validate_effects, validate_settings_context,
};

fn definition(name: &str) -> jsonschema::Validator {
    let mut schema: Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v7.schema.json")).unwrap();
    schema["$ref"] = json!(format!("#/$defs/{name}"));
    validator_for(&schema).unwrap()
}

#[test]
fn client_union_matches_schema_and_requires_nullable_direction() {
    const TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    const ROOM: &str = "0123456789abcdef0123456789abcdef";
    let messages = [
        json!({"type":"authenticate","protocolVersion":7,"token":TOKEN}),
        json!({"type":"createRoom","sequence":1}),
        json!({"type":"joinRoom","sequence":1,"roomCode":"ABCDEFGH"}),
        json!({"type":"resumeRoom","sequence":1,"roomId":ROOM,"resumeToken":TOKEN}),
        json!({"type":"ready","sequence":1,"roomId":ROOM,"roundId":1,"ready":true,"settingsRevision":1}),
        json!({"type":"updateSettings","sequence":1,"roomId":ROOM,"roundId":1,"settingsRevision":1,
            "settings":{"startingSpeedLevel":0,"turnAssistUnits":256,"winsToFinish":3,"bombFuseTicks":180,"enabledItems":[]}}),
        json!({"type":"movement","sequence":1,"roomId":ROOM,"roundId":1,"direction":null}),
        json!({"type":"manualAction","sequence":1,"roomId":ROOM,"roundId":1,"actionId":1,"action":"bomb"}),
        json!({"type":"leaveRoom","sequence":1,"roomId":ROOM,"roundId":1}),
        json!({"type":"rematch","sequence":1,"roomId":ROOM,"roundId":1,"ready":false}),
    ];
    let schema = definition("GameClientMessage");
    for value in messages {
        assert!(schema.is_valid(&value), "schema rejected {value}");
        assert!(serde_json::from_value::<GameClientMessage>(value.clone()).is_ok());
        for field in value.as_object().unwrap().keys() {
            let mut missing = value.clone();
            missing.as_object_mut().unwrap().remove(field);
            assert!(!schema.is_valid(&missing), "schema accepted {missing}");
            assert!(serde_json::from_value::<GameClientMessage>(missing).is_err());
        }
        let mut extra = value.clone();
        extra["extra"] = json!(true);
        assert!(!schema.is_valid(&extra));
        assert!(serde_json::from_value::<GameClientMessage>(extra).is_err());
    }
    for version in [0, 6, 8, u16::MAX] {
        let value = json!({"type":"authenticate","protocolVersion":version,"token":TOKEN});
        assert!(schema.is_valid(&value));
        assert!(serde_json::from_value::<GameClientMessage>(value).is_ok());
    }
    for version in [json!(-1), json!(65_536), json!(1.5), Value::Null] {
        let value = json!({"type":"authenticate","protocolVersion":version,"token":TOKEN});
        assert!(!schema.is_valid(&value));
        assert!(serde_json::from_value::<GameClientMessage>(value).is_err());
    }
}

#[test]
fn server_authentication_ack_is_fixed_to_v7() {
    let message = super::GameServerMessage::Authenticated {
        protocol_version: super::GameplayVersion,
    };
    let value = serde_json::to_value(message).unwrap();
    assert_eq!(value, json!({"type":"authenticated","protocolVersion":7}));
    assert!(definition("GameServerMessage").is_valid(&value));
}

#[test]
fn effect_timer_and_array_bounds_match_schema() {
    let schema = definition("EffectView");
    for (kind, maximum) in [("shield", 300), ("stopwatch", 180), ("slowFoot", 600)] {
        for ticks in [1, maximum] {
            let value = json!({"kind":kind,"remainingTicks":ticks});
            assert!(schema.is_valid(&value));
            assert!(serde_json::from_value::<super::EffectView>(value).is_ok());
        }
        for ticks in [0, maximum + 1] {
            let value = json!({"kind":kind,"remainingTicks":ticks});
            assert!(!schema.is_valid(&value));
            assert!(serde_json::from_value::<super::EffectView>(value).is_err());
        }
    }
    let valid = json!({"kind":"shield","remainingTicks":1});
    for changed in [
        json!({"kind":"shield"}),
        json!({"kind":"shield","remainingTicks":1,"extra":0}),
    ] {
        assert!(!schema.is_valid(&changed));
        assert!(serde_json::from_value::<super::EffectView>(changed).is_err());
    }
    assert!(serde_json::from_value::<super::EffectView>(valid).is_ok());
    assert!(
        EffectViews::new(vec![
            super::EffectView::new(super::EffectKind::SlowFoot, 1)
                .unwrap();
            16
        ])
        .is_ok()
    );
    assert!(
        EffectViews::new(vec![
            super::EffectView::new(super::EffectKind::SlowFoot, 1)
                .unwrap();
            17
        ])
        .is_err()
    );
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SemanticFixtures {
    effects: Vec<EffectFixture>,
    item_sets: Vec<ItemSetFixture>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EffectFixture {
    name: String,
    value: Value,
    expected_error: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ItemSetFixture {
    name: String,
    enabled_items: ItemSet,
    available_items: ItemSet,
    expected_error: Option<String>,
}

#[test]
fn rust_semantics_match_shared_cross_language_fixtures() {
    let fixtures: SemanticFixtures = serde_json::from_str(include_str!(
        "../../../../api/fixtures/gameplay-v7-semantics.json"
    ))
    .unwrap();
    let effect_schema = definition("EffectView");
    for fixture in fixtures.effects {
        assert!(
            fixture
                .value
                .as_array()
                .unwrap()
                .iter()
                .all(|value| effect_schema.is_valid(value)),
            "structurally invalid semantic fixture {}",
            fixture.name
        );
        let effects: EffectViews = serde_json::from_value(fixture.value).unwrap();
        let actual = validate_effects(&effects).err().map(|error| match error {
            EffectValidationError::DuplicateEffect => "duplicateEffect",
            EffectValidationError::SlowFootLimit => "slowFootLimit",
        });
        assert_eq!(
            actual,
            fixture.expected_error.as_deref(),
            "{}",
            fixture.name
        );
    }
    for fixture in fixtures.item_sets {
        let settings = GameSettings::new(0, 256, 3, 180, fixture.enabled_items).unwrap();
        let actual = validate_settings_context(&settings, fixture.available_items)
            .err()
            .map(|SettingsContextError::UnavailableItem| "unavailableItem");
        assert_eq!(
            actual,
            fixture.expected_error.as_deref(),
            "{}",
            fixture.name
        );
    }
}

#[test]
fn item_mask_set_and_settings_are_checked_and_canonical() {
    assert!(ItemMask::new((1 << 22) - 1).is_ok());
    assert!(ItemMask::new(1 << 22).is_err());
    assert!(serde_json::from_str::<ItemMask>("4194304").is_err());
    let all = ItemSet::from_mask(ItemMask::new((1 << 22) - 1).unwrap());
    let encoded = serde_json::to_value(all).unwrap();
    assert_eq!(encoded.as_array().unwrap().len(), 22);
    assert_eq!(encoded[0], "kick");
    assert_eq!(encoded[21], "teleporter");
    assert!(serde_json::from_value::<ItemSet>(json!(["kick", "kick"])).is_err());

    let schema = definition("GameSettings");
    for fuse in (60..=300).step_by(30) {
        let value = json!({"startingSpeedLevel":4,"turnAssistUnits":64,"winsToFinish":9,
            "bombFuseTicks":fuse,"enabledItems":["kick"]});
        assert!(schema.is_valid(&value));
        assert!(serde_json::from_value::<GameSettings>(value).is_ok());
    }
    for invalid in [59, 61, 301] {
        let value = json!({"startingSpeedLevel":0,"turnAssistUnits":256,"winsToFinish":3,
            "bombFuseTicks":invalid,"enabledItems":[]});
        assert!(!schema.is_valid(&value));
        assert!(serde_json::from_value::<GameSettings>(value).is_err());
    }
}

#[test]
fn unions_required_nulls_and_limits_are_closed_in_schema() {
    let actor = definition("ActorView");
    let visible = json!({"type":"visible","id":1,"seat":"one","position":{"x":0,"y":0},
        "heading":"north","speedLevel":0,"alive":true,"bombCapacity":1,"bombRange":1,
        "liveBombCount":0,"inventory":{"equipment":null,"weapon":null},"effects":[],
        "damageGraceRemainingTicks":60});
    assert!(actor.is_valid(&visible));
    for pointer in ["equipment", "weapon"] {
        let mut missing = visible.clone();
        missing["inventory"]
            .as_object_mut()
            .unwrap()
            .remove(pointer);
        assert!(!actor.is_valid(&missing));
        assert!(
            serde_json::from_value::<super::InventoryView>(missing["inventory"].clone()).is_err()
        );
    }
    let hidden = json!({"type":"hidden","id":1,"seat":"two","alive":false});
    assert!(actor.is_valid(&hidden));
    let mut leaked = hidden;
    leaked["position"] = json!({"x":0,"y":0});
    assert!(!actor.is_valid(&leaked));

    let trigger = definition("BombTrigger");
    for value in [
        json!({"type":"timed","remainingTicks":0,"paused":false}),
        json!({"type":"timed","remainingTicks":600,"paused":true}),
        json!({"type":"remote"}),
        json!({"type":"contact"}),
    ] {
        assert!(trigger.is_valid(&value), "{value}");
    }
    assert!(!trigger.is_valid(&json!({"type":"timed","remainingTicks":601,"paused":false})));
    assert!(!actor.is_valid(&json!({"type":"visible","id":1,"seat":"one"})));
    assert!(super::DamageGraceTicks::new(60).is_ok());
    assert!(super::DamageGraceTicks::new(61).is_err());
    assert!(super::BombTrigger::timed(600, false).is_ok());
    assert!(super::BombTrigger::timed(601, false).is_err());
    let event = super::WireEvent {
        tick: super::super::Tick::new(0).unwrap(),
        index: super::EventIndex::new(0).unwrap(),
        kind: super::EventKind::DamagePrevented {
            id: super::super::EntityId::new(0),
            seat: super::super::Seat::One,
            source: super::DamagePreventionSource::Shield,
        },
    };
    assert!(super::SnapshotEvents::new(vec![event.clone(); 96]).is_ok());
    assert!(super::SnapshotEvents::new(vec![event; 97]).is_err());
}

#[test]
fn every_new_bomb_event_and_pickup_variant_has_the_schema_wire_name() {
    use super::super::{Cell, DrawReason, EntityId, Seat, Tick, WireRoundResult};

    let bomb_schema = definition("BombView");
    for (trigger, blast) in [
        (
            super::BombTrigger::timed(600, true).unwrap(),
            super::BlastKind::Standard,
        ),
        (super::BombTrigger::remote(), super::BlastKind::Piercing),
        (super::BombTrigger::contact(), super::BlastKind::Power),
    ] {
        let bomb = super::BombView {
            id: EntityId::new(1),
            cell: Cell::new(1, 1).unwrap(),
            owner: EntityId::new(2),
            range: 8,
            placed_at: Tick::new(3).unwrap(),
            trigger,
            blast,
        };
        assert!(bomb_schema.is_valid(&serde_json::to_value(bomb).unwrap()));
    }

    let bomb = super::BombView {
        id: EntityId::new(1),
        cell: Cell::new(1, 1).unwrap(),
        owner: EntityId::new(2),
        range: 8,
        placed_at: Tick::new(3).unwrap(),
        trigger: super::BombTrigger::remote(),
        blast: super::BlastKind::Power,
    };
    let events = [
        super::EventKind::BombPlaced { bomb },
        super::EventKind::BombDetonated { bomb },
        super::EventKind::ActorEliminated {
            id: EntityId::new(1),
            seat: Seat::One,
        },
        super::EventKind::PickupCollected {
            id: EntityId::new(1),
            seat: Seat::Two,
            cell: Cell::new(1, 1).unwrap(),
            kind: super::PickupKind::InvisibilityCloak,
        },
        super::EventKind::RoundEnded {
            result: WireRoundResult::Draw {
                reason: DrawReason::Timeout,
            },
        },
        super::EventKind::DamagePrevented {
            id: EntityId::new(1),
            seat: Seat::One,
            source: super::DamagePreventionSource::Heart,
        },
        super::EventKind::DamagePrevented {
            id: EntityId::new(1),
            seat: Seat::One,
            source: super::DamagePreventionSource::Shield,
        },
    ];
    let event_schema = definition("EventKind");
    for event in events {
        assert!(event_schema.is_valid(&serde_json::to_value(event).unwrap()));
    }

    let pickups = [
        super::PickupKind::BombCapacity,
        super::PickupKind::BlastRange,
        super::PickupKind::MovementSpeed,
        super::PickupKind::Kick,
        super::PickupKind::Punch,
        super::PickupKind::Bracelet,
        super::PickupKind::WallPass,
        super::PickupKind::Heart,
        super::PickupKind::RemoteBomb,
        super::PickupKind::PiercingBomb,
        super::PickupKind::PowerBomb,
        super::PickupKind::Landmine,
        super::PickupKind::Shield,
        super::PickupKind::Stopwatch,
        super::PickupKind::InvisibilityCloak,
        super::PickupKind::GoldenFlame,
        super::PickupKind::ShortFuse,
        super::PickupKind::Diarrhea,
        super::PickupKind::SlowFoot,
        super::PickupKind::ShortFlame,
        super::PickupKind::LongFuse,
        super::PickupKind::ReverseControls,
        super::PickupKind::InvisibleBombs,
        super::PickupKind::OilBarrel,
        super::PickupKind::Teleporter,
    ];
    let pickup_schema = definition("PickupKind");
    for pickup in pickups {
        assert!(pickup_schema.is_valid(&serde_json::to_value(pickup).unwrap()));
    }
}

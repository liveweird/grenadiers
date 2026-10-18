use serde_json::Value;

use crate::protocol::{Direction, EntityId, Position, Seat};

use super::*;

const RESERVED_ITEMS: [ItemId; 22] = [
    ItemId::Kick,
    ItemId::Punch,
    ItemId::Bracelet,
    ItemId::WallPass,
    ItemId::Heart,
    ItemId::RemoteBomb,
    ItemId::PiercingBomb,
    ItemId::PowerBomb,
    ItemId::Landmine,
    ItemId::Shield,
    ItemId::Stopwatch,
    ItemId::InvisibilityCloak,
    ItemId::GoldenFlame,
    ItemId::ShortFuse,
    ItemId::Diarrhea,
    ItemId::SlowFoot,
    ItemId::ShortFlame,
    ItemId::LongFuse,
    ItemId::ReverseControls,
    ItemId::InvisibleBombs,
    ItemId::OilBarrel,
    ItemId::Teleporter,
];

#[test]
fn authentication_is_version_nine_and_version_eight_stays_archived() {
    let message = GameServerMessage::Authenticated {
        protocol_version: GameplayVersion,
    };
    assert_eq!(
        serde_json::from_str::<Value>(&encode_server_message(&message).unwrap()).unwrap(),
        serde_json::json!({"type": "authenticated", "protocolVersion": 9})
    );
    let archived = crate::protocol::v8::GameServerMessage::Authenticated {
        protocol_version: crate::protocol::v8::GameplayVersion,
    };
    assert_eq!(
        serde_json::from_str::<Value>(
            &crate::protocol::v8::encode_server_message(&archived).unwrap()
        )
        .unwrap()["protocolVersion"],
        8
    );
}

#[test]
fn foundation_catalog_matches_archived_v8_and_rejects_stopwatch() {
    let catalog = super::foundation_items();
    assert_eq!(catalog.iter().count(), 14);
    assert_eq!(1_usize << catalog.iter().count(), 16_384);
    assert!(!catalog.contains(ItemId::Stopwatch));
    for item in RESERVED_ITEMS {
        let settings = GameSettings::new(0, 256, 3, 180, ItemSet::new([item]).unwrap()).unwrap();
        assert_eq!(
            expanded_settings(settings).is_ok(),
            catalog.contains(item),
            "{item:?}"
        );
    }
    let settings = GameSettings::new(0, 256, 3, 180, catalog).unwrap();
    assert_eq!(
        expanded_settings(settings).unwrap().enabled_items().bits(),
        523_248
    );
}

#[test]
fn foundation_visible_actors_are_explicitly_unfrozen() {
    let actor = crate::protocol::v8::VisibleActorView {
        id: EntityId::new(1),
        seat: Seat::One,
        position: Position::new(1, 1).unwrap(),
        heading: Direction::East,
        speed_level: 0,
        alive: true,
        bomb_capacity: 1,
        bomb_range: 2,
        live_bomb_count: 0,
        inventory: InventoryView {
            equipment: None,
            weapon: None,
        },
        effects: EffectViews::new(Vec::new()).unwrap(),
        damage_grace_remaining_ticks: DamageGraceTicks::new(0).unwrap(),
    };
    let archived =
        serde_json::to_value(crate::protocol::v8::ActorView::from(actor.clone())).unwrap();
    let visible = ActorView::from(super::projection_recipient::visible_actor(actor));
    let hidden = ActorView::from(HiddenActorView {
        id: EntityId::new(2),
        seat: Seat::Two,
        alive: true,
    });
    let visible = serde_json::to_value(visible).unwrap();
    let hidden = serde_json::to_value(hidden).unwrap();
    assert_eq!(visible["voluntaryActionsFrozen"], false);
    assert!(archived.get("voluntaryActionsFrozen").is_none());
    assert_eq!(
        hidden,
        serde_json::json!({"type":"hidden","id":2,"seat":"two","alive":true})
    );
}

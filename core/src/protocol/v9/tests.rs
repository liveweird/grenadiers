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
fn expanded_catalog_activates_stopwatch_and_invisible_bombs_without_changing_reserved_ids() {
    let catalog = super::expanded_items();
    assert_eq!(catalog.iter().count(), 16);
    assert_eq!(1_usize << catalog.iter().count(), 65_536);
    assert!(catalog.contains(ItemId::Stopwatch));
    assert!(catalog.contains(ItemId::InvisibleBombs));
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
        1_048_560
    );
}

#[test]
fn expanded_settings_accept_every_subset_of_the_pinned_sixteen_item_catalog() {
    let bits = [
        16_u32, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16_384, 32_768, 65_536, 131_072,
        262_144, 524_288,
    ];
    assert_eq!(bits.len(), 16);
    for mask in 0_u32..65_536 {
        let enabled = bits.iter().enumerate().fold(0, |value, (index, bit)| {
            if mask & (1 << index) == 0 {
                value
            } else {
                value | bit
            }
        });
        let items = ItemSet::from_mask(ItemMask::new(enabled).unwrap());
        assert_eq!(
            expanded_settings(GameSettings::new(0, 256, 3, 180, items).unwrap())
                .unwrap()
                .enabled_items()
                .bits(),
            enabled
        );
    }
}

#[test]
fn visible_actor_serializes_the_authoritative_frozen_status() {
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
    let visible = ActorView::from(super::projection_recipient::visible_actor(actor, true));
    let hidden = ActorView::from(HiddenActorView {
        id: EntityId::new(2),
        seat: Seat::Two,
        alive: true,
    });
    let visible = serde_json::to_value(visible).unwrap();
    let hidden = serde_json::to_value(hidden).unwrap();
    assert_eq!(visible["voluntaryActionsFrozen"], true);
    assert!(archived.get("voluntaryActionsFrozen").is_none());
    assert_eq!(
        hidden,
        serde_json::json!({"type":"hidden","id":2,"seat":"two","alive":true})
    );
}

use serde_json::Value;
use std::collections::BTreeMap;

use crate::game::{self, ActorInit, EnabledItems, RulesConfig, SeatId, State, World};
use crate::protocol::{Counter, Direction, EntityId, Position, RoomCode, RoomId, Seat, Tick};
use crate::room::RoomSettings;

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
fn authentication_is_version_ten_and_version_nine_stays_archived() {
    let message = GameServerMessage::Authenticated {
        protocol_version: GameplayVersion,
    };
    assert_eq!(
        serde_json::from_str::<Value>(&encode_server_message(&message).unwrap()).unwrap(),
        serde_json::json!({"type": "authenticated", "protocolVersion": 10})
    );
    let archived = crate::protocol::v9::GameServerMessage::Authenticated {
        protocol_version: crate::protocol::v9::GameplayVersion,
    };
    assert_eq!(
        serde_json::from_str::<Value>(
            &crate::protocol::v9::encode_server_message(&archived).unwrap()
        )
        .unwrap()["protocolVersion"],
        9
    );
}

#[test]
fn expanded_catalog_activates_kick_without_changing_reserved_ids() {
    let catalog = super::expanded_items();
    assert_eq!(catalog.iter().count(), 19);
    assert_eq!(1_usize << catalog.iter().count(), 524_288);
    assert!(catalog.contains(ItemId::Stopwatch));
    assert!(catalog.contains(ItemId::InvisibleBombs));
    assert!(catalog.contains(ItemId::Teleporter));
    assert!(catalog.contains(ItemId::WallPass));
    assert!(catalog.contains(ItemId::Kick));
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
        3_145_721
    );
}

#[test]
fn expanded_settings_accept_every_subset_of_the_pinned_nineteen_item_catalog() {
    let bits = [
        1_u32, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16_384, 32_768, 65_536,
        131_072, 262_144, 524_288, 2_097_152,
    ];
    assert_eq!(bits.len(), 19);
    for mask in 0_u32..524_288 {
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
    let visible = ActorView::from(super::projection_recipient::visible_actor(
        actor,
        true,
        Some(crate::protocol::Tick::new(7).unwrap()),
    ));
    let hidden = ActorView::from(HiddenActorView {
        id: EntityId::new(2),
        seat: Seat::Two,
        alive: true,
    });
    let visible = serde_json::to_value(visible).unwrap();
    let hidden = serde_json::to_value(hidden).unwrap();
    assert_eq!(visible["voluntaryActionsFrozen"], true);
    assert_eq!(visible["lastTeleportTick"], 7);
    assert!(archived.get("voluntaryActionsFrozen").is_none());
    assert!(archived.get("lastTeleportTick").is_none());
    assert_eq!(
        hidden,
        serde_json::json!({"type":"hidden","id":2,"seat":"two","alive":true})
    );
}

#[test]
fn visible_actor_serializes_null_position_epoch_as_a_required_field() {
    let actor = VisibleActorView {
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
        voluntary_actions_frozen: false,
        last_teleport_tick: None,
    };
    let encoded = serde_json::to_value(ActorView::from(actor)).unwrap();
    assert_eq!(encoded["lastTeleportTick"], Value::Null);
}

fn teleporter_state(enabled: EnabledItems) -> State {
    let arena = game::fixed_arena();
    let spawns = arena.spawns();
    State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled)),
        World::with_pickups(
            arena,
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(spawns[0], game::PickupKind::Teleporter)]),
        )
        .unwrap(),
        SeatId::ALL.map(|seat| ActorInit {
            id: game::EntityId(seat.index() as u32 + 10),
            seat,
            position: game::Position::center(spawns[seat.index()]),
            heading: game::Direction::East,
            speed: game::SpeedLevel::Base,
        }),
    )
    .unwrap()
}

#[test]
fn v10_captures_position_epoch_and_archived_v9_rejects_actual_history() {
    let mut state = teleporter_state(EnabledItems::TELEPORTER);
    let report = state.step(&[]).unwrap();
    assert_eq!(state.last_teleport_tick(), Some(game::Tick(1)));

    let capture = capture_expanded(&state, std::slice::from_ref(&report)).unwrap();
    assert_eq!(
        capture.world.0.last_teleport_tick,
        Some(crate::protocol::Tick::new(1).unwrap())
    );
    assert_eq!(
        crate::protocol::v9::capture_expanded(&state, &[report]),
        Err(ProjectionError::InvalidItems)
    );
}

#[test]
fn position_epoch_bounds_and_enabled_item_are_validated_before_projection() {
    let mut state = teleporter_state(EnabledItems::TELEPORTER);
    state.step(&[]).unwrap();
    let capture = capture_expanded(&state, &[]).unwrap();

    let mut future = capture.world.0.clone();
    future.last_teleport_tick = crate::protocol::Tick::new(2).ok();
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_world_for_items(
            future,
            super::expanded_items(),
        ),
        Err(ProjectionError::InvalidClock)
    );

    let mut at_round_start = capture.world.0.clone();
    at_round_start.last_teleport_tick = crate::protocol::Tick::new(0).ok();
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_world_for_items(
            at_round_start,
            super::expanded_items(),
        ),
        Err(ProjectionError::InvalidClock)
    );

    let mut disabled = capture.world.0;
    disabled.enabled_items = ItemSet::empty();
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_world_for_items(
            disabled,
            super::expanded_items(),
        ),
        Err(ProjectionError::InvalidItems)
    );
}

fn running_room(enabled: ItemSet, tick: Tick) -> RoomView {
    RoomView {
        settings: GameSettings::new(0, 256, 3, 180, enabled).unwrap(),
        settings_revision: Counter::new(1).unwrap(),
        settings_editor: None,
        available_items: super::expanded_items(),
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        round_id: Counter::new(1).unwrap(),
        revision: Counter::new(1).unwrap(),
        tick,
        rules_version: RulesVersion::ExpandedV5,
        map_id: MapId::GeneratedV2,
        round_number: RoundCount::new(1).unwrap(),
        scores: Scores {
            one: Wins::new(0).unwrap(),
            two: Wins::new(0).unwrap(),
        },
        last_round: None,
        seats: [Seat::One, Seat::Two].map(|seat| SeatStatus {
            seat,
            occupied: true,
            ready: false,
            rematch_ready: false,
            connection: ConnectionStatus::Connected,
        }),
        phase: RoomPhase::Running,
    }
}

fn archived_v9_items() -> ItemSet {
    ItemSet::new([
        ItemId::Heart,
        ItemId::RemoteBomb,
        ItemId::PiercingBomb,
        ItemId::PowerBomb,
        ItemId::Landmine,
        ItemId::Shield,
        ItemId::Stopwatch,
        ItemId::InvisibilityCloak,
        ItemId::InvisibleBombs,
        ItemId::GoldenFlame,
        ItemId::ShortFuse,
        ItemId::Diarrhea,
        ItemId::SlowFoot,
        ItemId::ShortFlame,
        ItemId::LongFuse,
        ItemId::ReverseControls,
    ])
    .unwrap()
}

fn clear_teleporter_settings(capture: &mut ExpandedCapture) -> RoomView {
    capture.rules =
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::EMPTY));
    capture.world.0.available_items = archived_v9_items();
    capture.world.0.enabled_items = ItemSet::empty();
    let mut room = running_room(ItemSet::empty(), capture.world.0.physical_tick);
    room.available_items = archived_v9_items();
    room
}

#[test]
fn archived_v9_rejects_marker_only_history_after_masks_and_events_are_cleared() {
    let mut state = teleporter_state(EnabledItems::TELEPORTER);
    state.step(&[]).unwrap();
    let mut capture = capture_expanded(&state, &[]).unwrap();
    let room = clear_teleporter_settings(&mut capture);
    let generation = game::generate_expanded_arena([0; 32], EnabledItems::EMPTY).record();
    assert_eq!(
        crate::protocol::v9::bind_expanded(&room, room.round_id, &capture, Some(generation)),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

#[test]
fn archived_v9_rejects_event_only_history_after_masks_are_cleared() {
    let mut state = teleporter_state(EnabledItems::TELEPORTER);
    let report = state
        .step(&[game::SeatInput {
            seat: SeatId::Two,
            action: game::InputAction::PlaceBomb,
        }])
        .unwrap();
    assert_eq!(state.last_teleport_tick(), None);
    assert!(report.events.iter().any(|event| matches!(
        event.kind,
        game::StepEventKind::PickupCollected {
            kind: game::PickupKind::Teleporter,
            ..
        }
    )));
    let mut capture = capture_expanded(&state, &[report]).unwrap();
    let room = clear_teleporter_settings(&mut capture);
    let generation = game::generate_expanded_arena([0; 32], EnabledItems::EMPTY).record();
    assert_eq!(
        crate::protocol::v9::bind_expanded(&room, room.round_id, &capture, Some(generation)),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

#[test]
fn recipient_projection_pairs_visible_epochs_and_hidden_actor_keeps_exact_privacy_shape() {
    let enabled = EnabledItems::from_bits(
        EnabledItems::TELEPORTER.bits() | EnabledItems::INVISIBILITY_CLOAK.bits(),
    )
    .unwrap();
    let mut state = teleporter_state(enabled);
    state.step(&[]).unwrap();
    state.apply_test_effect(
        SeatId::Two,
        game::EffectKind::InvisibilityCloak,
        state.tick(),
    );
    let capture = capture_expanded(&state, &[]).unwrap();
    let room = running_room(capture.world.0.enabled_items, capture.world.0.physical_tick);
    let publication = BoundPublication {
        room,
        world: capture.world,
        reports: capture.reports,
    };

    let one = serde_json::to_value(super::projection_recipient::recipient_state(
        &publication,
        Seat::One,
    ))
    .unwrap();
    let two = serde_json::to_value(super::projection_recipient::recipient_state(
        &publication,
        Seat::Two,
    ))
    .unwrap();
    assert_eq!(one["actors"][0]["lastTeleportTick"], 1);
    assert_eq!(
        one["actors"][1],
        serde_json::json!({"type":"hidden","id":11,"seat":"two","alive":true})
    );
    assert_eq!(two["actors"][0]["lastTeleportTick"], 1);
    assert_eq!(two["actors"][1]["lastTeleportTick"], 1);
}

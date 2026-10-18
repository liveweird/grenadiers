use std::collections::BTreeMap;

use crate::game::{self, ActorInit, EnabledItems, RulesConfig, SeatId, State, World};
use crate::protocol::{Seat, Tick};
use crate::room::RoomSettings;

use super::super::*;

fn collected_state() -> (State, game::StepReport) {
    let arena = game::fixed_arena();
    let spawns = arena.spawns();
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([
            (spawns[0], game::PickupKind::ShortFlame),
            (spawns[1], game::PickupKind::Shield),
        ]),
    )
    .unwrap();
    let mut state = State::new(
        RulesConfig::expanded_v5(
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::from_bits(66048).unwrap()),
        ),
        world,
        SeatId::ALL.map(|seat| ActorInit {
            id: game::EntityId(seat.index() as u32 + 1),
            seat,
            position: game::Position::center(spawns[seat.index()]),
            heading: game::Direction::East,
            speed: game::SpeedLevel::Base,
        }),
    )
    .unwrap();
    let report = state.step(&[]).unwrap();
    (state, report)
}

fn room() -> RoomView {
    let mut room = super::classic_room(1);
    room.rules_version = RulesVersion::ExpandedV5;
    room.map_id = MapId::GeneratedV2;
    room.available_items = super::super::projection_binding::expanded_items();
    room.settings =
        super::wire_settings(ItemSet::new([ItemId::Shield, ItemId::ShortFlame]).unwrap());
    room
}

#[test]
fn short_flame_and_shield_publish_distinct_physical_timers_and_quiet_admission() {
    let (state, report) = collected_state();
    let room = room();
    let record =
        game::generate_expanded_arena([0; 32], EnabledItems::from_bits(66048).unwrap()).record();
    let capture = capture_expanded(&state, &[report]).unwrap();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(record)).unwrap();
    for frame in prepare_snapshot_pair(&publication).unwrap() {
        let frame: serde_json::Value = serde_json::from_str(&frame).unwrap();
        assert_eq!(
            frame["room"]["availableItems"],
            serde_json::json!([
                "heart",
                "shield",
                "goldenFlame",
                "shortFuse",
                "diarrhea",
                "slowFoot",
                "shortFlame",
                "longFuse",
                "reverseControls",
            ])
        );
        assert_eq!(
            frame["state"]["actors"][0]["effects"],
            serde_json::json!([
                {"kind": "shortFlame", "remainingTicks": 600},
            ])
        );
        assert_eq!(frame["state"]["actors"][0]["bombRange"], 2);
        assert_eq!(
            frame["state"]["actors"][1]["effects"],
            serde_json::json!([
                {"kind": "shield", "remainingTicks": 300},
            ])
        );
        assert_eq!(frame["events"][0]["kind"]["kind"], "shortFlame");
    }
    let encoded = prepare_expanded_admission(
        &room,
        Some(room.round_id),
        Some(&capture),
        Some(record),
        Seat::One,
        None,
        room.revision,
        ResumeToken::new("c".repeat(64)).unwrap(),
    )
    .unwrap();
    let frame: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert!(frame.get("events").is_none());
    assert_eq!(
        frame["state"]["actors"][0]["effects"][0]["remainingTicks"],
        600
    );
    assert_eq!(
        frame["state"]["actors"][1]["effects"][0]["remainingTicks"],
        300
    );
}

#[test]
fn short_flame_state_cannot_cross_classic_or_disabled_item_bindings() {
    let (state, report) = collected_state();
    assert!(crate::protocol::v7::capture_classic(&state, &[]).is_err());
    let capture = capture_expanded(&state, &[report]).unwrap();
    let mut room = room();
    let record =
        game::generate_expanded_arena([0; 32], EnabledItems::from_bits(66048).unwrap()).record();
    room.settings = super::wire_settings(ItemSet::new([ItemId::Shield]).unwrap());
    assert_eq!(
        bind_expanded(&room, room.round_id, &capture, Some(record)),
        Err(ProjectionError::InvalidExpandedBinding),
    );
}

#[test]
fn short_flame_bombs_publish_captured_range_without_changing_base_upgrade() {
    let (mut state, _) = collected_state();
    let report = state
        .step(&[game::SeatInput {
            seat: game::SeatId::One,
            action: game::InputAction::PlaceBomb,
        }])
        .unwrap();
    let capture = capture_expanded(&state, &[report]).unwrap();
    let mut room = room();
    room.tick = Tick::new(state.tick().0).unwrap();
    let record =
        game::generate_expanded_arena([0; 32], EnabledItems::from_bits(66048).unwrap()).record();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(record)).unwrap();
    for encoded in prepare_snapshot_pair(&publication).unwrap() {
        let frame: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(frame["state"]["actors"][0]["bombRange"], 2);
        assert_eq!(frame["state"]["bombs"][0]["range"], 1);
        assert_eq!(frame["state"]["bombs"][0]["trigger"]["remainingTicks"], 180);
        assert_eq!(frame["events"][0]["kind"]["bomb"]["range"], 1);
    }
}

use std::collections::BTreeMap;

use crate::game::{self, ActorInit, EnabledItems, PickupKind, RulesConfig, SeatId, State, World};
use crate::protocol::Seat;
use crate::room::RoomSettings;

use super::super::*;

fn shield_state() -> State {
    let arena = game::fixed_arena();
    let spawns = arena.spawns();
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(spawns[0], PickupKind::Shield)]),
    )
    .unwrap();
    State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::SHIELD)),
        world,
        SeatId::ALL.map(|seat| ActorInit {
            id: game::EntityId(seat.index() as u32 + 1),
            seat,
            position: game::Position::center(spawns[seat.index()]),
            heading: game::Direction::East,
            speed: game::SpeedLevel::Base,
        }),
    )
    .unwrap()
}

fn shield_room(tick: u64) -> RoomView {
    let mut room = super::classic_room(tick);
    room.rules_version = RulesVersion::ExpandedV5;
    room.map_id = MapId::GeneratedV2;
    room.available_items = super::super::projection_binding::expanded_items();
    room.settings = super::wire_settings(ItemSet::new([ItemId::Shield]).unwrap());
    room
}

#[test]
fn shield_settings_accept_all_four_masks_without_enabling_future_items() {
    for bits in [0, 16, 512, 528] {
        let items = ItemSet::from_mask(ItemMask::new(bits).unwrap());
        let domain = expanded_settings(super::wire_settings(items)).unwrap();
        assert_eq!(domain.enabled_items().bits(), bits);
    }
    for item in [ItemId::Stopwatch, ItemId::Kick, ItemId::InvisibleBombs] {
        assert_eq!(
            expanded_settings(super::wire_settings(ItemSet::new([item]).unwrap())),
            Err(ProjectionError::InvalidItems)
        );
    }
    assert_eq!(
        GameSettings::for_rules(RulesVersion::ExpandedV5).enabled_items(),
        ItemSet::new([ItemId::Heart]).unwrap()
    );
}

#[test]
fn collected_shield_is_published_to_both_seats_and_retained_in_quiet_admission() {
    let mut state = shield_state();
    let report = state.step(&[]).unwrap();
    let room = shield_room(1);
    let record = game::generate_expanded_arena([0; 32], EnabledItems::SHIELD).record();
    let capture = capture_expanded(&state, &[report]).unwrap();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(record)).unwrap();
    for frame in prepare_snapshot_pair(&publication).unwrap() {
        let frame: serde_json::Value = serde_json::from_str(&frame).unwrap();
        assert_eq!(frame["state"]["actors"][0]["effects"][0]["kind"], "shield");
        assert_eq!(
            frame["state"]["actors"][0]["effects"][0]["remainingTicks"],
            300
        );
        assert_eq!(frame["events"][0]["kind"]["kind"], "shield");
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
        300
    );
}

#[test]
fn shield_prevention_is_accepted_only_when_shield_is_enabled() {
    let mut state = shield_state();
    state.step(&[]).unwrap();
    let cell = state.actors()[0].position().cell().unwrap();
    state.insert_test_flame(cell, game::Tick(5));
    let report = state.step(&[]).unwrap();
    let capture = capture_expanded(&state, std::slice::from_ref(&report)).unwrap();
    let room = shield_room(2);
    let record = game::generate_expanded_arena([0; 32], EnabledItems::SHIELD).record();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(record)).unwrap();
    let frame: serde_json::Value =
        serde_json::from_str(&prepare_snapshot_pair(&publication).unwrap()[0]).unwrap();
    assert_eq!(frame["events"].as_array().unwrap().len(), 1);
    assert_eq!(frame["events"][0]["kind"]["source"], "shield");

    let legacy = super::classic_state_at(2);
    let mut forged = report;
    forged.tick = legacy.tick();
    assert_eq!(
        capture_expanded(&legacy, &[forged]),
        Err(ProjectionError::InvalidItems)
    );
}

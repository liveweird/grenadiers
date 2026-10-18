use std::collections::BTreeMap;

use crate::game::{
    self, ActorInit, EnabledItems, InputAction, PickupKind, RulesConfig, SeatId, SeatInput, State,
    World,
};
use crate::room::RoomSettings;

use super::super::*;

fn power_state() -> State {
    let arena = game::fixed_arena();
    let spawns = arena.spawns();
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([
            (spawns[0], PickupKind::PowerBomb),
            (game::CellId::new(2, 1), PickupKind::PiercingBomb),
        ]),
    )
    .unwrap();
    let enabled = EnabledItems::from_bits(
        EnabledItems::PIERCING_BOMB.bits() | EnabledItems::POWER_BOMB.bits(),
    )
    .unwrap();
    State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled)),
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

fn power_room() -> (RoomView, game::GenerationRecord) {
    let mut room = super::classic_room(2);
    room.rules_version = RulesVersion::ExpandedV5;
    room.map_id = MapId::GeneratedV2;
    room.available_items = super::super::projection_binding::expanded_items();
    room.settings =
        super::wire_settings(ItemSet::new([ItemId::PiercingBomb, ItemId::PowerBomb]).unwrap());
    let enabled = EnabledItems::from_bits(
        EnabledItems::PIERCING_BOMB.bits() | EnabledItems::POWER_BOMB.bits(),
    )
    .unwrap();
    let record = game::generate_expanded_arena([0; 32], enabled).record();
    (room, record)
}

#[test]
fn enabled_power_inventory_bomb_and_events_publish_to_both_recipients() {
    let mut state = power_state();
    let collected = state.step(&[]).unwrap();
    let placed = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    let (mut room, record) = power_room();
    let capture = capture_expanded(&state, &[collected, placed]).unwrap();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(record)).unwrap();
    for encoded in prepare_snapshot_pair(&publication).unwrap() {
        let frame: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(frame["state"]["bombs"][0]["blast"], "power");
        assert_eq!(
            frame["state"]["actors"][0]["inventory"]["weapon"],
            serde_json::Value::Null
        );
        assert!(frame["events"].as_array().unwrap().iter().any(|event| {
            event["kind"]["type"] == "bombPlaced" && event["kind"]["bomb"]["blast"] == "power"
        }));
    }

    room.settings = super::wire_settings(ItemSet::empty());
    assert_eq!(
        bind_expanded(&room, room.round_id, &capture, Some(record)),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

#[test]
fn power_event_only_and_existing_bomb_do_not_require_a_current_charge() {
    let (mut room, record) = power_room();
    let mut state = power_state();
    state.step(&[]).unwrap();
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(game::Direction::East),
        }])
        .unwrap();
    for _ in 0..20 {
        state.step(&[]).unwrap();
    }
    room.tick = crate::protocol::Tick::new(state.tick().0).unwrap();
    let live_capture = capture_expanded(&state, &[]).unwrap();
    let live = bind_expanded(&room, room.round_id, &live_capture, Some(record)).unwrap();
    for encoded in prepare_snapshot_pair(&live).unwrap() {
        let frame: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(frame["state"]["bombs"][0]["blast"], "power");
        assert_eq!(
            frame["state"]["actors"][0]["inventory"]["weapon"],
            "piercingBomb"
        );
    }

    let detonation = loop {
        let report = state.step(&[]).unwrap();
        if report.events.iter().any(|event| {
            matches!(event.kind, game::StepEventKind::BombDetonated(ref bomb)
                if bomb.blast == game::BlastKind::Power)
        }) {
            break report;
        }
    };
    assert!(state.world().bombs().is_empty());
    room.tick = crate::protocol::Tick::new(state.tick().0).unwrap();
    let capture = capture_expanded(&state, &[detonation]).unwrap();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(record)).unwrap();
    for encoded in prepare_snapshot_pair(&publication).unwrap() {
        let frame: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert!(frame["state"]["bombs"].as_array().unwrap().is_empty());
        assert!(frame["events"].as_array().unwrap().iter().any(|event| {
            event["kind"]["type"] == "bombDetonated" && event["kind"]["bomb"]["blast"] == "power"
        }));
    }
}

#[test]
fn live_power_bomb_remains_canonical_after_its_owner_dies() {
    let (mut room, record) = power_room();
    let mut state = power_state();
    state.step(&[]).unwrap();
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(game::Direction::East),
        }])
        .unwrap();
    for _ in 0..30 {
        state.step(&[]).unwrap();
    }
    let owner = state.actors()[0].id();
    let owner_cell = state.actors()[0].position().cell().unwrap();
    state.insert_test_flame(owner_cell, game::Tick(100));
    state.step(&[]).unwrap();
    assert!(!state.actors()[0].alive());
    assert!(state.actors()[0].inventory().weapon().is_none());
    assert_eq!(state.world().bombs().len(), 1);

    let ended_at = crate::protocol::Tick::new(state.tick().0).unwrap();
    room.tick = ended_at;
    room.scores.two = Wins::new(1).unwrap();
    room.last_round = Some(CompletedRound {
        round_id: room.round_id,
        round_number: room.round_id,
        ended_at,
        result: crate::protocol::WireRoundResult::Winner {
            seat: crate::protocol::Seat::Two,
        },
    });
    room.phase = RoomPhase::Result {
        hold_until: crate::protocol::Tick::new(state.tick().0 + 180).unwrap(),
    };
    let capture = capture_expanded(&state, &[]).unwrap();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(record)).unwrap();
    for encoded in prepare_snapshot_pair(&publication).unwrap() {
        let frame: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(frame["state"]["bombs"][0]["blast"], "power");
        let actor = frame["state"]["actors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|actor| actor["id"] == owner.0)
            .unwrap();
        assert_eq!(actor["alive"], false);
        assert_eq!(actor["inventory"]["weapon"], serde_json::Value::Null);
    }
}

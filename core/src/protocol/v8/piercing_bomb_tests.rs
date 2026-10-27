use std::collections::BTreeMap;

use crate::game::{
    self, ActorInit, EnabledItems, InputAction, PickupKind, RulesConfig, SeatId, SeatInput, State,
    World,
};
use crate::room::RoomSettings;

use super::super::*;

fn piercing_state() -> State {
    let arena = game::fixed_arena();
    let spawns = arena.spawns().to_vec();
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(spawns[0], PickupKind::PiercingBomb)]),
    )
    .unwrap();
    State::new(
        RulesConfig::expanded_v5(
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::PIERCING_BOMB),
        ),
        world,
        SeatId::DUEL.map(|seat| ActorInit {
            id: game::EntityId(seat.index() as u32 + 1),
            seat,
            position: game::Position::center(spawns[seat.index()]),
            heading: game::Direction::East,
            speed: game::SpeedLevel::Base,
        }),
    )
    .unwrap()
}

fn piercing_room() -> (RoomView, game::GenerationRecord) {
    let mut room = super::classic_room(2);
    room.rules_version = RulesVersion::ExpandedV5;
    room.map_id = MapId::GeneratedV2;
    room.available_items = super::super::projection_binding::expanded_items();
    room.settings = super::wire_settings(ItemSet::new([ItemId::PiercingBomb]).unwrap());
    let record = game::generate_expanded_arena([0; 32], EnabledItems::PIERCING_BOMB).record();
    (room, record)
}

#[test]
fn enabled_piercing_inventory_bomb_and_events_publish_to_both_recipients() {
    let mut state = piercing_state();
    let collected = state.step(&[]).unwrap();
    let placed = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    let (mut room, record) = piercing_room();
    let capture = capture_expanded(&state, &[collected, placed]).unwrap();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(record)).unwrap();
    for encoded in prepare_snapshot_pair(&publication).unwrap() {
        let frame: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(frame["state"]["bombs"][0]["blast"], "piercing");
        assert_eq!(
            frame["state"]["actors"][0]["inventory"]["weapon"],
            serde_json::Value::Null
        );
        assert!(frame["events"].as_array().unwrap().iter().any(|event| {
            event["kind"]["type"] == "bombPlaced" && event["kind"]["bomb"]["blast"] == "piercing"
        }));
    }

    room.settings = super::wire_settings(ItemSet::empty());
    assert_eq!(
        bind_expanded(&room, room.round_id, &capture, Some(record)),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

#[test]
fn event_only_detonation_and_dead_owner_live_bomb_publish_to_both_recipients() {
    let (mut room, record) = piercing_room();
    let mut detonated = piercing_state();
    detonated.step(&[]).unwrap();
    detonated
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    detonated
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(game::Direction::East),
        }])
        .unwrap();
    let detonation = loop {
        let report = detonated.step(&[]).unwrap();
        if report.events.iter().any(|event| {
            matches!(event.kind, game::StepEventKind::BombDetonated(ref bomb)
                if bomb.blast == game::BlastKind::Piercing)
        }) {
            break report;
        }
    };
    assert!(detonated.world().bombs().is_empty());
    room.tick = crate::protocol::Tick::new(detonated.tick().0).unwrap();
    let capture = capture_expanded(&detonated, &[detonation]).unwrap();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(record)).unwrap();
    for encoded in prepare_snapshot_pair(&publication).unwrap() {
        let frame: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert!(frame["state"]["bombs"].as_array().unwrap().is_empty());
        assert!(frame["events"].as_array().unwrap().iter().any(|event| {
            event["kind"]["type"] == "bombDetonated" && event["kind"]["bomb"]["blast"] == "piercing"
        }));
    }

    let mut live = piercing_state();
    live.step(&[]).unwrap();
    live.step(&[SeatInput {
        seat: SeatId::One,
        action: InputAction::PlaceBomb,
    }])
    .unwrap();
    live.step(&[SeatInput {
        seat: SeatId::One,
        action: InputAction::Press(game::Direction::East),
    }])
    .unwrap();
    for _ in 0..30 {
        live.step(&[]).unwrap();
    }
    let owner = live.actors()[0].id();
    let owner_cell = live.actors()[0].position().cell().unwrap();
    live.insert_test_flame(owner_cell, game::Tick(100));
    live.step(&[]).unwrap();
    assert!(!live.actors()[0].alive());
    assert_eq!(live.world().bombs().len(), 1);
    let ended_at = crate::protocol::Tick::new(live.tick().0).unwrap();
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
        hold_until: crate::protocol::Tick::new(live.tick().0 + 180).unwrap(),
    };
    let capture = capture_expanded(&live, &[]).unwrap();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(record)).unwrap();
    for encoded in prepare_snapshot_pair(&publication).unwrap() {
        let frame: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(frame["state"]["bombs"][0]["blast"], "piercing");
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

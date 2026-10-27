use std::collections::BTreeMap;

use crate::game::{
    self, ActorInit, EnabledItems, InputAction, PickupKind, RulesConfig, SeatId, SeatInput, State,
    World,
};
use crate::room::RoomSettings;

use super::super::*;

fn state() -> State {
    let arena = game::fixed_arena();
    let mut spawns = arena.spawns().to_vec();
    spawns[0] = game::CellId::new(3, 1);
    State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::LANDMINE)),
        World::with_pickups(
            arena,
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(spawns[0], PickupKind::Landmine)]),
        )
        .unwrap(),
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

fn room(tick: u64) -> (RoomView, game::GenerationRecord) {
    let mut room = super::classic_room(tick);
    room.rules_version = RulesVersion::ExpandedV5;
    room.map_id = MapId::GeneratedV2;
    room.available_items = super::super::projection_binding::expanded_items();
    room.settings = super::wire_settings(ItemSet::new([ItemId::Landmine]).unwrap());
    let record = game::generate_expanded_arena([0; 32], EnabledItems::LANDMINE).record();
    (room, record)
}

fn action(action: InputAction) -> [SeatInput; 1] {
    [SeatInput {
        seat: SeatId::One,
        action,
    }]
}

#[test]
fn enabled_landmine_inventory_pickup_live_bomb_and_placement_publish_to_both_recipients() {
    let mut state = state();
    let collected = state.step(&[]).unwrap();
    let placed = state.step(&action(InputAction::PlaceBomb)).unwrap();
    let mine = *state.world().bombs().values().next().unwrap();
    let (room, record) = room(state.tick().0);
    let capture = capture_expanded(&state, &[collected, placed]).unwrap();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(record)).unwrap();

    for encoded in prepare_snapshot_pair(&publication).unwrap() {
        let frame: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(frame["state"]["bombs"][0]["id"], mine.id().0);
        assert_eq!(frame["state"]["bombs"][0]["trigger"]["type"], "contact");
        assert_eq!(frame["state"]["bombs"][0]["blast"], "standard");
        assert_eq!(
            frame["state"]["actors"][0]["inventory"]["weapon"],
            serde_json::Value::Null
        );
        assert!(frame["events"].as_array().unwrap().iter().any(|event| {
            event["kind"]["type"] == "bombPlaced" && event["kind"]["bomb"]["id"] == mine.id().0
        }));
    }

    let mut disabled = room.clone();
    disabled.settings = super::wire_settings(ItemSet::empty());
    assert_eq!(
        bind_expanded(&disabled, disabled.round_id, &capture, Some(record)),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

#[test]
fn contact_detonation_is_honest_event_only_history_and_reserved_combinations_reject() {
    let mut state = state();
    state.step(&[]).unwrap();
    state.step(&action(InputAction::PlaceBomb)).unwrap();
    let mine = *state.world().bombs().values().next().unwrap();
    state
        .step(&action(InputAction::Press(game::Direction::West)))
        .unwrap();
    for _ in 1..18 {
        state.step(&[]).unwrap();
    }
    state.step(&action(InputAction::Clear)).unwrap();
    state
        .step(&action(InputAction::Press(game::Direction::East)))
        .unwrap();
    let detonated = loop {
        let report = state.step(&[]).unwrap();
        if report.events.iter().any(|event| matches!(event.kind, game::StepEventKind::BombDetonated(view) if view.id == mine.id())) {
            break report;
        }
    };
    let (mut room, record) = room(state.tick().0);
    let ended_at = crate::protocol::Tick::new(state.tick().0).unwrap();
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
    let capture = capture_expanded(&state, &[detonated]).unwrap();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(record)).unwrap();
    for encoded in prepare_snapshot_pair(&publication).unwrap() {
        let frame: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert!(frame["state"]["bombs"].as_array().unwrap().is_empty());
        assert!(frame["events"].as_array().unwrap().iter().any(|event| {
            event["kind"]["type"] == "bombDetonated"
                && event["kind"]["bomb"]["id"] == mine.id().0
                && event["kind"]["bomb"]["trigger"]["type"] == "contact"
        }));
    }

    for blast in [BlastKind::Piercing, BlastKind::Power] {
        let mut invalid = capture.clone();
        let event = invalid.reports.0[0]
            .events
            .iter_mut()
            .find(|event| matches!(event.kind, EventKind::BombDetonated { .. }))
            .unwrap();
        let EventKind::BombDetonated { bomb } = &mut event.kind else {
            unreachable!()
        };
        bomb.blast = blast;
        assert_eq!(
            bind_expanded(&room, room.round_id, &invalid, Some(record)),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }
}

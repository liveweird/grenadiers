use std::collections::BTreeMap;

use crate::game::{
    self, ActorInit, EnabledItems, InputAction, PickupKind, RulesConfig, SeatId, SeatInput, State,
    World,
};
use crate::room::RoomSettings;

use super::super::*;

fn remote_state() -> State {
    let arena = game::fixed_arena();
    let spawns = arena.spawns().to_vec();
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(spawns[0], PickupKind::RemoteBomb)]),
    )
    .unwrap();
    State::new(
        RulesConfig::expanded_v5(
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::REMOTE_BOMB),
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

fn remote_room() -> (RoomView, game::GenerationRecord) {
    let mut room = super::classic_room(2);
    room.rules_version = RulesVersion::ExpandedV5;
    room.map_id = MapId::GeneratedV2;
    room.available_items = super::super::projection_binding::expanded_items();
    room.settings = super::wire_settings(ItemSet::new([ItemId::RemoteBomb]).unwrap());
    let record = game::generate_expanded_arena([0; 32], EnabledItems::REMOTE_BOMB).record();
    (room, record)
}

#[test]
fn enabled_remote_inventory_live_bomb_and_detonation_event_publish_to_both_recipients() {
    let mut state = remote_state();
    let collected = state.step(&[]).unwrap();
    let placed = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    let (mut room, record) = remote_room();
    let capture = capture_expanded(&state, &[collected, placed]).unwrap();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(record)).unwrap();
    for encoded in prepare_snapshot_pair(&publication).unwrap() {
        let frame: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(frame["state"]["bombs"][0]["trigger"]["type"], "remote");
        assert_eq!(frame["state"]["bombs"][0]["blast"], "standard");
        assert_eq!(
            frame["state"]["actors"][0]["inventory"]["weapon"],
            serde_json::Value::Null
        );
    }

    let detonated = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
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
    let capture = capture_expanded(&state, &[detonated]).unwrap();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(record)).unwrap();
    for encoded in prepare_snapshot_pair(&publication).unwrap() {
        let frame: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert!(frame["state"]["bombs"].as_array().unwrap().is_empty());
        assert!(frame["events"].as_array().unwrap().iter().any(|event| {
            event["kind"]["type"] == "bombDetonated"
                && event["kind"]["bomb"]["trigger"]["type"] == "remote"
        }));
    }

    room.settings = super::wire_settings(ItemSet::empty());
    assert_eq!(
        bind_expanded(&room, room.round_id, &capture, Some(record)),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

#[test]
fn historical_remote_detonation_and_new_live_remote_share_one_valid_publication_window() {
    let arena = game::fixed_arena();
    let spawns = arena.spawns().to_vec();
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(spawns[0], PickupKind::RemoteBomb)]),
    )
    .unwrap();
    let mut state = State::new(
        RulesConfig::expanded_v5(
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::REMOTE_BOMB),
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
    .unwrap();
    let old_id = game::EntityId(30);
    game::test_support::add_test_bomb_with_trigger(
        &mut state,
        old_id,
        game::CellId::new(5, 5),
        game::EntityId(1),
        2,
        game::Tick(0),
        game::BombTrigger::Remote,
    )
    .unwrap();
    state.step(&[]).unwrap();
    let detonated = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    let replacement = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    let live = *state.world().bombs().values().next().unwrap();
    assert_ne!(live.id(), old_id);
    assert_eq!(live.owner(), game::EntityId(1));

    let (mut room, record) = remote_room();
    room.tick = crate::protocol::Tick::new(state.tick().0).unwrap();
    let capture = capture_expanded(&state, &[detonated, replacement]).unwrap();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(record)).unwrap();
    for encoded in prepare_snapshot_pair(&publication).unwrap() {
        let frame: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(frame["state"]["bombs"][0]["id"], live.id().0);
        assert!(frame["events"].as_array().unwrap().iter().any(|event| {
            event["kind"]["type"] == "bombDetonated" && event["kind"]["bomb"]["id"] == old_id.0
        }));
        assert!(frame["events"].as_array().unwrap().iter().any(|event| {
            event["kind"]["type"] == "bombPlaced" && event["kind"]["bomb"]["id"] == live.id().0
        }));
    }

    let mut duplicate_live = capture.clone();
    let mut duplicate = duplicate_live.world.0.bombs[0];
    duplicate.id = crate::protocol::EntityId::new(live.id().0 + 1);
    duplicate.cell = crate::protocol::Cell::new(2, 1).unwrap();
    duplicate_live.world.0.bombs.push(duplicate);
    duplicate_live.world.0.actors[0].live_bomb_count = 2;
    assert_eq!(
        bind_expanded(&room, room.round_id, &duplicate_live, Some(record)),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

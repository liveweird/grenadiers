use std::collections::BTreeMap;

use crate::game::{self, ActorInit, EnabledItems, PickupKind, RulesConfig, SeatId, State, World};
use crate::room::RoomSettings;

use super::super::*;

fn state() -> State {
    let arena = game::fixed_arena();
    let spawns = arena.spawns();
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(spawns[0], PickupKind::SlowFoot)]),
    )
    .unwrap();
    State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::SLOW_FOOT)),
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

#[test]
fn enabled_slow_foot_collection_and_independent_timer_are_canonical() {
    let mut state = state();
    let collected = state.step(&[]).unwrap();
    let mut room = super::classic_room(1);
    room.rules_version = RulesVersion::ExpandedV5;
    room.map_id = MapId::GeneratedV2;
    room.available_items = super::super::projection_binding::expanded_items();
    room.settings = super::wire_settings(ItemSet::new([ItemId::SlowFoot]).unwrap());
    let record = game::generate_expanded_arena([0; 32], EnabledItems::SLOW_FOOT).record();
    let capture = capture_expanded(&state, &[collected]).unwrap();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(record)).unwrap();
    for encoded in prepare_snapshot_pair(&publication).unwrap() {
        let frame: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(
            frame["state"]["actors"][0]["effects"][0]["kind"],
            "slowFoot"
        );
        assert_eq!(
            frame["state"]["actors"][0]["effects"][0]["remainingTicks"],
            600
        );
    }

    room.settings = super::wire_settings(ItemSet::default());
    assert_eq!(
        bind_expanded(&room, room.round_id, &capture, Some(record)),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

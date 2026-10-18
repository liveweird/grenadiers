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
        BTreeMap::from([(spawns[0], PickupKind::ReverseControls)]),
    )
    .unwrap();
    State::new(
        RulesConfig::expanded_v5(
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::REVERSE_CONTROLS),
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
    .unwrap()
}

#[test]
fn enabled_reverse_controls_collection_is_canonical_for_both_recipients() {
    let mut state = state();
    let report = state.step(&[]).unwrap();
    let mut room = super::classic_room(1);
    room.rules_version = RulesVersion::ExpandedV5;
    room.map_id = MapId::GeneratedV2;
    room.available_items = super::super::projection_binding::expanded_items();
    room.settings = super::wire_settings(ItemSet::new([ItemId::ReverseControls]).unwrap());
    let record = game::generate_expanded_arena([0; 32], EnabledItems::REVERSE_CONTROLS).record();
    let capture = capture_expanded(&state, &[report]).unwrap();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(record)).unwrap();
    for encoded in prepare_snapshot_pair(&publication).unwrap() {
        let frame: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(
            frame["state"]["actors"][0]["effects"][0]["kind"],
            "reverseControls"
        );
        assert_eq!(
            frame["state"]["actors"][0]["effects"][0]["remainingTicks"],
            600
        );
        assert_eq!(frame["events"][0]["kind"]["kind"], "reverseControls");
    }

    room.settings = super::wire_settings(ItemSet::default());
    assert_eq!(
        bind_expanded(&room, room.round_id, &capture, Some(record)),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

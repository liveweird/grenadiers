use std::collections::BTreeMap;

use serde_json::Value;

use crate::game::{
    self, ActorInit, CellId, Direction as GameDirection, EnabledItems, EntityId as GameEntityId,
    OilState, Position as GamePosition, RulesConfig, SeatId, SpeedLevel, State, World,
};
use crate::protocol::{Cell, Counter, RoomCode, RoomId, Seat};
use crate::room::RoomSettings;

use super::*;

pub(super) fn enabled() -> EnabledItems {
    EnabledItems::from_bits(EnabledItems::OIL_BARREL.bits() | EnabledItems::WALL_PASS.bits())
        .unwrap()
}

pub(super) fn oil_state() -> State {
    let arena = game::fixed_arena();
    let spawns = arena.spawns();
    State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled())),
        World::with_pickups_and_oil(
            arena,
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([
                (CellId::new(5, 1), OilState::Concealed),
                (CellId::new(3, 1), OilState::Pending),
                (CellId::new(4, 1), OilState::Unlit),
                (CellId::new(3, 3), OilState::Burning),
            ]),
        )
        .unwrap(),
        SeatId::ALL.map(|seat| ActorInit {
            id: GameEntityId(seat.index() as u32 + 1),
            seat,
            position: GamePosition::center(spawns[seat.index()]),
            heading: GameDirection::East,
            speed: SpeedLevel::Base,
        }),
    )
    .unwrap()
}

fn room(enabled: ItemSet, tick: crate::protocol::Tick) -> RoomView {
    RoomView {
        settings: GameSettings::new(0, 256, 3, 180, enabled).unwrap(),
        settings_revision: Counter::new(1).unwrap(),
        settings_editor: None,
        available_items: expanded_items(),
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

fn generation() -> game::GenerationRecord {
    game::generate_expanded_arena([47; 32], enabled()).record()
}

#[test]
fn v11_catalog_adds_oil_at_its_reserved_bit_without_moving_teleporter() {
    let catalog = expanded_items();
    assert_eq!(catalog.iter().count(), 22);
    assert_eq!(catalog.mask().bits(), 4_194_303);
    assert_eq!(
        ItemSet::new([ItemId::OilBarrel]).unwrap().mask().bits(),
        1_048_576
    );
    assert_eq!(
        ItemSet::new([ItemId::Teleporter]).unwrap().mask().bits(),
        2_097_152
    );
}

#[test]
fn trusted_capture_keeps_all_private_phases_and_only_materializes_sorted_public_oil() {
    let state = oil_state();
    let capture = capture_expanded(&state, &[]).unwrap();
    use crate::protocol::v8::projection_facts::OilPhase;
    assert_eq!(
        capture
            .world
            .0
            .oil
            .iter()
            .map(|oil| (oil.cell, oil.phase))
            .collect::<Vec<_>>(),
        vec![
            (Cell::new(3, 1).unwrap(), OilPhase::Pending),
            (Cell::new(4, 1).unwrap(), OilPhase::Unlit),
            (Cell::new(5, 1).unwrap(), OilPhase::Concealed),
            (Cell::new(3, 3).unwrap(), OilPhase::Burning),
        ]
    );
    assert!(capture.world.0.unlit_oil_grace.iter().all(Vec::is_empty));

    let room = room(capture.world.0.enabled_items, capture.world.0.physical_tick);
    let publication = bind_expanded(&room, room.round_id, &capture, Some(generation())).unwrap();
    let frames = prepare_snapshot_pair(&publication).unwrap();
    assert_eq!(frames[0], frames[1]);
    let state = &serde_json::from_str::<Value>(&frames[0]).unwrap()["state"];
    assert_eq!(
        state["oilBarrels"],
        serde_json::json!([
            {"cell":{"column":4,"row":1},"state":"unlit"},
            {"cell":{"column":3,"row":3},"state":"burning"}
        ])
    );
    assert!(state["flames"].as_array().unwrap().is_empty());
    for private in ["concealed", "pending", "unlitOilGrace", "privatePickup"] {
        assert!(!frames[0].contains(private));
    }
}

#[test]
fn private_capture_debug_redacts_locations_phases_grace_and_history() {
    let mut capture = capture_expanded(&oil_state(), &[]).unwrap();
    capture.world.0.actors[0].inventory.equipment = Some(EquipmentId::Heart);
    capture.world.0.unlit_oil_grace[0] = vec![Cell::new(4, 1).unwrap()];
    let debug = format!("{capture:?}");
    assert!(debug.contains("<redacted>"));
    for private in [
        "Pending",
        "Concealed",
        "unlit_oil_grace",
        "captured_fuse_ticks",
    ] {
        assert!(!debug.contains(private));
    }
}

#[test]
fn archived_v10_rejects_oil_settings_private_phases_and_typed_grace() {
    let arena = game::fixed_arena();
    let spawns = arena.spawns();
    let neutral = State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT),
        World::new(arena),
        SeatId::ALL.map(|seat| ActorInit {
            id: GameEntityId(seat.index() as u32 + 1),
            seat,
            position: GamePosition::center(spawns[seat.index()]),
            heading: GameDirection::East,
            speed: SpeedLevel::Base,
        }),
    )
    .unwrap();
    let mut capture = capture_expanded(&neutral, &[]).unwrap();
    let v10_items = ItemSet::new(
        expanded_items()
            .iter()
            .filter(|item| *item != ItemId::OilBarrel),
    )
    .unwrap();
    capture.world.0.available_items = v10_items;
    let mut archived_room = room(ItemSet::empty(), capture.world.0.physical_tick);
    archived_room.available_items = v10_items;
    let archived_generation = game::generate_expanded_arena([10; 32], EnabledItems::EMPTY).record();
    assert!(
        crate::protocol::v10::bind_expanded(
            &archived_room,
            archived_room.round_id,
            &capture,
            Some(archived_generation)
        )
        .is_ok()
    );

    let mut settings = archived_room.clone();
    settings.settings =
        GameSettings::new(0, 256, 3, 180, ItemSet::new([ItemId::OilBarrel]).unwrap()).unwrap();
    assert_eq!(
        crate::protocol::v10::bind_expanded(
            &settings,
            settings.round_id,
            &capture,
            Some(archived_generation)
        ),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    let mut private = capture.clone();
    private
        .world
        .0
        .oil
        .push(crate::protocol::v8::projection_facts::OilFacts {
            cell: Cell::new(5, 1).unwrap(),
            phase: crate::protocol::v8::projection_facts::OilPhase::Concealed,
        });
    assert_eq!(
        crate::protocol::v10::bind_expanded(
            &archived_room,
            archived_room.round_id,
            &private,
            Some(archived_generation)
        ),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    let mut grace = capture;
    grace.world.0.unlit_oil_grace[0] = vec![Cell::new(3, 1).unwrap()];
    assert_eq!(
        crate::protocol::v10::bind_expanded(
            &archived_room,
            archived_room.round_id,
            &grace,
            Some(archived_generation)
        ),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

#[test]
fn every_expanded_archive_rejects_actual_oil_before_recipient_filtering() {
    let state = oil_state();
    for capture in [
        crate::protocol::v8::capture_expanded(&state, &[]),
        crate::protocol::v9::capture_expanded(&state, &[]),
        crate::protocol::v10::capture_expanded(&state, &[]),
    ] {
        assert_eq!(capture, Err(ProjectionError::InvalidItems));
    }
}

#[test]
fn invalid_second_recipient_produces_no_partial_pair_or_capture_mutation() {
    use crate::game::EventAudience;
    use crate::protocol::v8::projection_facts::{EventFacts, ReportFacts};

    let state = oil_state();
    let capture = capture_expanded(&state, &[]).unwrap();
    let room = room(capture.world.0.enabled_items, capture.world.0.physical_tick);
    let mut publication =
        bind_expanded(&room, room.round_id, &capture, Some(generation())).unwrap();
    let event = EventFacts {
        tick: publication.world.0.physical_tick,
        audience: EventAudience::Two,
        kind: EventKind::ActorEliminated {
            id: publication.world.0.actors[1].id,
            seat: Seat::Two,
        },
        timed_bomb: None,
        kick_bomb: None,
        punch_bomb: None,
    };
    publication.reports.0 = vec![ReportFacts {
        tick: event.tick,
        events: vec![event; 97],
        forced_movement: None,
    }];
    let first = GameServerMessage::Snapshot {
        room: publication.room.clone(),
        state: super::projection_recipient::recipient_state(&publication, Seat::One),
        events: SnapshotEvents::default(),
    };
    assert!(encode_server_message(&first).is_ok());

    let before = publication.clone();
    assert_eq!(
        prepare_snapshot_pair(&publication),
        Err(ProjectionError::InvalidReports)
    );
    assert_eq!(publication, before);
}

#[test]
fn oil_can_never_appear_as_a_pickup_collection_event() {
    use crate::game::EventAudience;
    use crate::protocol::v8::projection_facts::{EventFacts, ReportFacts, ValidatedReports};

    let state = oil_state();
    let mut capture = capture_expanded(&state, &[]).unwrap();
    let room = room(capture.world.0.enabled_items, capture.world.0.physical_tick);
    assert!(bind_expanded(&room, room.round_id, &capture, Some(generation())).is_ok());
    capture.reports = ValidatedReports(vec![ReportFacts {
        tick: capture.world.0.physical_tick,
        events: vec![EventFacts {
            tick: capture.world.0.physical_tick,
            audience: EventAudience::Both,
            kind: EventKind::PickupCollected {
                id: capture.world.0.actors[0].id,
                seat: Seat::One,
                cell: Cell::new(2, 1).unwrap(),
                kind: PickupKind::OilBarrel,
            },
            timed_bomb: None,
            kick_bomb: None,
            punch_bomb: None,
        }],
        forced_movement: None,
    }]);
    assert_eq!(
        bind_expanded(&room, room.round_id, &capture, Some(generation())),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

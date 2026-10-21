use crate::game::{EnabledItems, EventAudience, SpeedLevel, StepEvent, StepEventKind};
use crate::protocol::{Counter, RoomCode, RoomId, Seat, Tick};
use crate::room::{MatchLifecycle, RoomSettings};

use super::*;

fn metadata(settings: GameSettings) -> RoomMetadata {
    RoomMetadata {
        settings,
        settings_revision: Counter::new(1).unwrap(),
        settings_editor: None,
        room_id: RoomId::new("00000000000000000000000000000001").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        revision: Counter::new(1).unwrap(),
        rules_version: RulesVersion::ExpandedV5,
        map_id: MapId::GeneratedV2,
    }
}

fn wire_settings(items: ItemSet) -> GameSettings {
    GameSettings::new(0, 256, 3, 180, items).unwrap()
}

#[test]
fn v8_identity_and_settings_accept_only_canonical_expanded_tuples() {
    let heart = ItemSet::new([ItemId::Heart]).unwrap();
    assert_eq!(
        GameSettings::for_rules(RulesVersion::ExpandedV5).enabled_items(),
        heart
    );
    let settings = wire_settings(heart);
    let lifecycle = MatchLifecycle::with_settings(
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::HEART),
    );
    let room = project_expanded_room(metadata(settings), &lifecycle).unwrap();
    assert_eq!(room.rules_version, RulesVersion::ExpandedV5);
    assert_eq!(room.map_id, MapId::GeneratedV2);
    assert_eq!(
        room.available_items,
        super::projection_binding::expanded_items()
    );

    let mut crossed = room.clone();
    crossed.map_id = MapId::GeneratedV1;
    assert_eq!(
        super::projection_binding::validate_expanded_room(&crossed),
        Err(ProjectionError::InvalidExpandedBinding)
    );
    crossed = room.clone();
    crossed.rules_version = RulesVersion::ConfigurableV4;
    assert_eq!(
        super::projection_binding::validate_expanded_room(&crossed),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    let unsupported = ItemSet::new([ItemId::Stopwatch]).unwrap();
    assert_eq!(
        expanded_settings(wire_settings(unsupported)),
        Err(ProjectionError::InvalidItems)
    );
}

#[test]
fn empty_v5_preset_keeps_expanded_identity_and_generator() {
    let lifecycle =
        MatchLifecycle::with_settings(RoomSettings::new(SpeedLevel::Base, 256, 3, 180).unwrap());
    let room =
        project_expanded_room(metadata(wire_settings(ItemSet::empty())), &lifecycle).unwrap();
    assert_eq!(room.rules_version, RulesVersion::ExpandedV5);
    assert_eq!(room.map_id, MapId::GeneratedV2);
    assert_eq!(
        room.available_items,
        super::projection_binding::expanded_items()
    );
    assert_eq!(room.settings.enabled_items(), ItemSet::empty());
}

#[test]
fn legacy_v8_identity_rejects_nonempty_item_configuration() {
    let heart = ItemSet::new([ItemId::Heart]).unwrap();
    let mut metadata = metadata(wire_settings(heart));
    metadata.rules_version = RulesVersion::ConfigurableV4;
    metadata.map_id = MapId::GeneratedV1;
    let lifecycle = MatchLifecycle::with_settings(
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::HEART),
    );
    assert_eq!(
        project_expanded_room(metadata, &lifecycle),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

#[test]
fn generator_provenance_must_match_the_rules_identity() {
    let legacy = crate::game::generate_arena([1; 32]).record();
    let expanded = crate::game::generate_expanded_arena([1; 32], EnabledItems::HEART).record();
    let matches = super::projection_binding::generation_matches;
    assert!(matches(RulesVersion::ClassicV1, None));
    assert!(matches(RulesVersion::ClassicV2, Some(legacy)));
    assert!(matches(RulesVersion::ExpandedV5, Some(expanded)));
    assert!(!matches(RulesVersion::ClassicV1, Some(legacy)));
    assert!(!matches(RulesVersion::ClassicV2, Some(expanded)));
    assert!(!matches(RulesVersion::ExpandedV5, Some(legacy)));
    assert!(!matches(RulesVersion::ExpandedV5, None));
}

#[test]
fn v8_authentication_ack_and_client_parser_use_the_v8_contract() {
    let ack = GameServerMessage::Authenticated {
        protocol_version: GameplayVersion,
    };
    assert_eq!(
        serde_json::to_value(ack).unwrap(),
        serde_json::json!({"type":"authenticated","protocolVersion":8})
    );
    let token = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    assert!(
        parse_client_message(&format!(
            r#"{{"type":"authenticate","protocolVersion":8,"token":"{token}"}}"#
        ))
        .is_ok()
    );
}

fn forged_waiting_room() -> RoomView {
    let heart = ItemSet::new([ItemId::Heart]).unwrap();
    RoomView {
        settings: wire_settings(heart),
        settings_revision: Counter::new(1).unwrap(),
        settings_editor: None,
        available_items: super::projection_binding::expanded_items(),
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        round_id: Counter::new(1).unwrap(),
        revision: Counter::new(1).unwrap(),
        tick: Tick::new(0).unwrap(),
        rules_version: RulesVersion::ExpandedV5,
        map_id: MapId::GeneratedV2,
        round_number: RoundCount::new(0).unwrap(),
        scores: Scores {
            one: Wins::new(0).unwrap(),
            two: Wins::new(0).unwrap(),
        },
        last_round: None,
        seats: [
            SeatStatus {
                seat: Seat::One,
                occupied: true,
                ready: false,
                rematch_ready: false,
                connection: ConnectionStatus::Connected,
            },
            SeatStatus {
                seat: Seat::Two,
                occupied: false,
                ready: false,
                rematch_ready: false,
                connection: ConnectionStatus::Vacant,
            },
        ],
        phase: RoomPhase::Waiting,
    }
}

fn null_admission(room: &RoomView) -> Result<String, ProjectionError> {
    prepare_expanded_admission(
        room,
        None,
        None,
        None,
        Seat::One,
        None,
        Counter::new(1).unwrap(),
        ResumeToken::new("c".repeat(64)).unwrap(),
    )
}

#[test]
fn null_admission_validates_the_complete_raw_room_dto() {
    let room = forged_waiting_room();
    assert!(null_admission(&room).is_ok());

    let mut duplicate_seats = room.clone();
    duplicate_seats.seats[1].seat = Seat::One;
    assert_eq!(
        null_admission(&duplicate_seats),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    let mut inconsistent_connection = room.clone();
    inconsistent_connection.seats[1].occupied = true;
    assert_eq!(
        null_admission(&inconsistent_connection),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    let mut invalid_editor = room;
    invalid_editor.settings_editor = Some(Seat::Two);
    assert_eq!(
        null_admission(&invalid_editor),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

fn classic_state_at(tick: u64) -> crate::game::State {
    use crate::game::{ActorInit, Direction, EntityId, Position, RulesConfig, SeatId, World};
    let arena = crate::game::fixed_arena();
    let spawns = arena.spawns();
    crate::game::State::new_at_tick(
        RulesConfig::classic_v1(),
        World::new(arena),
        SeatId::ALL.map(|seat| ActorInit {
            id: EntityId(seat.index() as u32 + 1),
            seat,
            position: Position::center(spawns[seat.index()]),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        }),
        crate::game::Tick(tick),
    )
    .unwrap()
}

fn classic_room(tick: u64) -> RoomView {
    let mut room = forged_waiting_room();
    room.settings = GameSettings::for_rules(RulesVersion::ClassicV1);
    room.available_items = ItemSet::empty();
    room.rules_version = RulesVersion::ClassicV1;
    room.map_id = MapId::FixedV1;
    room.tick = Tick::new(tick).unwrap();
    room.round_number = RoundCount::new(1).unwrap();
    room.seats[1].occupied = true;
    room.seats[1].connection = ConnectionStatus::Connected;
    room.phase = RoomPhase::Running;
    room
}

fn bomb_report(
    state: &crate::game::State,
    placed_at: u64,
    detonates_at: u64,
    placed: bool,
) -> crate::game::StepReport {
    let tick = state.tick();
    let bomb = crate::game::Bomb::new_with_blast(
        crate::game::EntityId(3),
        crate::game::CellId::new(1, 1),
        crate::game::EntityId(1),
        2,
        crate::game::Tick(placed_at),
        crate::game::BombTrigger::Timed {
            detonates_at: crate::game::Tick(detonates_at),
        },
        crate::game::BlastKind::Standard,
    )
    .unwrap();
    let view = bomb.into();
    crate::game::StepReport {
        tick,
        traces: crate::game::SeatId::ALL
            .map(|seat| crate::game::MovementTrace::new(state.actors()[seat.index()].position())),
        events: vec![StepEvent::bomb(
            tick,
            EventAudience::Both,
            if placed {
                StepEventKind::BombPlaced(view)
            } else {
                StepEventKind::BombDetonated(view)
            },
            bomb,
            false,
        )],
    }
}

#[test]
fn v8_binding_checks_configured_fuse_for_world_and_event_bombs() {
    let room = classic_room(10);
    for report in [
        bomb_report(&classic_state_at(10), 10, 11, true),
        bomb_report(&classic_state_at(10), 0, 181, false),
    ] {
        let capture = capture_expanded(&classic_state_at(10), &[report]).unwrap();
        assert_eq!(
            bind_expanded(&room, room.round_id, &capture, None),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }

    let mut state = classic_state_at(10);
    crate::game::test_support::add_test_bomb_with_deadline(
        &mut state,
        crate::game::EntityId(3),
        crate::game::CellId::new(1, 1),
        crate::game::EntityId(1),
        2,
        crate::game::Tick(0),
        crate::game::Tick(11),
    )
    .unwrap();
    let capture = capture_expanded(&state, &[]).unwrap();
    assert_eq!(
        bind_expanded(&room, room.round_id, &capture, None),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

#[test]
fn expanded_capture_rejects_every_special_blast_until_its_item_is_available() {
    let state = classic_state_at(10);
    for blast in [
        crate::game::BlastKind::Piercing,
        crate::game::BlastKind::Power,
    ] {
        let mut report = bomb_report(&state, 10, 190, true);
        let StepEventKind::BombPlaced(ref mut bomb) = report.events[0].kind else {
            panic!("expected placement event");
        };
        bomb.blast = blast;

        assert_eq!(
            capture_expanded(&state, &[report]),
            Err(ProjectionError::InvalidItems)
        );
    }
}

#[test]
fn v8_binding_allows_positive_remaining_fuse_for_chain_detonation() {
    let state = classic_state_at(179);
    let report = bomb_report(&state, 0, 180, false);
    let capture = capture_expanded(&state, &[report]).unwrap();
    let room = classic_room(179);
    assert!(bind_expanded(&room, room.round_id, &capture, None).is_ok());
}

#[test]
fn v8_capture_and_binding_reject_overdue_frozen_bombs_and_accept_the_deadline() {
    let mut finished = classic_state_at(180);
    for seat in crate::game::SeatId::ALL {
        let cell = finished.actors()[seat.index()].position().cell().unwrap();
        finished.insert_test_flame(cell, crate::game::Tick(182));
    }
    finished.step(&[]).unwrap();

    let mut room = classic_room(200);
    let result = crate::protocol::WireRoundResult::Draw {
        reason: crate::protocol::DrawReason::MutualElimination,
    };
    room.last_round = Some(CompletedRound {
        round_id: room.round_id,
        round_number: room.round_id,
        ended_at: Tick::new(181).unwrap(),
        result,
    });
    room.phase = RoomPhase::Result {
        hold_until: Tick::new(361).unwrap(),
    };

    for (placed_at, expected) in [(0, false), (1, true)] {
        let mut state = finished.clone();
        crate::game::test_support::add_test_bomb_with_deadline(
            &mut state,
            crate::game::EntityId(3),
            crate::game::CellId::new(2, 1),
            crate::game::EntityId(1),
            2,
            crate::game::Tick(placed_at),
            crate::game::Tick(placed_at + 180),
        )
        .unwrap();
        let publication = capture_expanded(&state, &[])
            .and_then(|capture| bind_expanded(&room, room.round_id, &capture, None));
        assert_eq!(publication.is_ok(), expected);
    }
}
#[path = "diarrhea_tests.rs"]
mod diarrhea_tests;
#[path = "golden_flame_tests.rs"]
mod golden_flame_tests;
#[path = "invisibility_cloak_tests.rs"]
mod invisibility_cloak_tests;
#[path = "landmine_tests.rs"]
mod landmine_tests;
#[path = "piercing_bomb_tests.rs"]
mod piercing_bomb_tests;
#[path = "power_bomb_tests.rs"]
mod power_bomb_tests;
#[path = "remote_bomb_tests.rs"]
mod remote_bomb_tests;
#[path = "reverse_controls_tests.rs"]
mod reverse_controls_tests;
#[path = "shield_tests.rs"]
mod shield_tests;
#[path = "short_flame_tests.rs"]
mod short_flame_tests;
#[path = "short_fuse_tests.rs"]
mod short_fuse_tests;
#[path = "slow_foot_tests.rs"]
mod slow_foot_tests;

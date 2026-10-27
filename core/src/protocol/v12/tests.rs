use crate::game::SeatId;
use crate::protocol::{Counter, RoomCode, RoomId, Seat};
use crate::room::{CpuProfile, MatchLifecycle, RoomSettings};

use super::*;

fn counter(value: u32) -> Counter {
    Counter::new(value).unwrap()
}

fn metadata(mode: crate::room::RoomMode) -> RoomMetadata {
    let settings_editor = matches!(mode, crate::room::RoomMode::Solo(_)).then_some(Seat::One);
    RoomMetadata {
        settings: GameSettings::new(0, 256, 3, 180, ItemSet::empty()).unwrap(),
        settings_revision: counter(1),
        settings_editor,
        room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
        room_code: RoomCode::new("ABCDEFGH").unwrap(),
        revision: counter(1),
        rules_version: RulesVersion::ExpandedV5,
        map_id: MapId::GeneratedV2,
        mode,
    }
}

fn solo_room() -> RoomView {
    let mut lifecycle = MatchLifecycle::with_settings(RoomSettings::DEFAULT);
    lifecycle.occupy(SeatId::One).unwrap();
    lifecycle.occupy(SeatId::Two).unwrap();
    project_expanded_room(
        metadata(crate::room::RoomMode::Solo(CpuProfile::INITIAL)),
        &lifecycle,
    )
    .unwrap()
}

#[test]
fn create_room_requires_closed_mode_without_client_cpu_authority() {
    for input in [
        r#"{"type":"createRoom","sequence":1,"mode":{"type":"humanDuel"}}"#,
        r#"{"type":"createRoom","sequence":1,"mode":{"type":"solo","difficulty":"standard"}}"#,
    ] {
        assert!(parse_client_message(input).is_ok());
        assert!(crate::protocol::v11::parse_client_message(input).is_err());
    }
    for input in [
        r#"{"type":"createRoom","sequence":1}"#,
        r#"{"type":"createRoom","sequence":1,"mode":{"type":"solo"}}"#,
        r#"{"type":"createRoom","sequence":1,"mode":{"type":"solo","difficulty":"easy"}}"#,
        r#"{"type":"createRoom","sequence":1,"mode":{"type":"solo","difficulty":"standard","seed":7}}"#,
        r#"{"type":"createRoom","sequence":1,"mode":{"type":"solo","difficulty":"standard","policy":"utilityV1"}}"#,
        r#"{"type":"createRoom","sequence":1,"mode":{"type":"solo","difficulty":"standard","seat":"two"}}"#,
    ] {
        assert_eq!(
            parse_client_message(input),
            Err(ParseClientMessageError::InvalidMessage)
        );
    }
}

#[test]
fn projected_modes_assign_only_the_approved_controllers() {
    let solo = solo_room();
    assert_eq!(
        solo.mode,
        RoomMode::Solo {
            difficulty: Difficulty::Standard
        }
    );
    assert_eq!(solo.seats[0].controller, Controller::Human);
    assert_eq!(solo.seats[0].connection, ConnectionStatus::Connected);
    assert_eq!(
        solo.seats[1].controller,
        Controller::Cpu {
            difficulty: Difficulty::Standard
        }
    );
    assert_eq!(solo.seats[1].connection, ConnectionStatus::ServerControlled);
    assert!(solo.seats[1].occupied);

    let mut lifecycle = MatchLifecycle::with_settings(RoomSettings::DEFAULT);
    lifecycle.occupy(SeatId::One).unwrap();
    let duel =
        project_expanded_room(metadata(crate::room::RoomMode::HumanDuel), &lifecycle).unwrap();
    assert_eq!(duel.mode, RoomMode::HumanDuel);
    assert!(
        duel.seats
            .iter()
            .all(|seat| seat.controller == Controller::Human)
    );
    assert_eq!(duel.seats[1].connection, ConnectionStatus::Vacant);
}

#[test]
fn current_and_original_cpu_policies_share_the_v12_room_shape() {
    let mut lifecycle = MatchLifecycle::with_settings(RoomSettings::DEFAULT);
    lifecycle.occupy(SeatId::One).unwrap();
    lifecycle.occupy(SeatId::Two).unwrap();
    for profile in [
        CpuProfile::PULSE_V2,
        CpuProfile::ROUTE_V3,
        CpuProfile::PLANT_V4,
        CpuProfile::GOALS_V5,
        CpuProfile::WEAPONS_V6,
        CpuProfile::WALL_PASS_V7,
        CpuProfile::RESPONSIVE_V8,
        CpuProfile::CURRENT,
    ] {
        let current =
            project_expanded_room(metadata(crate::room::RoomMode::Solo(profile)), &lifecycle)
                .unwrap();
        assert_eq!(current, solo_room());
    }
}

#[test]
fn metadata_validation_rejects_cross_mode_and_controller_combinations() {
    let base = solo_room();
    let mutations: [fn(&mut RoomView); 7] = [
        |room| room.mode = RoomMode::HumanDuel,
        |room| {
            room.seats[0].controller = Controller::Cpu {
                difficulty: Difficulty::Standard,
            }
        },
        |room| room.seats[0].connection = ConnectionStatus::ServerControlled,
        |room| room.seats[1].controller = Controller::Human,
        |room| room.seats[1].connection = ConnectionStatus::Connected,
        |room| room.seats[1].occupied = false,
        |room| room.settings_editor = Some(Seat::Two),
    ];
    for mutate in mutations {
        let mut room = base.clone();
        mutate(&mut room);
        assert_eq!(
            validate_room_view(&room),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }
}

#[test]
fn inherited_room_validation_covers_metadata_and_null_admission() {
    let base = solo_room();
    let mutations: [fn(&mut RoomView); 5] = [
        |room| room.rules_version = RulesVersion::ConfigurableV4,
        |room| room.map_id = MapId::FixedV1,
        |room| {
            room.settings =
                GameSettings::new(0, 256, 3, 180, ItemSet::new([ItemId::Heart]).unwrap()).unwrap();
            room.available_items = ItemSet::empty();
        },
        |room| {
            room.seats[0].ready = true;
            room.seats[1].ready = true;
        },
        |room| {
            room.phase = RoomPhase::Countdown {
                kind: CountdownKind::Initial,
                ends_at: room.tick,
            }
        },
    ];
    for mutate in mutations {
        let mut room = base.clone();
        mutate(&mut room);
        assert_eq!(
            validate_room_view(&room),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }

    let mut invalid = base;
    invalid.rules_version = RulesVersion::ConfigurableV4;
    let result = prepare_admission(
        AdmissionProjection {
            room: &invalid,
            state_epoch: None,
            capture: None,
            generation: None,
        },
        personal(Seat::One),
    );
    assert_eq!(result, Err(ProjectionError::InvalidExpandedBinding));
}

#[test]
fn solo_editor_follows_human_eligibility_until_closing_publication() {
    let base = solo_room();
    let mut missing_owner = base.clone();
    missing_owner.settings_editor = None;
    assert_eq!(
        validate_room_view(&missing_owner),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    let mut closing = base;
    closing.settings_editor = None;
    closing.seats[0].occupied = false;
    closing.seats[0].connection = ConnectionStatus::Vacant;
    assert!(validate_room_view(&closing).is_ok());
}

#[test]
fn solo_admission_is_only_for_connected_human_seat_one() {
    let room = solo_room();
    let projection = || AdmissionProjection {
        room: &room,
        state_epoch: None,
        capture: None,
        generation: None,
    };
    let encoded = prepare_admission(projection(), personal(Seat::One)).unwrap();
    assert!(encoded.contains(r#""mode":{"type":"solo","difficulty":"standard"}"#));
    assert_eq!(
        prepare_admission(projection(), personal(Seat::Two)),
        Err(ProjectionError::InvalidExpandedBinding),
    );
}

fn personal(recipient: Seat) -> PersonalAdmission {
    PersonalAdmission {
        recipient,
        last_action_id: None,
        sequence: counter(2),
        resume_token: ResumeToken::new("f".repeat(64)).unwrap(),
    }
}

#[test]
fn authenticated_serializes_v12_without_activating_the_live_protocol() {
    let encoded = encode_server_message(&GameServerMessage::Authenticated {
        protocol_version: GameplayVersion,
    })
    .unwrap();
    assert_eq!(encoded, r#"{"type":"authenticated","protocolVersion":12}"#);
}

use crate::game::{self, ActorInit, RulesConfig, SeatId, State, World};
use crate::protocol::{Counter, RoomCode, RoomId, Seat};
use crate::room::{MatchLifecycle, RoomSettings};

use super::*;

fn counter(value: u32) -> Counter {
    Counter::new(value).unwrap()
}

fn token(fill: char) -> ResumeToken {
    ResumeToken::new(fill.to_string().repeat(64)).unwrap()
}

fn waiting_room(
    project: fn(RoomMetadata, &MatchLifecycle) -> Result<RoomView, ProjectionError>,
) -> RoomView {
    let settings = GameSettings::new(0, 256, 3, 180, ItemSet::empty()).unwrap();
    let mut lifecycle = MatchLifecycle::with_settings(RoomSettings::DEFAULT);
    lifecycle.occupy(SeatId::One).unwrap();
    project(
        RoomMetadata {
            settings,
            settings_revision: counter(1),
            settings_editor: None,
            room_id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
            room_code: RoomCode::new("ABCDEFGH").unwrap(),
            revision: counter(2),
            rules_version: RulesVersion::ExpandedV5,
            map_id: MapId::GeneratedV2,
        },
        &lifecycle,
    )
    .unwrap()
}

fn running_fixture(
    project: fn(RoomMetadata, &MatchLifecycle) -> Result<RoomView, ProjectionError>,
    capture: fn(&State, &[game::StepReport]) -> Result<ExpandedCapture, ProjectionError>,
) -> (RoomView, ExpandedCapture, game::GenerationRecord) {
    let generated = game::generate_expanded_arena([7; 32], game::EnabledItems::EMPTY);
    let (world, generation) = World::from_generated(generated).unwrap();
    let spawns = world.arena().spawns().to_vec();
    let state = State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT),
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
    let capture = capture(&state, &[]).unwrap();
    let mut room = waiting_room(project);
    room.tick = capture.world.0.physical_tick;
    room.round_number = RoundCount::new(1).unwrap();
    room.seats[1].occupied = true;
    room.seats[1].connection = ConnectionStatus::Connected;
    room.phase = RoomPhase::Running;
    (room, capture, generation)
}

macro_rules! assert_version_parity {
    ($version:ident) => {{
        let room = waiting_room(crate::protocol::$version::project_expanded_room);
        let legacy = crate::protocol::$version::prepare_expanded_admission(
            &room,
            None,
            None,
            None,
            Seat::One,
            None,
            counter(17),
            token('a'),
        )
        .unwrap();
        let typed = crate::protocol::$version::prepare_admission(
            crate::protocol::$version::AdmissionProjection {
                room: &room,
                state_epoch: None,
                capture: None,
                generation: None,
            },
            crate::protocol::$version::PersonalAdmission {
                recipient: Seat::One,
                last_action_id: None,
                sequence: counter(17),
                resume_token: token('a'),
            },
        )
        .unwrap();
        assert_eq!(typed, legacy);

        let decoded: serde_json::Value = serde_json::from_str(&typed).unwrap();
        assert_eq!(decoded["type"], "admitted");
        assert_eq!(decoded["sequence"], 17);
        assert_eq!(decoded["youSeat"], "one");
        assert_eq!(decoded["resumeToken"], "a".repeat(64));
        assert!(decoded["lastActionId"].is_null());
        assert!(decoded["state"].is_null());
        assert_eq!(decoded["room"]["rulesVersion"], 5);
        assert_eq!(decoded["room"]["mapId"], "generated_v2");
        assert!(!typed.contains("\"events\""));
    }};
}

#[test]
fn typed_and_legacy_null_admissions_are_byte_identical_in_all_versions() {
    assert_version_parity!(v8);
    assert_version_parity!(v9);
    assert_version_parity!(v10);
    assert_version_parity!(v11);
}

macro_rules! assert_version_state_parity {
    ($version:ident) => {{
        let (room, capture, generation) = running_fixture(
            crate::protocol::$version::project_expanded_room,
            crate::protocol::$version::capture_expanded,
        );
        let legacy = crate::protocol::$version::prepare_expanded_admission(
            &room,
            Some(room.round_id),
            Some(&capture),
            Some(generation),
            Seat::Two,
            Some(counter(9)),
            counter(27),
            token('d'),
        )
        .unwrap();
        let typed = crate::protocol::$version::prepare_admission(
            crate::protocol::$version::AdmissionProjection {
                room: &room,
                state_epoch: Some(room.round_id),
                capture: Some(&capture),
                generation: Some(generation),
            },
            crate::protocol::$version::PersonalAdmission {
                recipient: Seat::Two,
                last_action_id: Some(counter(9)),
                sequence: counter(27),
                resume_token: token('d'),
            },
        )
        .unwrap();
        assert_eq!(typed, legacy);
        let decoded: serde_json::Value = serde_json::from_str(&typed).unwrap();
        assert_eq!(decoded["sequence"], 27);
        assert_eq!(decoded["youSeat"], "two");
        assert_eq!(decoded["lastActionId"], 9);
        assert!(!decoded["state"].is_null());
        assert!(!typed.contains("\"events\""));
        decoded
    }};
}

#[test]
fn typed_and_legacy_state_admissions_are_byte_identical_in_all_versions() {
    let v8 = assert_version_state_parity!(v8);
    let v9 = assert_version_state_parity!(v9);
    let v10 = assert_version_state_parity!(v10);
    let v11 = assert_version_state_parity!(v11);
    assert!(
        v8["state"]["actors"][0]
            .get("voluntaryActionsFrozen")
            .is_none()
    );
    assert!(v9["state"]["actors"][0]["voluntaryActionsFrozen"].is_boolean());
    assert!(v9["state"]["actors"][0].get("lastTeleportTick").is_none());
    assert!(v10["state"]["actors"][0].get("lastTeleportTick").is_some());
    assert!(v10["state"].get("oilBarrels").is_none());
    assert!(v11["state"]["oilBarrels"].is_array());
}

macro_rules! assert_version_validation {
    ($version:ident) => {{
        let room = waiting_room(crate::protocol::$version::project_expanded_room);
        let prepare_null = |room: &RoomView| {
            crate::protocol::$version::prepare_admission(
                crate::protocol::$version::AdmissionProjection {
                    room,
                    state_epoch: None,
                    capture: None,
                    generation: None,
                },
                crate::protocol::$version::PersonalAdmission {
                    recipient: Seat::One,
                    last_action_id: None,
                    sequence: counter(17),
                    resume_token: token('b'),
                },
            )
        };

        let mut malformed = room.clone();
        malformed.seats[1].seat = Seat::One;
        assert_eq!(
            prepare_null(&malformed),
            Err(ProjectionError::InvalidExpandedBinding)
        );
        let mut vacant = room.clone();
        vacant.seats[0].occupied = false;
        vacant.seats[0].connection = ConnectionStatus::Vacant;
        assert_eq!(
            prepare_null(&vacant),
            Err(ProjectionError::InvalidExpandedBinding)
        );
        let mut running = room.clone();
        running.phase = RoomPhase::Running;
        assert_eq!(
            prepare_null(&running),
            Err(ProjectionError::InvalidExpandedBinding)
        );
        assert_eq!(
            crate::protocol::$version::prepare_admission(
                crate::protocol::$version::AdmissionProjection {
                    room: &room,
                    state_epoch: None,
                    capture: None,
                    generation: None,
                },
                crate::protocol::$version::PersonalAdmission {
                    recipient: Seat::One,
                    last_action_id: Some(counter(1)),
                    sequence: counter(23),
                    resume_token: token('b'),
                },
            ),
            Err(ProjectionError::InvalidExpandedBinding)
        );

        let (running, capture, generation) = running_fixture(
            crate::protocol::$version::project_expanded_room,
            crate::protocol::$version::capture_expanded,
        );
        let personal = || crate::protocol::$version::PersonalAdmission {
            recipient: Seat::One,
            last_action_id: None,
            sequence: counter(31),
            resume_token: token('e'),
        };
        for projection in [
            crate::protocol::$version::AdmissionProjection {
                room: &running,
                state_epoch: Some(running.round_id),
                capture: None,
                generation: Some(generation),
            },
            crate::protocol::$version::AdmissionProjection {
                room: &running,
                state_epoch: None,
                capture: Some(&capture),
                generation: Some(generation),
            },
            crate::protocol::$version::AdmissionProjection {
                room: &running,
                state_epoch: Some(running.round_id),
                capture: Some(&capture),
                generation: None,
            },
            crate::protocol::$version::AdmissionProjection {
                room: &running,
                state_epoch: Some(running.round_id),
                capture: Some(&capture),
                generation: Some(game::generate_arena([8; 32]).record()),
            },
        ] {
            assert_eq!(
                crate::protocol::$version::prepare_admission(projection, personal()),
                Err(ProjectionError::InvalidExpandedBinding)
            );
        }
        assert_eq!(
            crate::protocol::$version::prepare_admission(
                crate::protocol::$version::AdmissionProjection {
                    room: &room,
                    state_epoch: None,
                    capture: None,
                    generation: Some(generation),
                },
                personal(),
            ),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }};
}

#[test]
fn typed_inputs_retain_room_recipient_and_action_validation_in_all_versions() {
    assert_version_validation!(v8);
    assert_version_validation!(v9);
    assert_version_validation!(v10);
    assert_version_validation!(v11);
}

#[test]
fn admission_input_reexports_have_one_type_identity() {
    fn projection_is_v8(_: crate::protocol::v8::AdmissionProjection<'_>) {}
    fn personal_is_v8(_: crate::protocol::v8::PersonalAdmission) {}

    let room = waiting_room(crate::protocol::v8::project_expanded_room);
    macro_rules! assert_aliases {
        ($version:ident) => {
            projection_is_v8(crate::protocol::$version::AdmissionProjection {
                room: &room,
                state_epoch: None,
                capture: None,
                generation: None,
            });
            personal_is_v8(crate::protocol::$version::PersonalAdmission {
                recipient: Seat::One,
                last_action_id: None,
                sequence: counter(1),
                resume_token: token('c'),
            });
        };
    }
    assert_aliases!(v8);
    assert_aliases!(v9);
    assert_aliases!(v10);
    assert_aliases!(v11);
}

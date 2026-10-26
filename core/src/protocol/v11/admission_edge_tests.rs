use crate::game::test_support::add_test_bomb_with_deadline;
use crate::game::{self, ActorInit, EnabledItems, EntityId, RulesConfig, SeatId, State, World};
use crate::protocol::{Counter, RoomCode, RoomId, Seat};
use crate::room::{COUNTDOWN_TICKS, DepartureReason, MatchLifecycle, RoomSettings};

use super::*;

type AdmissionPreparer =
    for<'a> fn(AdmissionProjection<'a>, PersonalAdmission) -> Result<String, ProjectionError>;

fn counter(value: u32) -> Counter {
    Counter::new(value).unwrap()
}

fn token() -> ResumeToken {
    ResumeToken::new("f".repeat(64)).unwrap()
}

fn personal(last_action_id: Option<Counter>) -> PersonalAdmission {
    PersonalAdmission {
        recipient: Seat::Two,
        last_action_id,
        sequence: counter(41),
        resume_token: token(),
    }
}

fn archive_room(template: &RoomView, available_items: ItemSet) -> RoomView {
    let mut room = template.clone();
    room.settings = GameSettings::new(0, 256, 3, 180, ItemSet::empty()).unwrap();
    room.available_items = available_items;
    room
}

#[test]
fn admission_rejects_private_oil_in_archives_before_recipient_projection() {
    let state = super::oil_tests::oil_state();
    let capture = capture_expanded(&state, &[]).unwrap();
    let room = super::oil_tests::room(capture.world.0.enabled_items, capture.world.0.physical_tick);
    let generation = game::generate_expanded_arena([47; 32], super::oil_tests::enabled()).record();
    assert!(
        prepare_admission(
            AdmissionProjection {
                room: &room,
                state_epoch: Some(room.round_id),
                capture: Some(&capture),
                generation: Some(generation),
            },
            personal(None),
        )
        .is_ok()
    );

    let mut private_oil = capture;
    private_oil.rules = RulesConfig::expanded_v5(RoomSettings::DEFAULT);
    private_oil.world.0.enabled_items = ItemSet::empty();
    for (available_items, prepare) in [
        (
            crate::protocol::v8::projection_binding::expanded_items(),
            crate::protocol::v8::prepare_admission as AdmissionPreparer,
        ),
        (
            ItemSet::from_mask(ItemMask::new(1_048_560).unwrap()),
            crate::protocol::v9::prepare_admission as AdmissionPreparer,
        ),
        (
            ItemSet::from_mask(ItemMask::new(3_145_727).unwrap()),
            crate::protocol::v10::prepare_admission as AdmissionPreparer,
        ),
    ] {
        let archived = archive_room(&room, available_items);
        private_oil.world.0.available_items = available_items;
        let archived_generation =
            game::generate_expanded_arena([49; 32], EnabledItems::EMPTY).record();
        let mut neutral = private_oil.clone();
        neutral.world.0.oil.clear();
        neutral.world.0.unlit_oil_grace = [Vec::new(), Vec::new()];
        assert!(
            prepare(
                AdmissionProjection {
                    room: &archived,
                    state_epoch: Some(archived.round_id),
                    capture: Some(&neutral),
                    generation: Some(archived_generation),
                },
                personal(None),
            )
            .is_ok()
        );
        assert_eq!(
            prepare(
                AdmissionProjection {
                    room: &archived,
                    state_epoch: Some(archived.round_id),
                    capture: Some(&private_oil),
                    generation: Some(archived_generation),
                },
                personal(None),
            ),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }
}

fn metadata() -> RoomMetadata {
    RoomMetadata {
        settings: GameSettings::new(0, 256, 3, 180, ItemSet::empty()).unwrap(),
        settings_revision: counter(1),
        settings_editor: None,
        room_id: RoomId::new("fedcba9876543210fedcba9876543210").unwrap(),
        room_code: RoomCode::new("HGFEDCBA").unwrap(),
        revision: counter(1),
        rules_version: RulesVersion::ExpandedV5,
        map_id: MapId::GeneratedV2,
    }
}

fn archived_body() -> (RoomView, ExpandedCapture, game::GenerationRecord, Counter) {
    let mut lifecycle = MatchLifecycle::with_settings(RoomSettings::DEFAULT);
    lifecycle.occupy(SeatId::One).unwrap();
    lifecycle.occupy(SeatId::Two).unwrap();
    lifecycle.set_ready(SeatId::One, true).unwrap();
    lifecycle.set_ready(SeatId::Two, true).unwrap();
    for _ in 0..COUNTDOWN_TICKS {
        lifecycle.advance_idle_tick().unwrap();
    }

    let generated = game::generate_expanded_arena([51; 32], EnabledItems::EMPTY);
    let (world, generation) = World::from_generated(generated).unwrap();
    let spawn = world.arena().spawns()[0];
    let spawns = world.arena().spawns();
    let mut state = State::new_at_tick(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT),
        world,
        SeatId::ALL.map(|seat| ActorInit {
            id: EntityId(seat.index() as u32 + 1),
            seat,
            position: game::Position::center(spawns[seat.index()]),
            heading: game::Direction::East,
            speed: game::SpeedLevel::Base,
        }),
        lifecycle.tick(),
    )
    .unwrap();
    let bomb_tick = state.tick();
    add_test_bomb_with_deadline(
        &mut state,
        EntityId(3),
        spawn,
        EntityId(1),
        2,
        bomb_tick,
        game::Tick(bomb_tick.0 + 1),
    )
    .unwrap();
    state.step(&[]).unwrap();
    let result = state.result().unwrap();
    lifecycle.complete_running_tick(Some(result)).unwrap();
    let capture = capture_expanded(&state, &[]).unwrap();
    assert!(capture.world.0.result.is_some());

    for _ in 0..(COUNTDOWN_TICKS * 2 - 1) {
        lifecycle.advance_idle_tick().unwrap();
    }
    lifecycle
        .depart_with_grace([Some(DepartureReason::TransportLoss), None])
        .unwrap();
    let room = project_expanded_room(metadata(), &lifecycle).unwrap();
    let body_epoch = room.last_round.unwrap().round_id;
    assert!(body_epoch < room.round_id);
    (room, capture, generation, body_epoch)
}

#[test]
fn archived_completed_body_allows_old_epoch_only_without_action_floor() {
    let (room, capture, generation, body_epoch) = archived_body();
    let projection = || AdmissionProjection {
        room: &room,
        state_epoch: Some(body_epoch),
        capture: Some(&capture),
        generation: Some(generation),
    };
    assert!(prepare_admission(projection(), personal(None)).is_ok());
    assert_eq!(
        prepare_admission(projection(), personal(Some(counter(1)))),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

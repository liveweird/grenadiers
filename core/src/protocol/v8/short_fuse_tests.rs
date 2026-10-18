use std::collections::BTreeMap;

use crate::game::{self, ActorInit, EnabledItems, RulesConfig, SeatId, State, World};
use crate::protocol::{Counter, Seat, Tick};
use crate::room::RoomSettings;

use super::super::*;

fn state_at(at: u64, base: u16, enabled: EnabledItems, pickup: bool) -> State {
    let arena = game::fixed_arena();
    let spawns = arena.spawns();
    let exposed = if pickup {
        BTreeMap::from([(spawns[0], game::PickupKind::ShortFuse)])
    } else {
        BTreeMap::new()
    };
    let world = World::with_pickups(arena, BTreeMap::new(), BTreeMap::new(), exposed).unwrap();
    State::new_at_tick(
        RulesConfig::expanded_v5(
            RoomSettings::new(game::SpeedLevel::Base, 256, 3, u64::from(base))
                .unwrap()
                .with_enabled_items(enabled),
        ),
        world,
        SeatId::ALL.map(|seat| ActorInit {
            id: game::EntityId(seat.index() as u32 + 1),
            seat,
            position: game::Position::center(spawns[seat.index()]),
            heading: game::Direction::East,
            speed: game::SpeedLevel::Base,
        }),
        game::Tick(at),
    )
    .unwrap()
}

fn room(at: u64, base: u16, enabled: EnabledItems) -> RoomView {
    let mut room = super::classic_room(at);
    room.rules_version = RulesVersion::ExpandedV5;
    room.map_id = MapId::GeneratedV2;
    room.available_items = super::super::projection_binding::expanded_items();
    room.settings = GameSettings::new(
        0,
        256,
        3,
        base,
        ItemSet::from_mask(ItemMask::new(enabled.bits()).unwrap()),
    )
    .unwrap();
    room
}

fn bind(
    state: &State,
    reports: &[game::StepReport],
    base: u16,
    enabled: EnabledItems,
) -> Result<super::super::projection_facts::BoundPublication, ProjectionError> {
    let capture = capture_expanded(state, reports)?;
    let room = room(state.tick().0, base, enabled);
    let record = game::generate_expanded_arena([0; 32], enabled).record();
    bind_expanded(&room, room.round_id, &capture, Some(record))
}

fn add_bomb(state: &mut State, id: u32, owner: usize, placed: u64, deadline: u64) {
    let cell = state.actors()[owner].position().cell().unwrap();
    game::test_support::add_test_bomb_with_deadline(
        state,
        game::EntityId(id),
        cell,
        game::EntityId(owner as u32 + 1),
        2,
        game::Tick(placed),
        game::Tick(deadline),
    )
    .unwrap();
}

#[test]
fn collected_short_fuse_publishes_real_captured_schedule_and_quiet_remaining_timer() {
    let enabled = EnabledItems::SHORT_FUSE;
    let mut state = state_at(0, 180, enabled, true);
    let collection = state.step(&[]).unwrap();
    let placement = state
        .step(&[game::SeatInput {
            seat: SeatId::One,
            action: game::InputAction::PlaceBomb,
        }])
        .unwrap();
    let publication = bind(&state, &[collection, placement], 180, enabled).unwrap();
    for encoded in prepare_snapshot_pair(&publication).unwrap() {
        let frame: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(frame["state"]["bombs"][0]["trigger"]["remainingTicks"], 90);
        assert_eq!(
            frame["state"]["actors"][0]["effects"][0]["kind"],
            "shortFuse"
        );
        assert_eq!(
            frame["state"]["actors"][0]["effects"][0]["remainingTicks"],
            599
        );
        assert_eq!(frame["events"][0]["kind"]["kind"], "shortFuse");
    }
    assert!(crate::protocol::v7::capture_classic(&state, &[]).is_err());
    let capture = capture_expanded(&state, &[]).unwrap();
    let room = room(2, 180, enabled);
    let record = game::generate_expanded_arena([0; 32], enabled).record();
    let encoded = prepare_expanded_admission(
        &room,
        Some(room.round_id),
        Some(&capture),
        Some(record),
        Seat::One,
        Some(Counter::new(1).unwrap()),
        room.revision,
        ResumeToken::new("c".repeat(64)).unwrap(),
    )
    .unwrap();
    let frame: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert!(frame.get("events").is_none());
    assert_eq!(frame["state"]["bombs"][0]["trigger"]["remainingTicks"], 90);
    assert_eq!(
        frame["state"]["actors"][0]["effects"][0]["remainingTicks"],
        599
    );
}

#[test]
fn normal_and_short_bombs_coexist_without_requiring_a_current_owner_effect() {
    for base in [60_u16, 90, 150, 180, 300] {
        let enabled = EnabledItems::SHORT_FUSE;
        let mut state = state_at(10, base, enabled, false);
        add_bomb(&mut state, 3, 0, 0, u64::from(base));
        add_bomb(&mut state, 4, 1, 0, u64::from(base).div_ceil(2));
        assert!(
            state
                .actors()
                .iter()
                .all(|actor| actor.effect_observations().next().is_none())
        );
        assert!(bind(&state, &[], base, enabled).is_ok());
    }
}

#[test]
fn short_schedules_are_rejected_when_disabled_and_arbitrary_or_doubled_fuses_stay_invalid() {
    for (enabled, deadline) in [
        (EnabledItems::EMPTY, 90),
        (EnabledItems::HEART, 90),
        (EnabledItems::SHORT_FUSE, 91),
        (EnabledItems::SHORT_FUSE, 360),
    ] {
        let mut state = state_at(10, 180, enabled, false);
        add_bomb(&mut state, 3, 0, 0, deadline);
        assert_eq!(
            bind(&state, &[], 180, enabled),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }
}

#[test]
fn short_placement_and_natural_or_early_chain_events_keep_their_captured_schedule() {
    let enabled = EnabledItems::SHORT_FUSE;
    for (at, placed, deadline, placement) in [
        (10, 10, 100, true),
        (99, 10, 100, false),
        (100, 10, 100, false),
        (10, 10, 190, true),
        (99, 10, 190, false),
    ] {
        let state = state_at(at, 180, enabled, false);
        let report = super::bomb_report(&state, placed, deadline, placement);
        assert!(bind(&state, &[report], 180, enabled).is_ok());
    }
    for enabled in [EnabledItems::EMPTY, EnabledItems::HEART] {
        let state = state_at(99, 180, enabled, false);
        let report = super::bomb_report(&state, 10, 100, false);
        assert_eq!(
            bind(&state, &[report], 180, enabled),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }
}

#[test]
fn short_fuse_does_not_enable_paused_remote_contact_or_special_blasts() {
    let enabled = EnabledItems::SHORT_FUSE;
    let mut state = state_at(10, 180, enabled, false);
    add_bomb(&mut state, 3, 0, 0, 90);
    let room = room(10, 180, enabled);
    let record = game::generate_expanded_arena([0; 32], enabled).record();
    let initial = capture_expanded(&state, &[]).unwrap();
    for trigger in [
        BombTrigger::timed(80, true).unwrap(),
        BombTrigger::remote(),
        BombTrigger::contact(),
    ] {
        let mut capture = initial.clone();
        capture.world.0.bombs[0].trigger = trigger;
        assert_eq!(
            bind_expanded(&room, room.round_id, &capture, Some(record)),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }
    let mut capture = initial;
    capture.world.0.bombs[0].blast = BlastKind::Piercing;
    assert_eq!(
        bind_expanded(&room, room.round_id, &capture, Some(record)),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

#[test]
fn overdue_short_bomb_cannot_alias_an_ordinary_deadline_in_a_frozen_result() {
    let enabled = EnabledItems::SHORT_FUSE;
    let mut state = state_at(189, 180, enabled, false);
    for actor in state.actors().to_vec() {
        state.insert_test_flame(actor.position().cell().unwrap(), game::Tick(191));
    }
    state.step(&[]).unwrap();
    assert!(state.result().is_some());
    add_bomb(&mut state, 3, 0, 10, 100);
    assert_eq!(
        capture_expanded(&state, &[]),
        Err(ProjectionError::InvalidBombs)
    );
}

#[test]
fn a_short_bomb_at_its_exact_deadline_can_be_retained_in_a_frozen_result() {
    let enabled = EnabledItems::SHORT_FUSE;
    let mut state = state_at(99, 180, enabled, false);
    for actor in state.actors().to_vec() {
        state.insert_test_flame(actor.position().cell().unwrap(), game::Tick(101));
    }
    state.step(&[]).unwrap();
    add_bomb(&mut state, 3, 0, 10, 100);
    let capture = capture_expanded(&state, &[]).unwrap();
    let mut room = room(120, 180, enabled);
    room.last_round = Some(CompletedRound {
        round_id: room.round_id,
        round_number: room.round_id,
        ended_at: Tick::new(100).unwrap(),
        result: crate::protocol::WireRoundResult::Draw {
            reason: crate::protocol::DrawReason::MutualElimination,
        },
    });
    room.phase = RoomPhase::Result {
        hold_until: Tick::new(280).unwrap(),
    };
    let record = game::generate_expanded_arena([0; 32], enabled).record();
    assert!(bind_expanded(&room, room.round_id, &capture, Some(record)).is_ok());
}

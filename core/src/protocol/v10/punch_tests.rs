use std::collections::{BTreeMap, BTreeSet};

use crate::game::{
    self, ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, InputAction, PickupKind,
    Position, RulesConfig, SeatId, SeatInput, SpeedLevel, State, Terrain as GameTerrain,
    Tick as GameTick, World,
};
use crate::protocol::{Cell, Tick};
use crate::room::RoomSettings;

use super::*;

fn arena() -> Arena {
    let mut terrain = [GameTerrain::Floor; game::arena::ARENA_CELL_COUNT];
    for row in 0..game::arena::ARENA_HEIGHT {
        for column in 0..game::arena::ARENA_WIDTH {
            if row == 0
                || column == 0
                || row == game::arena::ARENA_HEIGHT - 1
                || column == game::arena::ARENA_WIDTH - 1
            {
                terrain[usize::from(row * game::arena::ARENA_WIDTH + column)] = GameTerrain::Wall;
            }
        }
    }
    Arena::new(
        terrain,
        BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap()
}

fn empty_state() -> State {
    let enabled = EnabledItems::PUNCH;
    let mut state = State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled)),
        World::with_pickups(
            arena(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(CellId::new(4, 5), PickupKind::Punch)]),
        )
        .unwrap(),
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position: Position::center(CellId::new(4, 5)),
                heading: Direction::East,
                speed: SpeedLevel::Base,
            },
            ActorInit {
                id: EntityId(2),
                seat: SeatId::Two,
                position: Position::center(CellId::new(13, 11)),
                heading: Direction::West,
                speed: SpeedLevel::Base,
            },
        ],
    )
    .unwrap();
    state.step(&[]).unwrap();
    state
}

fn punched_state(deadline: u64) -> (State, game::StepReport) {
    let mut state = empty_state();
    game::test_support::add_test_bomb_with_deadline(
        &mut state,
        EntityId(9),
        CellId::new(5, 5),
        EntityId(2),
        2,
        GameTick(0),
        GameTick(deadline),
    )
    .unwrap();
    let report = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::UseEquipment,
        }])
        .unwrap();
    (state, report)
}

fn archived_items() -> ItemSet {
    ItemSet::new([
        ItemId::Heart,
        ItemId::RemoteBomb,
        ItemId::PiercingBomb,
        ItemId::PowerBomb,
        ItemId::Landmine,
        ItemId::Shield,
        ItemId::Stopwatch,
        ItemId::InvisibilityCloak,
        ItemId::InvisibleBombs,
        ItemId::GoldenFlame,
        ItemId::ShortFuse,
        ItemId::Diarrhea,
        ItemId::SlowFoot,
        ItemId::ShortFlame,
        ItemId::LongFuse,
        ItemId::ReverseControls,
    ])
    .unwrap()
}

#[test]
fn live_private_punch_facts_validate_and_stay_off_both_recipient_wires() {
    let (state, _) = punched_state(180);
    let capture = capture_expanded(&state, &[]).unwrap();
    assert_eq!(
        capture.world.0.last_successful_punch_ticks[0],
        Some(Tick::new(2).unwrap())
    );
    let bomb = capture.world.0.bombs[0];
    assert_eq!(bomb.cell, Cell::new(8, 5).unwrap());
    assert!(capture.world.0.punch_bombs[&bomb.id].has_been_punched);

    let enabled = ItemSet::new([ItemId::Punch]).unwrap();
    let room = super::kick_tests::room(enabled, capture.world.0.physical_tick);
    let generation = game::generate_expanded_arena([11; 32], EnabledItems::PUNCH).record();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(generation)).unwrap();
    for frame in prepare_snapshot_pair(&publication).unwrap() {
        assert!(!frame.contains("lastSuccessfulPunch"));
        assert!(!frame.contains("hasBeenPunched"));
    }
}

#[test]
fn archived_protocol_rejects_private_history_with_a_neutralized_positive_control() {
    let (state, _) = punched_state(180);
    let mut archived = capture_expanded(&state, &[]).unwrap();
    archived.rules = RulesConfig::expanded_v5(RoomSettings::DEFAULT);
    archived.world.0.actors[0].inventory.equipment = None;
    archived.world.0.available_items = archived_items();
    archived.world.0.enabled_items = ItemSet::empty();
    let mut room = super::kick_tests::room(ItemSet::empty(), archived.world.0.physical_tick);
    room.available_items = archived_items();
    let generation = game::generate_expanded_arena([12; 32], EnabledItems::EMPTY).record();

    let mut control = archived.clone();
    control.world.0.last_successful_punch_ticks = [None; 2];
    for facts in control.world.0.punch_bombs.values_mut() {
        facts.has_been_punched = false;
    }
    assert!(
        crate::protocol::v9::bind_expanded(&room, room.round_id, &control, Some(generation),)
            .is_ok()
    );
    assert_eq!(
        crate::protocol::v9::bind_expanded(&room, room.round_id, &archived, Some(generation),),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

#[test]
fn event_only_history_and_actor_clock_are_validated_before_filtering() {
    let (mut state, _) = punched_state(3);
    let detonation = state.step(&[]).unwrap();
    assert!(state.world().bombs().is_empty());
    let capture = capture_expanded(&state, &[detonation]).unwrap();
    let event = capture.reports.0[0]
        .events
        .iter()
        .find_map(|event| event.punch_bomb)
        .unwrap();
    assert!(event.has_been_punched);

    let mut malformed_event = capture.reports.0.clone();
    malformed_event[0].events[0]
        .punch_bomb
        .as_mut()
        .unwrap()
        .bomb
        .cell = Cell::new(7, 5).unwrap();
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_reports(
            &capture.world,
            malformed_event,
        ),
        Err(ProjectionError::InvalidEvent)
    );

    let mut malformed_clock = capture.world.0;
    malformed_clock.last_successful_punch_ticks[0] =
        Some(Tick::new(malformed_clock.physical_tick.get() + 1).unwrap());
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_world_for_items(
            malformed_clock,
            super::expanded_items(),
        ),
        Err(ProjectionError::InvalidActors)
    );
}

#[test]
fn punched_contact_and_placement_history_are_rejected() {
    let (state, _) = punched_state(180);
    let capture = capture_expanded(&state, &[]).unwrap();
    let bomb = capture.world.0.bombs[0];
    let mut contact = capture.world.0;
    contact.timed_bombs.clear();
    contact.bombs[0].trigger = BombTrigger::contact();
    contact.punch_bombs.get_mut(&bomb.id).unwrap().bomb.trigger = BombTrigger::contact();
    contact.kick_bombs.get_mut(&bomb.id).unwrap().bomb.trigger = BombTrigger::contact();
    contact.enabled_items = ItemSet::new([ItemId::Punch, ItemId::Landmine]).unwrap();
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_world_for_items(
            contact,
            super::expanded_items(),
        ),
        Err(ProjectionError::InvalidBombs)
    );

    let mut placement_state = empty_state();
    let placement_report = placement_state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    let placement_capture = capture_expanded(&placement_state, &[placement_report]).unwrap();
    let mut placement = placement_capture.reports.0.clone();
    let placement_punch = placement[0].events[0].punch_bomb.as_mut().unwrap();
    placement_punch.has_been_punched = true;
    assert!(
        crate::protocol::v8::projection_validation::validate_reports(
            &placement_capture.world,
            placement,
        )
        .is_err()
    );
}

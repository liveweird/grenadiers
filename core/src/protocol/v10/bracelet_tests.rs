use std::collections::{BTreeMap, BTreeSet};

use crate::game::{
    self, ActorInit, Arena, CellId, Direction as GameDirection, EffectKind as GameEffectKind,
    EnabledItems, EntityId as GameEntityId, InputAction, PickupKind as GamePickupKind,
    Position as GamePosition, RulesConfig, SeatId, SeatInput, SpeedLevel, State,
    Terrain as GameTerrain, Tick as GameTick, World,
};
use crate::protocol::{Position, Seat, Tick};
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

fn bracelet_state_with_collection() -> (State, game::StepReport) {
    let world = World::with_pickups(
        arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(CellId::new(4, 4), GamePickupKind::Bracelet)]),
    )
    .unwrap();
    let mut state = State::new(
        RulesConfig::expanded_v5(
            RoomSettings::DEFAULT.with_enabled_items(
                EnabledItems::from_bits(
                    EnabledItems::BRACELET.bits() | EnabledItems::INVISIBILITY_CLOAK.bits(),
                )
                .unwrap(),
            ),
        ),
        world,
        [
            ActorInit {
                id: GameEntityId(1),
                seat: SeatId::One,
                position: GamePosition { x: 4_608, y: 4_096 },
                heading: GameDirection::East,
                speed: SpeedLevel::Base,
            },
            ActorInit {
                id: GameEntityId(2),
                seat: SeatId::Two,
                position: GamePosition { x: 5_248, y: 4_096 },
                heading: GameDirection::West,
                speed: SpeedLevel::Base,
            },
        ],
    )
    .unwrap();
    let report = state.step(&[]).unwrap();
    state.apply_test_effect(SeatId::Two, GameEffectKind::InvisibilityCloak, GameTick(1));
    (state, report)
}

fn bracelet_state() -> State {
    bracelet_state_with_collection().0
}

fn shoved_state() -> (State, game::StepReport) {
    let mut state = bracelet_state();
    let report = state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(GameDirection::East),
        }])
        .unwrap();
    (state, report)
}

fn bracelet_items() -> ItemSet {
    ItemSet::new([ItemId::Bracelet, ItemId::InvisibilityCloak]).unwrap()
}

#[test]
fn private_shove_facts_validate_and_stay_off_visible_and_hidden_wires() {
    let (state, report) = shoved_state();
    let capture = capture_expanded(&state, &[report]).unwrap();
    assert_eq!(
        capture.world.0.last_successful_shove_ticks,
        [Some(Tick::new(2).unwrap()), None]
    );
    let forced = capture.reports.0[0].forced_movement.unwrap();
    assert_eq!(forced.victim, Seat::Two);
    assert_eq!(forced.start, Position::new(5_248, 4_096).unwrap());
    assert_eq!(forced.end, Position::new(6_272, 4_096).unwrap());

    let room = super::kick_tests::room(bracelet_items(), capture.world.0.physical_tick);
    let generation = game::generate_expanded_arena(
        [31; 32],
        EnabledItems::from_bits(
            EnabledItems::BRACELET.bits() | EnabledItems::INVISIBILITY_CLOAK.bits(),
        )
        .unwrap(),
    )
    .record();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(generation)).unwrap();
    let frames = prepare_snapshot_pair(&publication).unwrap();
    let one: serde_json::Value = serde_json::from_str(&frames[0]).unwrap();
    let two: serde_json::Value = serde_json::from_str(&frames[1]).unwrap();
    assert_eq!(
        one["state"]["actors"][0]["inventory"]["equipment"],
        "bracelet"
    );
    assert_eq!(one["state"]["actors"][1]["type"], "hidden");
    assert_eq!(
        one["state"]["actors"][1],
        serde_json::json!({"type":"hidden", "id":2, "seat":"two", "alive":true})
    );
    assert_eq!(two["state"]["actors"][0]["type"], "visible");
    for frame in frames {
        assert!(!frame.contains("lastSuccessfulShove"));
        assert!(!frame.contains("forcedMovement"));
    }
}

#[test]
fn forced_segment_requires_valid_geometry_endpoint_and_opposite_pusher_marker() {
    let (state, report) = shoved_state();
    let capture = capture_expanded(&state, &[report]).unwrap();

    let mut missing = capture.clone();
    missing.world.0.last_successful_shove_ticks = [None; 2];
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_reports(
            &missing.world,
            missing.reports.0,
        ),
        Err(ProjectionError::InvalidReports)
    );

    let mut wrong_actor = capture.clone();
    wrong_actor.world.0.last_successful_shove_ticks = [None, Some(Tick::new(2).unwrap())];
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_reports(
            &wrong_actor.world,
            wrong_actor.reports.0,
        ),
        Err(ProjectionError::InvalidReports)
    );

    for end in [
        Position::new(5_248, 4_096).unwrap(),
        Position::new(6_272, 5_120).unwrap(),
        Position::new(6_273, 4_096).unwrap(),
    ] {
        let mut invalid = capture.clone();
        invalid.reports.0[0].forced_movement.as_mut().unwrap().end = end;
        assert_eq!(
            crate::protocol::v8::projection_validation::validate_reports(
                &invalid.world,
                invalid.reports.0,
            ),
            Err(ProjectionError::InvalidReports)
        );
    }

    let mut endpoint = capture.clone();
    endpoint.world.0.actors[1].position = Position::new(6_271, 4_096).unwrap();
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_reports(
            &endpoint.world,
            endpoint.reports.0,
        ),
        Err(ProjectionError::InvalidReports)
    );
}

#[test]
fn retained_later_marker_and_same_tick_teleport_preserve_historical_forced_reports() {
    let (state, report) = shoved_state();
    let mut capture = capture_expanded(&state, &[report]).unwrap();
    capture.world.0.physical_tick = Tick::new(32).unwrap();
    capture.world.0.running_updates = 32;
    capture.world.0.remaining_ticks -= 30;
    capture.world.0.last_successful_shove_ticks[0] = Some(Tick::new(32).unwrap());
    assert!(
        crate::protocol::v8::projection_validation::validate_reports(
            &capture.world,
            capture.reports.0.clone(),
        )
        .is_ok()
    );

    let mut before_round = capture.clone();
    before_round.world.0.running_updates = 2;
    before_round.world.0.remaining_ticks = MAX_RUNNING_UPDATES - 2;
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_reports(
            &before_round.world,
            before_round.reports.0,
        ),
        Err(ProjectionError::InvalidReports)
    );

    let mut teleported = capture;
    let report_tick = teleported.reports.0[0].tick;
    teleported.world.0.physical_tick = report_tick;
    teleported.world.0.running_updates = u16::try_from(report_tick.get()).unwrap();
    teleported.world.0.remaining_ticks = MAX_RUNNING_UPDATES - teleported.world.0.running_updates;
    teleported.world.0.last_successful_shove_ticks[0] = Some(report_tick);
    teleported.world.0.last_teleport_tick = Some(report_tick);
    teleported.world.0.actors[1].position = Position::new(4_147, 4_096).unwrap();
    assert!(
        crate::protocol::v8::projection_validation::validate_reports(
            &teleported.world,
            teleported.reports.0,
        )
        .is_ok()
    );
}

#[test]
fn marker_must_be_enabled_supported_and_inside_the_physical_round() {
    let (state, report) = shoved_state();
    let capture = capture_expanded(&state, &[report]).unwrap();
    let mut future = capture.world.0.clone();
    future.last_successful_shove_ticks[0] = Some(Tick::new(3).unwrap());
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_world_for_items(
            future,
            super::expanded_items(),
        ),
        Err(ProjectionError::InvalidActors)
    );

    let mut preconstruction = capture.world.0.clone();
    preconstruction.physical_tick = Tick::new(32).unwrap();
    preconstruction.last_successful_shove_ticks[0] = Some(Tick::new(1).unwrap());
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_world_for_items(
            preconstruction,
            super::expanded_items(),
        ),
        Err(ProjectionError::InvalidActors)
    );

    let mut disabled = capture.world.0;
    disabled.enabled_items = ItemSet::new([ItemId::InvisibilityCloak]).unwrap();
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_world_for_items(
            disabled,
            super::expanded_items(),
        ),
        Err(ProjectionError::InvalidActors)
    );
}

#[path = "bracelet_tests/archive_tests.rs"]
mod archive_tests;

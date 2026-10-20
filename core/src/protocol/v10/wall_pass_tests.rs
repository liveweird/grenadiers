use std::collections::{BTreeMap, BTreeSet};

use crate::game::{
    self, ActorInit, Arena, CellId, Direction, EnabledItems, EntityId as GameEntityId, InputAction,
    PickupKind as GamePickupKind, Position as GamePosition, RulesConfig, SeatId, SeatInput,
    SpeedLevel, State, Terrain as GameTerrain, World,
};
use crate::protocol::{Cell, Counter, RoomCode, RoomId, Seat};
use crate::room::RoomSettings;

use super::*;

fn arena(block: CellId) -> Arena {
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
        BTreeSet::from([block]),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap()
}

fn enabled() -> EnabledItems {
    EnabledItems::from_bits(EnabledItems::WALL_PASS.bits() | EnabledItems::HEART.bits()).unwrap()
}

fn wall_pass_state() -> State {
    let pickup = CellId::new(4, 5);
    State::new(
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled())),
        World::with_pickups(
            arena(CellId::new(5, 5)),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(pickup, GamePickupKind::WallPass)]),
        )
        .unwrap(),
        [
            ActorInit {
                id: GameEntityId(1),
                seat: SeatId::One,
                position: GamePosition::center(pickup),
                heading: Direction::East,
                speed: SpeedLevel::Base,
            },
            ActorInit {
                id: GameEntityId(2),
                seat: SeatId::Two,
                position: GamePosition::center(CellId::new(13, 11)),
                heading: Direction::West,
                speed: SpeedLevel::Base,
            },
        ],
    )
    .unwrap()
}

fn holder_inside_block() -> State {
    let mut state = wall_pass_state();
    state.step(&[]).unwrap();
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    for _ in 0..40 {
        if state.actors()[0].position().x > 5 * 1_024 - 320 {
            return state;
        }
        state.step(&[]).unwrap();
    }
    panic!("Wall Pass holder did not enter the adjacent block");
}

fn room(enabled: ItemSet, tick: crate::protocol::Tick) -> RoomView {
    RoomView {
        settings: GameSettings::new(0, 256, 3, 180, enabled).unwrap(),
        settings_revision: Counter::new(1).unwrap(),
        settings_editor: None,
        available_items: super::expanded_items(),
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

fn valid_exit_grace_capture() -> ExpandedCapture {
    let state = holder_inside_block();
    let mut capture = capture_expanded(&state, &[]).unwrap();
    let block = Cell::new(5, 5).unwrap();
    let mut facts = capture.world.0.clone();
    facts.actors[0].inventory.equipment = Some(EquipmentId::Heart);
    facts.soft_wall_grace[0] = vec![block];
    capture.world = crate::protocol::v8::projection_validation::validate_world_for_items(
        facts,
        super::expanded_items(),
    )
    .unwrap();
    capture
}

fn archived_v9_items() -> ItemSet {
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

fn clear_for_v9(capture: &mut ExpandedCapture) -> RoomView {
    capture.rules =
        RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::HEART));
    capture.world.0.available_items = archived_v9_items();
    capture.world.0.enabled_items = ItemSet::new([ItemId::Heart]).unwrap();
    let mut archived_room = room(capture.world.0.enabled_items, capture.world.0.physical_tick);
    archived_room.available_items = archived_v9_items();
    archived_room
}

#[test]
fn active_holder_and_valid_private_exit_grace_bind_without_wire_leakage() {
    let state = holder_inside_block();
    let holder = capture_expanded(&state, &[]).unwrap();
    assert_eq!(
        holder.world.0.actors[0].inventory.equipment,
        Some(EquipmentId::WallPass)
    );
    assert!(holder.world.0.soft_wall_grace[0].is_empty());

    let capture = valid_exit_grace_capture();
    let room = room(capture.world.0.enabled_items, capture.world.0.physical_tick);
    let generation = game::generate_expanded_arena([7; 32], enabled()).record();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(generation)).unwrap();
    for frame in prepare_snapshot_pair(&publication).unwrap() {
        assert!(!frame.contains("softWall"));
        assert!(!frame.contains("wallGrace"));
        let value: serde_json::Value = serde_json::from_str(&frame).unwrap();
        assert_eq!(
            value["state"]["actors"][0]["inventory"]["equipment"],
            "heart"
        );
    }
    let admission = prepare_expanded_admission(
        &room,
        Some(room.round_id),
        Some(&capture),
        Some(generation),
        Seat::One,
        None,
        room.revision,
        ResumeToken::new("a".repeat(64)).unwrap(),
    )
    .unwrap();
    assert!(!admission.contains("softWall"));
    assert!(!admission.contains("wallGrace"));
}

#[test]
fn malformed_private_grace_and_ungraced_soft_overlap_reject_before_projection() {
    let capture = valid_exit_grace_capture();
    let valid = capture.world.0;
    let invalid_cells = [
        vec![],
        vec![Cell::new(4, 5).unwrap()],
        vec![Cell::new(5, 5).unwrap(); 5],
    ];
    for grace in invalid_cells {
        let mut facts = valid.clone();
        facts.soft_wall_grace[0] = grace;
        assert!(
            crate::protocol::v8::projection_validation::validate_world_for_items(
                facts,
                super::expanded_items(),
            )
            .is_err()
        );
    }
    let mut holder_with_grace = valid.clone();
    holder_with_grace.actors[0].inventory.equipment = Some(EquipmentId::WallPass);
    assert!(
        crate::protocol::v8::projection_validation::validate_world_for_items(
            holder_with_grace,
            super::expanded_items(),
        )
        .is_err()
    );
    let mut dead_with_grace = valid;
    dead_with_grace.actors[0].alive = false;
    dead_with_grace.actors[0].inventory.equipment = None;
    assert!(
        crate::protocol::v8::projection_validation::validate_world_for_items(
            dead_with_grace,
            super::expanded_items(),
        )
        .is_err()
    );
}

#[test]
fn archived_bindings_reject_private_grace_after_public_wall_pass_facts_are_cleared() {
    let mut capture = valid_exit_grace_capture();
    let generation = game::generate_expanded_arena([9; 32], EnabledItems::HEART).record();
    let mut archived_room = clear_for_v9(&mut capture);
    assert_eq!(
        crate::protocol::v9::bind_expanded(
            &archived_room,
            archived_room.round_id,
            &capture,
            Some(generation),
        ),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    capture.world.0.available_items = crate::protocol::v8::projection_binding::expanded_items();
    archived_room.available_items = crate::protocol::v8::projection_binding::expanded_items();
    assert_eq!(
        crate::protocol::v8::bind_expanded(
            &archived_room,
            archived_room.round_id,
            &capture,
            Some(generation),
        ),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

#[test]
fn archives_reject_wall_pass_equipment_pickup_event_history_and_configuration() {
    let generation = game::generate_expanded_arena([11; 32], EnabledItems::HEART).record();
    let state = holder_inside_block();
    assert_eq!(
        crate::protocol::v9::capture_expanded(&state, &[]),
        Err(ProjectionError::InvalidItems)
    );
    assert_eq!(
        crate::protocol::v8::capture_expanded(&state, &[]),
        Err(ProjectionError::InvalidItems)
    );

    let mut equipment = capture_expanded(&state, &[]).unwrap();
    let equipment_room = clear_for_v9(&mut equipment);
    assert_eq!(
        crate::protocol::v9::bind_expanded(
            &equipment_room,
            equipment_room.round_id,
            &equipment,
            Some(generation),
        ),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    let mut pickup = capture_expanded(&wall_pass_state(), &[]).unwrap();
    let pickup_room = clear_for_v9(&mut pickup);
    assert_eq!(
        crate::protocol::v9::bind_expanded(
            &pickup_room,
            pickup_room.round_id,
            &pickup,
            Some(generation),
        ),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    let mut collected_state = wall_pass_state();
    let report = collected_state.step(&[]).unwrap();
    let mut history = capture_expanded(&collected_state, &[report]).unwrap();
    history.world.0.actors[0].inventory.equipment = None;
    let history_room = clear_for_v9(&mut history);
    assert_eq!(
        crate::protocol::v9::bind_expanded(
            &history_room,
            history_room.round_id,
            &history,
            Some(generation),
        ),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    let settings =
        GameSettings::new(0, 256, 3, 180, ItemSet::new([ItemId::WallPass]).unwrap()).unwrap();
    assert_eq!(
        crate::protocol::v9::expanded_settings(settings),
        Err(ProjectionError::InvalidItems)
    );
    assert_eq!(
        crate::protocol::v8::expanded_settings(settings),
        Err(ProjectionError::InvalidItems)
    );
}

use std::collections::{BTreeMap, BTreeSet};

use crate::game::{
    self, ActorInit, Arena, BombTrigger as GameBombTrigger, CellId, Direction as GameDirection,
    EnabledItems, EntityId as GameEntityId, InputAction, PickupKind as GamePickupKind,
    Position as GamePosition, RulesConfig, SeatId, SeatInput, SpeedLevel, State,
    Terrain as GameTerrain, Tick as GameTick, World,
};
use crate::protocol::{Cell, Counter, RoomCode, RoomId, Seat};
use crate::room::RoomSettings;

use super::*;

fn enabled(remote: bool) -> EnabledItems {
    let bits = EnabledItems::KICK.bits()
        | if remote {
            EnabledItems::REMOTE_BOMB.bits()
        } else {
            0
        };
    EnabledItems::from_bits(bits).unwrap()
}

fn arena(blocks: BTreeSet<CellId>) -> Arena {
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
    Arena::new(terrain, blocks, [CellId::new(1, 1), CellId::new(13, 11)]).unwrap()
}

fn kick_state(trigger: GameBombTrigger, blocks: BTreeSet<CellId>) -> State {
    let bomb_cell = CellId::new(4, 5);
    let pickup_cell = CellId::new(3, 5);
    let center = GamePosition::center(bomb_cell);
    let mut state = State::new(
        RulesConfig::expanded_v5(
            RoomSettings::DEFAULT
                .with_enabled_items(enabled(matches!(trigger, GameBombTrigger::Remote))),
        ),
        World::with_pickups(
            arena(blocks),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(pickup_cell, GamePickupKind::Kick)]),
        )
        .unwrap(),
        [
            ActorInit {
                id: GameEntityId(1),
                seat: SeatId::One,
                position: GamePosition {
                    x: i32::from(bomb_cell.column) * 1_024 - 320,
                    y: center.y,
                },
                heading: GameDirection::East,
                speed: SpeedLevel::Base,
            },
            ActorInit {
                id: GameEntityId(2),
                seat: SeatId::Two,
                position: GamePosition::center(CellId::new(13, 11)),
                heading: GameDirection::West,
                speed: SpeedLevel::Base,
            },
        ],
    )
    .unwrap();
    state.step(&[]).unwrap();
    game::test_support::add_test_bomb_with_trigger(
        &mut state,
        GameEntityId(9),
        bomb_cell,
        GameEntityId(1),
        2,
        GameTick(0),
        trigger,
    )
    .unwrap();
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(GameDirection::East),
        }])
        .unwrap();
    state
}

fn timed_state(blocked: bool) -> State {
    let blocks = if blocked {
        BTreeSet::from([CellId::new(5, 5)])
    } else {
        BTreeSet::new()
    };
    let mut state = kick_state(
        GameBombTrigger::Timed {
            detonates_at: GameTick(180),
        },
        blocks,
    );
    if blocked {
        state.step(&[]).unwrap();
    }
    state
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
fn live_capture_binding_and_admission_keep_active_motion_private() {
    let state = timed_state(false);
    let capture = capture_expanded(&state, &[]).unwrap();
    let bomb = capture.world.0.bombs[0];
    let kick = capture.world.0.kick_bombs[&bomb.id];
    assert_eq!(kick.bomb, bomb);
    assert!(kick.has_been_kicked);
    assert_eq!(kick.motion.unwrap().countdown, 0);
    assert_eq!(
        kick.motion.unwrap().direction,
        crate::protocol::Direction::East
    );

    let room = room(capture.world.0.enabled_items, capture.world.0.physical_tick);
    let generation = game::generate_expanded_arena([7; 32], EnabledItems::KICK).record();
    let publication = bind_expanded(&room, room.round_id, &capture, Some(generation)).unwrap();
    let mut frames = prepare_snapshot_pair(&publication).unwrap().to_vec();
    frames.push(
        prepare_expanded_admission(
            &room,
            Some(room.round_id),
            Some(&capture),
            Some(generation),
            Seat::One,
            None,
            room.revision,
            ResumeToken::new("a".repeat(64)).unwrap(),
        )
        .unwrap(),
    );
    for frame in frames {
        for private in ["motion", "countdown", "hasBeenKicked", "direction"] {
            assert!(!frame.contains(private));
        }
    }
}

#[test]
fn malformed_live_private_facts_reject_capture_revalidation_and_binding() {
    let state = timed_state(false);
    let capture = capture_expanded(&state, &[]).unwrap();
    let bomb = capture.world.0.bombs[0];
    let mut malformed = capture.world.0.clone();
    malformed
        .kick_bombs
        .get_mut(&bomb.id)
        .unwrap()
        .motion
        .as_mut()
        .unwrap()
        .countdown = 10;
    malformed.enabled_items = ItemSet::new([ItemId::Kick, ItemId::InvisibleBombs]).unwrap();
    malformed.actors[1].effects = EffectViews::new(vec![
        EffectView::new(EffectKind::InvisibleBombs, 1).unwrap(),
    ])
    .unwrap();
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_world_for_items(
            malformed.clone(),
            super::expanded_items(),
        ),
        Err(ProjectionError::InvalidBombs)
    );

    let mut rebound = capture;
    rebound.world.0 = malformed;
    let room = room(rebound.world.0.enabled_items, rebound.world.0.physical_tick);
    let generation = game::generate_expanded_arena([7; 32], EnabledItems::KICK).record();
    assert_eq!(
        bind_expanded(&room, room.round_id, &rebound, Some(generation)),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

#[test]
fn private_fact_association_history_and_contact_exclusion_are_validated() {
    let capture = capture_expanded(&timed_state(false), &[]).unwrap();
    let bomb = capture.world.0.bombs[0];
    for mutation in 0..3 {
        let mut facts = capture.world.0.clone();
        let kick = facts.kick_bombs.get_mut(&bomb.id).unwrap();
        match mutation {
            0 => kick.bomb.cell = Cell::new(6, 5).unwrap(),
            1 => kick.bomb.owner = crate::protocol::EntityId::new(2),
            _ => kick.has_been_kicked = false,
        }
        assert!(
            crate::protocol::v8::projection_validation::validate_world_for_items(
                facts,
                super::expanded_items(),
            )
            .is_err()
        );
    }

    let mut contact = capture.world.0;
    contact.timed_bombs.clear();
    contact.bombs[0].trigger = BombTrigger::contact();
    contact.kick_bombs.get_mut(&bomb.id).unwrap().bomb.trigger = BombTrigger::contact();
    contact.enabled_items = ItemSet::new([ItemId::Kick, ItemId::Landmine]).unwrap();
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_world_for_items(
            contact,
            super::expanded_items(),
        ),
        Err(ProjectionError::InvalidBombs)
    );
}

#[test]
fn stopped_and_event_only_history_survive_capture_and_archives_reject_it() {
    let mut stopped = capture_expanded(&timed_state(true), &[]).unwrap();
    let stopped_fact = stopped.world.0.kick_bombs.values().next().unwrap();
    assert!(stopped_fact.has_been_kicked);
    assert_eq!(stopped_fact.motion, None);

    stopped.rules = RulesConfig::expanded_v5(RoomSettings::DEFAULT);
    stopped.world.0.actors[0].inventory.equipment = None;
    stopped.world.0.available_items = archived_items();
    stopped.world.0.enabled_items = ItemSet::empty();
    let mut stopped_room = room(ItemSet::empty(), stopped.world.0.physical_tick);
    stopped_room.available_items = archived_items();
    let empty_generation = game::generate_expanded_arena([8; 32], EnabledItems::EMPTY).record();
    let mut stopped_control = stopped.clone();
    stopped_control
        .world
        .0
        .kick_bombs
        .values_mut()
        .next()
        .unwrap()
        .has_been_kicked = false;
    assert!(
        crate::protocol::v9::bind_expanded(
            &stopped_room,
            stopped_room.round_id,
            &stopped_control,
            Some(empty_generation),
        )
        .is_ok()
    );
    assert_eq!(
        crate::protocol::v9::bind_expanded(
            &stopped_room,
            stopped_room.round_id,
            &stopped,
            Some(empty_generation),
        ),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    let mut remote = kick_state(GameBombTrigger::Remote, BTreeSet::from([CellId::new(5, 5)]));
    remote.step(&[]).unwrap();
    remote
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Release(GameDirection::East),
            },
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Press(GameDirection::West),
            },
        ])
        .unwrap();
    for update in 0..60 {
        if update > 0 && update % 8 == 0 {
            remote
                .step(&[SeatInput {
                    seat: SeatId::One,
                    action: InputAction::Press(GameDirection::West),
                }])
                .unwrap();
        } else {
            remote.step(&[]).unwrap();
        }
        if remote.actors()[0].position().x <= GamePosition::center(CellId::new(1, 5)).x {
            break;
        }
    }
    assert!(remote.actors()[0].position().x <= GamePosition::center(CellId::new(1, 5)).x);
    remote
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Release(GameDirection::West),
        }])
        .unwrap();
    let report = remote
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    assert_eq!(remote.result(), None);
    let mut event_only = capture_expanded(&remote, &[report]).unwrap();
    assert!(event_only.world.0.bombs.is_empty());
    let event_kick = event_only.reports.0[0]
        .events
        .iter()
        .find_map(|event| event.kick_bomb)
        .unwrap();
    assert!(event_kick.has_been_kicked);
    assert_eq!(event_kick.motion, None);

    let mut malformed_reports = event_only.reports.0.clone();
    malformed_reports[0].events[0]
        .kick_bomb
        .as_mut()
        .unwrap()
        .bomb
        .cell = Cell::new(6, 5).unwrap();
    assert_eq!(
        crate::protocol::v8::projection_validation::validate_reports(
            &event_only.world,
            malformed_reports,
        ),
        Err(ProjectionError::InvalidEvent)
    );

    let remote_only = ItemSet::new([ItemId::RemoteBomb]).unwrap();
    event_only.rules = RulesConfig::expanded_v5(
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::REMOTE_BOMB),
    );
    event_only.world.0.actors[0].inventory.equipment = None;
    event_only.world.0.available_items = archived_items();
    event_only.world.0.enabled_items = remote_only;
    let mut archived_room = room(remote_only, event_only.world.0.physical_tick);
    archived_room.available_items = archived_items();
    let generation = game::generate_expanded_arena([9; 32], EnabledItems::REMOTE_BOMB).record();
    let mut event_control = event_only.clone();
    for control_kick in event_control.reports.0[0]
        .events
        .iter_mut()
        .filter_map(|event| event.kick_bomb.as_mut())
    {
        control_kick.motion = None;
        control_kick.has_been_kicked = false;
    }
    let control_world = crate::protocol::v8::projection_validation::validate_world_for_items(
        event_control.world.0.clone(),
        archived_items(),
    )
    .unwrap();
    crate::protocol::v8::projection_validation::validate_reports(
        &control_world,
        event_control.reports.0.clone(),
    )
    .unwrap();
    assert!(
        crate::protocol::v9::bind_expanded(
            &archived_room,
            archived_room.round_id,
            &event_control,
            Some(generation),
        )
        .is_ok()
    );
    assert_eq!(
        crate::protocol::v9::bind_expanded(
            &archived_room,
            archived_room.round_id,
            &event_only,
            Some(generation),
        ),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

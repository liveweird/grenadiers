use super::*;

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

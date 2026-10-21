use super::*;

fn archived_capture(
    mut capture: ExpandedCapture,
    available: ItemSet,
    retain_marker: bool,
    retain_live_history: bool,
    retain_event_history: bool,
) -> ExpandedCapture {
    capture.rules = RulesConfig::expanded_v5(RoomSettings::DEFAULT);
    capture.world.0.actors[0].inventory.equipment = None;
    capture.world.0.available_items = available;
    capture.world.0.enabled_items = ItemSet::empty();
    if !retain_marker {
        capture.world.0.last_successful_punch_ticks = [None; 2];
    }
    if !retain_live_history {
        for facts in capture.world.0.punch_bombs.values_mut() {
            facts.has_been_punched = false;
        }
    }
    if !retain_event_history {
        for facts in capture
            .reports
            .0
            .iter_mut()
            .flat_map(|report| &mut report.events)
            .filter_map(|event| event.punch_bomb.as_mut())
        {
            facts.has_been_punched = false;
        }
    }
    capture
}

fn archived_room(available: ItemSet, tick: Tick) -> RoomView {
    let mut room = super::super::kick_tests::room(ItemSet::empty(), tick);
    room.available_items = available;
    room
}

fn generation() -> game::GenerationRecord {
    game::generate_expanded_arena([12; 32], EnabledItems::EMPTY).record()
}

#[test]
fn v8_and_v9_accept_neutral_controls_and_reject_cooldown_alone() {
    let (state, _) = punched_state(180);
    let current = capture_expanded(&state, &[]).unwrap();
    let marker = current.world.0.last_successful_punch_ticks[0];
    assert!(marker.is_some());

    let v8_items = crate::protocol::v8::projection_binding::expanded_items();
    let v8_control = archived_capture(current.clone(), v8_items, false, false, false);
    let v8_room = archived_room(v8_items, v8_control.world.0.physical_tick);
    assert!(
        crate::protocol::v8::bind_expanded(
            &v8_room,
            v8_room.round_id,
            &v8_control,
            Some(generation()),
        )
        .is_ok()
    );
    let mut v8_cooldown = v8_control;
    v8_cooldown.world.0.last_successful_punch_ticks[0] = marker;
    assert_eq!(
        crate::protocol::v8::bind_expanded(
            &v8_room,
            v8_room.round_id,
            &v8_cooldown,
            Some(generation()),
        ),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    let v9_items = archived_items();
    let v9_control = archived_capture(current, v9_items, false, false, false);
    let v9_room = archived_room(v9_items, v9_control.world.0.physical_tick);
    assert!(
        crate::protocol::v9::bind_expanded(
            &v9_room,
            v9_room.round_id,
            &v9_control,
            Some(generation()),
        )
        .is_ok()
    );
    let mut v9_cooldown = v9_control;
    v9_cooldown.world.0.last_successful_punch_ticks[0] = marker;
    assert_eq!(
        crate::protocol::v9::bind_expanded(
            &v9_room,
            v9_room.round_id,
            &v9_cooldown,
            Some(generation()),
        ),
        Err(ProjectionError::InvalidExpandedBinding)
    );
}

#[test]
fn live_history_policy_isolated_from_association_rejects_v8_and_v9() {
    let (state, _) = punched_state(180);
    let current = capture_expanded(&state, &[]).unwrap();
    let bomb = current.world.0.bombs[0];
    let facts = current.world.0.punch_bombs[&bomb.id];
    assert!(
        crate::protocol::v8::projection_punch::validate_bomb_history_for_test(
            &current.world.0,
            super::super::expanded_items(),
            bomb,
            facts,
            false,
        )
        .is_ok()
    );

    for archived in [
        crate::protocol::v8::projection_binding::expanded_items(),
        archived_items(),
    ] {
        let mut world = current.world.0.clone();
        world.available_items = archived;
        world.enabled_items = ItemSet::empty();
        assert!(
            crate::protocol::v8::projection_punch::validate_bomb_history_for_test(
                &world, archived, bomb, facts, false,
            )
            .is_err()
        );
    }
}

#[test]
fn event_only_history_has_v8_and_v9_controls_and_isolated_policy_rejections() {
    let (state, detonation) = event_only_state();
    let current = capture_expanded(&state, &[detonation]).unwrap();
    let facts = current.reports.0[0]
        .events
        .iter()
        .find_map(|event| event.punch_bomb)
        .unwrap();
    assert!(facts.has_been_punched);
    assert!(
        crate::protocol::v8::projection_punch::validate_bomb_history_for_test(
            &current.world.0,
            super::super::expanded_items(),
            facts.bomb,
            facts,
            false,
        )
        .is_ok()
    );

    let v8_items = crate::protocol::v8::projection_binding::expanded_items();
    let v8_control = archived_capture(current.clone(), v8_items, false, false, false);
    let v8_room = archived_room(v8_items, v8_control.world.0.physical_tick);
    assert!(
        crate::protocol::v8::bind_expanded(
            &v8_room,
            v8_room.round_id,
            &v8_control,
            Some(generation()),
        )
        .is_ok()
    );
    let v8_history = archived_capture(current.clone(), v8_items, true, false, true);
    assert_eq!(
        crate::protocol::v8::bind_expanded(
            &v8_room,
            v8_room.round_id,
            &v8_history,
            Some(generation()),
        ),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    let v9_items = archived_items();
    let v9_control = archived_capture(current.clone(), v9_items, false, false, false);
    let v9_room = archived_room(v9_items, v9_control.world.0.physical_tick);
    assert!(
        crate::protocol::v9::bind_expanded(
            &v9_room,
            v9_room.round_id,
            &v9_control,
            Some(generation()),
        )
        .is_ok()
    );
    let v9_history = archived_capture(current.clone(), v9_items, true, false, true);
    assert_eq!(
        crate::protocol::v9::bind_expanded(
            &v9_room,
            v9_room.round_id,
            &v9_history,
            Some(generation()),
        ),
        Err(ProjectionError::InvalidExpandedBinding)
    );

    for archived in [v8_items, v9_items] {
        let mut world = current.world.0.clone();
        world.available_items = archived;
        world.enabled_items = ItemSet::empty();
        assert!(
            crate::protocol::v8::projection_punch::validate_bomb_history_for_test(
                &world, archived, facts.bomb, facts, false,
            )
            .is_err()
        );
    }
}

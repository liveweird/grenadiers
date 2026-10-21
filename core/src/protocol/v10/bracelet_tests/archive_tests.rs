use super::*;

fn v9_items() -> ItemSet {
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

fn neutral_capture(mut capture: ExpandedCapture, available: ItemSet) -> ExpandedCapture {
    capture.rules = RulesConfig::expanded_v5(RoomSettings::DEFAULT);
    capture.world.0.available_items = available;
    capture.world.0.enabled_items = ItemSet::empty();
    capture.world.0.last_successful_shove_ticks = [None; 2];
    for actor in &mut capture.world.0.actors {
        actor.inventory.equipment = None;
        actor.effects = EffectViews::new(Vec::new()).unwrap();
    }
    capture.reports.0[0].forced_movement = None;
    capture
}

fn archived_room(available: ItemSet, tick: Tick) -> RoomView {
    let mut room = super::super::kick_tests::room(ItemSet::empty(), tick);
    room.available_items = available;
    room
}

fn generation() -> game::GenerationRecord {
    game::generate_expanded_arena([46; 32], EnabledItems::EMPTY).record()
}

fn bind_v8(
    room: &RoomView,
    capture: &ExpandedCapture,
) -> Result<BoundPublication, ProjectionError> {
    crate::protocol::v8::bind_expanded(room, room.round_id, capture, Some(generation()))
}

fn bind_v9(
    room: &RoomView,
    capture: &ExpandedCapture,
) -> Result<BoundPublication, ProjectionError> {
    crate::protocol::v9::bind_expanded(room, room.round_id, capture, Some(generation()))
}

type ArchivedBind = fn(&RoomView, &ExpandedCapture) -> Result<BoundPublication, ProjectionError>;

#[test]
fn v8_and_v9_neutral_controls_reject_bracelet_settings_and_gear_independently() {
    let (state, report) = shoved_state();
    let current = capture_expanded(&state, &[report]).unwrap();
    let archives: [(ItemSet, ArchivedBind); 2] = [
        (
            crate::protocol::v8::projection_binding::expanded_items(),
            bind_v8,
        ),
        (v9_items(), bind_v9),
    ];
    for (items, bind) in archives {
        let control = neutral_capture(current.clone(), items);
        let room = archived_room(items, control.world.0.physical_tick);
        assert!(bind(&room, &control).is_ok());

        let mut gear = control.clone();
        gear.world.0.actors[0].inventory.equipment = Some(EquipmentId::Bracelet);
        assert_eq!(
            bind(&room, &gear),
            Err(ProjectionError::InvalidExpandedBinding)
        );

        let mut settings = room.clone();
        settings.settings =
            GameSettings::new(0, 256, 3, 180, ItemSet::new([ItemId::Bracelet]).unwrap()).unwrap();
        assert_eq!(
            bind(&settings, &control),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }
}

#[test]
fn v8_and_v9_neutral_controls_reject_bare_private_shove_marker() {
    let (state, report) = shoved_state();
    let current = capture_expanded(&state, &[report]).unwrap();
    let marker = current.world.0.last_successful_shove_ticks[0];
    assert!(marker.is_some());
    let archives: [(ItemSet, ArchivedBind); 2] = [
        (
            crate::protocol::v8::projection_binding::expanded_items(),
            bind_v8,
        ),
        (v9_items(), bind_v9),
    ];
    for (items, bind) in archives {
        let control = neutral_capture(current.clone(), items);
        let room = archived_room(items, control.world.0.physical_tick);
        assert!(bind(&room, &control).is_ok());

        let mut private = control;
        private.world.0.last_successful_shove_ticks[0] = marker;
        assert_eq!(
            bind(&room, &private),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }
}

#[test]
fn forced_history_policy_isolated_from_marker_association_rejects_archives() {
    let (state, report) = shoved_state();
    let current = capture_expanded(&state, &[report]).unwrap();
    let report = &current.reports.0[0];
    assert!(
        crate::protocol::v8::projection_bracelet::validate_report_for_test(
            &current.world.0,
            report,
            super::super::expanded_items(),
        )
        .is_ok()
    );
    for items in [
        crate::protocol::v8::projection_binding::expanded_items(),
        v9_items(),
    ] {
        assert_eq!(
            crate::protocol::v8::projection_bracelet::validate_report_for_test(
                &current.world.0,
                report,
                items,
            ),
            Err(ProjectionError::InvalidReports)
        );
    }
}

#[test]
fn v8_and_v9_controls_reject_bracelet_pickup_history_after_gear_is_cleared() {
    let (state, collection) = bracelet_state_with_collection();
    let current = capture_expanded(&state, &[collection]).unwrap();
    assert!(matches!(
        current.reports.0[0].events[0].kind,
        EventKind::PickupCollected {
            kind: PickupKind::Bracelet,
            ..
        }
    ));
    let archives: [(ItemSet, ArchivedBind); 2] = [
        (
            crate::protocol::v8::projection_binding::expanded_items(),
            bind_v8,
        ),
        (v9_items(), bind_v9),
    ];
    for (items, bind) in archives {
        let history = neutral_capture(current.clone(), items);
        let room = archived_room(items, history.world.0.physical_tick);
        let mut control = history.clone();
        control.reports.0[0].events.clear();
        assert!(bind(&room, &control).is_ok());
        assert_eq!(
            bind(&room, &history),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }
}

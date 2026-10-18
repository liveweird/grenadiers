use crate::protocol::{Counter, MapId, RulesVersion, Seat};

use super::projection_facts::ProjectionError;
use super::projection_test_support::{classic_state, room};
use super::*;

#[test]
fn classic_admission_is_personal_checked_and_rejects_unavailable_settings() {
    let state = classic_state();
    let capture = capture_classic(&state, &[]).unwrap();
    let room = room(
        0,
        GameSettings::for_rules(RulesVersion::ClassicV1),
        ItemSet::empty(),
    );
    let encoded = prepare_classic_admission(
        &room,
        Some(room.round_id),
        Some(&capture),
        Seat::One,
        Some(Counter::new(7).unwrap()),
        Counter::new(9).unwrap(),
        ResumeToken::new("a".repeat(64)).unwrap(),
    )
    .unwrap();
    let json: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(json["youSeat"], "one");
    assert_eq!(json["lastActionId"], 7);
    assert_eq!(json["state"]["actors"][0]["seat"], "one");

    let enabled = ItemSet::new([ItemId::Heart]).unwrap();
    let settings = GameSettings::new(0, 128, 3, 180, enabled).unwrap();
    assert_eq!(
        classic_settings(settings),
        Err(ProjectionError::InvalidItems)
    );
}

#[test]
fn null_classic_admission_validates_the_complete_waiting_room() {
    let mut waiting = room(
        0,
        GameSettings::for_rules(RulesVersion::ClassicV1),
        ItemSet::empty(),
    );
    waiting.round_number = RoundCount::new(0).unwrap();
    waiting.phase = RoomPhase::Waiting;
    let admit = |room: &RoomView| {
        prepare_classic_admission(
            room,
            None,
            None,
            Seat::One,
            None,
            Counter::new(1).unwrap(),
            ResumeToken::new("b".repeat(64)).unwrap(),
        )
    };
    let encoded = admit(&waiting).unwrap();
    assert!(serde_json::from_str::<serde_json::Value>(&encoded).unwrap()["state"].is_null());

    let mut items = waiting.clone();
    items.available_items = ItemSet::new([ItemId::Heart]).unwrap();
    assert_eq!(admit(&items), Err(ProjectionError::InvalidClassicBinding));
    let mut map = waiting.clone();
    map.map_id = MapId::GeneratedV1;
    assert_eq!(admit(&map), Err(ProjectionError::InvalidClassicBinding));
    let mut rules = waiting.clone();
    rules.rules_version = RulesVersion::ClassicV2;
    rules.map_id = MapId::GeneratedV1;
    assert!(admit(&rules).is_ok());
    rules.rules_version = RulesVersion::ClassicV3;
    assert_eq!(admit(&rules), Err(ProjectionError::InvalidClassicBinding));
    let mut structure = waiting;
    structure.seats.swap(0, 1);
    assert_eq!(
        admit(&structure),
        Err(ProjectionError::InvalidClassicBinding)
    );
}

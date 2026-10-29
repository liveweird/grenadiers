use super::*;

#[test]
fn equipment_reaches_the_shared_room_action_path() {
    let id = Counter::new(7).unwrap();
    assert_eq!(
        manual_room_action(ManualActionKind::Equipment, id),
        Some(RoomAction::Equipment(7))
    );
    assert_eq!(
        manual_room_action(ManualActionKind::Bomb, id),
        Some(RoomAction::Bomb(7))
    );
}

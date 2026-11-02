use super::*;

#[test]
fn bomb_reaches_the_shared_room_action_path() {
    let id = Counter::new(7).unwrap();
    assert_eq!(
        manual_room_action(ManualActionKind::Bomb, id),
        Some(RoomAction::Bomb(7))
    );
}

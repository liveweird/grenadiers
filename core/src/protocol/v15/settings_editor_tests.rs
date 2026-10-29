use crate::{
    game::{EnabledItems, PlayerCount, StormCloudConfig},
    room::BoardSize,
};

use super::{
    ConnectionStatus, ProjectionError, Seat, projection,
    server_conformance_lifecycle::{lifecycle, room_view},
};

fn waiting_room() -> super::RoomView {
    let lifecycle = lifecycle(
        PlayerCount::Two,
        BoardSize::ElevenByNine,
        EnabledItems::EMPTY,
        false,
    );
    room_view(&lifecycle, StormCloudConfig::Disabled, false)
}

#[test]
fn settings_editor_is_an_optional_eligible_creator_grant() {
    let mut room = waiting_room();
    assert_eq!(room.settings_editor, Some(Seat::One));
    room.settings_editor = None;
    assert!(projection::validate_room(&room).is_ok());

    for editor in [Seat::Two, Seat::Three, Seat::Four] {
        let mut invalid = waiting_room();
        invalid.settings_editor = Some(editor);
        assert_eq!(
            projection::validate_room(&invalid),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }

    for connection in [ConnectionStatus::Vacant, ConnectionStatus::Expired] {
        let mut invalid = waiting_room();
        invalid.seats[0].occupied = false;
        invalid.seats[0].connection = connection;
        assert_eq!(
            projection::validate_room(&invalid),
            Err(ProjectionError::InvalidExpandedBinding)
        );
    }
}

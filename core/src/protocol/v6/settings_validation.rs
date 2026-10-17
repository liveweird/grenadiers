use super::super::{RulesVersion, Seat};
use super::{ConnectionStatus, GameSettings, RoomProjectionError, RoomView};

pub(super) fn validate_settings(room: &RoomView) -> Result<(), RoomProjectionError> {
    let target = room.settings.domain().wins_to_finish();
    if room.scores.one.get() > target || room.scores.two.get() > target {
        return Err(RoomProjectionError::InvalidSettings);
    }
    if room.rules_version != RulesVersion::ConfigurableV4
        && room.settings != GameSettings::for_rules(room.rules_version)
    {
        return Err(RoomProjectionError::InvalidSettings);
    }
    if let Some(editor) = room.settings_editor {
        let index = match editor {
            Seat::One => 0,
            Seat::Two => 1,
        };
        if !room.seats[index].occupied
            || matches!(
                room.seats[index].connection,
                ConnectionStatus::Vacant | ConnectionStatus::Expired
            )
        {
            return Err(RoomProjectionError::InvalidSettings);
        }
    }
    Ok(())
}

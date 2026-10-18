use crate::protocol::Seat;

use super::{ConnectionStatus, GameSettings, ProjectionError, RoomView, RulesVersion};

pub(super) fn validate_settings(room: &RoomView) -> Result<(), ProjectionError> {
    super::expanded_settings(room.settings)?;
    let target = room.settings.wins_to_finish();
    if room.scores.one.get() > target || room.scores.two.get() > target {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    if matches!(
        room.rules_version,
        RulesVersion::ClassicV1 | RulesVersion::ClassicV2 | RulesVersion::ClassicV3
    ) && room.settings != GameSettings::for_rules(room.rules_version)
    {
        return Err(ProjectionError::InvalidExpandedBinding);
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
            return Err(ProjectionError::InvalidExpandedBinding);
        }
    }
    Ok(())
}

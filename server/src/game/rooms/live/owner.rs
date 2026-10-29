use grenadiers_core::protocol::{Counter, RoomId, v14::ResumeToken};

#[cfg(test)]
use super::super::{match_owner::MatchOwnerHandle, match_runtime::MatchAction};

use super::{
    super::{BindingKey, RoomError, roster_owner::RosterOwnerHandle, roster_runtime::RosterAction},
    LiveAdmission,
};

#[derive(Clone)]
pub(in crate::game::rooms) enum LiveOwner {
    #[cfg(test)]
    Legacy(MatchOwnerHandle),
    Roster(RosterOwnerHandle),
}

impl LiveOwner {
    #[cfg(test)]
    pub(in crate::game::rooms) fn legacy(&self) -> Option<&MatchOwnerHandle> {
        match self {
            Self::Legacy(value) => Some(value),
            Self::Roster(_) => None,
        }
    }

    pub(in crate::game::rooms) async fn admit(
        &self,
        creator: bool,
        binding: BindingKey,
        sequence: Counter,
    ) -> Result<LiveAdmission, RoomError> {
        match self {
            #[cfg(test)]
            Self::Legacy(value) => {
                let response = if creator {
                    value.admit_creator(binding, sequence)
                } else {
                    value.join_sequenced(binding, sequence)
                };
                response
                    .await
                    .unwrap_or(Err(RoomError::RoomNotFound))
                    .map(LiveAdmission::Legacy)
            }
            Self::Roster(value) => {
                let response = if creator {
                    value.admit_creator(binding, sequence)
                } else {
                    value.join_sequenced(binding, sequence)
                };
                response
                    .await
                    .unwrap_or(Err(RoomError::RoomNotFound))
                    .map(LiveAdmission::Roster)
            }
        }
    }

    pub(in crate::game::rooms) async fn resume(
        &self,
        binding: BindingKey,
        sequence: Counter,
        room_id: RoomId,
        token: ResumeToken,
    ) -> Result<LiveAdmission, RoomError> {
        match self {
            #[cfg(test)]
            Self::Legacy(value) => value
                .resume(binding, sequence, room_id, token)
                .await
                .unwrap_or(Err(RoomError::ResumeRejected))
                .map(LiveAdmission::Legacy),
            Self::Roster(value) => value
                .resume(binding, sequence, room_id, token)
                .await
                .unwrap_or(Err(RoomError::ResumeRejected))
                .map(LiveAdmission::Roster),
        }
    }

    pub(in crate::game::rooms) async fn action(
        &self,
        binding: BindingKey,
        room_id: RoomId,
        round_id: u64,
        action: RosterAction,
    ) -> Result<(), RoomError> {
        match self {
            #[cfg(test)]
            Self::Legacy(value) => value
                .action(binding, room_id, round_id, legacy_action(action))
                .await
                .unwrap_or(Err(RoomError::RoomNotFound)),
            Self::Roster(value) => value
                .action(binding, room_id, round_id, action)
                .await
                .unwrap_or(Err(RoomError::RoomNotFound)),
        }
    }

    pub(in crate::game::rooms) async fn environment_settings(
        &self,
        binding: BindingKey,
        room_id: RoomId,
        round_id: u64,
        action: super::super::live_registry::EnvironmentSettingsAction,
    ) -> Result<(), RoomError> {
        match self {
            #[cfg(test)]
            Self::Legacy(_) => Err(RoomError::UnsupportedConfiguration),
            Self::Roster(value) => value
                .environment_settings(binding, room_id, round_id, action)
                .await
                .unwrap_or(Err(RoomError::RoomNotFound)),
        }
    }

    pub(in crate::game::rooms) async fn leave(
        &self,
        binding: BindingKey,
        room_id: RoomId,
        round_id: u64,
    ) -> Result<(), RoomError> {
        match self {
            #[cfg(test)]
            Self::Legacy(value) => value
                .leave(binding, room_id, round_id)
                .await
                .unwrap_or(Err(RoomError::RoomNotFound)),
            Self::Roster(value) => value
                .leave(binding, room_id, round_id)
                .await
                .unwrap_or(Err(RoomError::RoomNotFound)),
        }
    }
}

#[cfg(test)]
fn legacy_action(action: RosterAction) -> MatchAction {
    match action {
        RosterAction::Ready {
            ready,
            settings_revision,
        } => MatchAction::ReadyChecked {
            ready,
            settings_revision,
        },
        RosterAction::UpdateSettings {
            settings_revision,
            settings,
        } => MatchAction::UpdateSettings {
            settings_revision,
            settings,
        },
        RosterAction::Movement(direction) => MatchAction::Movement(direction),
        RosterAction::Bomb(id) => MatchAction::Bomb(id),
        RosterAction::Equipment(id) => MatchAction::Equipment(id),
        RosterAction::Rematch(ready) => MatchAction::Rematch(ready),
    }
}

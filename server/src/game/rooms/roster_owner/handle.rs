use grenadiers_core::protocol::{Counter, RoomId, v14::ResumeToken};
use tokio::sync::oneshot;

use super::super::{
    roster_runtime::RosterAction,
    types::{BindingKey, RoomError},
};
use super::types::{OwnerCommand, Reply, RosterAdmission, RosterOwnerHandle};

impl RosterOwnerHandle {
    pub(in crate::game::rooms) fn join_sequenced(
        &self,
        binding: BindingKey,
        sequence: Counter,
    ) -> Reply<RosterAdmission> {
        self.submit(|reply| OwnerCommand::Join {
            creator: false,
            binding,
            sequence,
            reply,
        })
    }

    pub(in crate::game::rooms) fn admit_creator(
        &self,
        binding: BindingKey,
        sequence: Counter,
    ) -> Reply<RosterAdmission> {
        self.submit(|reply| OwnerCommand::Join {
            creator: true,
            binding,
            sequence,
            reply,
        })
    }

    pub(in crate::game::rooms) fn resume(
        &self,
        binding: BindingKey,
        sequence: Counter,
        room_id: RoomId,
        token: ResumeToken,
    ) -> Reply<RosterAdmission> {
        self.submit(|reply| OwnerCommand::Resume {
            binding,
            sequence,
            room_id,
            token,
            reply,
        })
    }

    pub(in crate::game::rooms) fn action(
        &self,
        binding: BindingKey,
        room_id: RoomId,
        round_id: u64,
        action: RosterAction,
    ) -> Reply<()> {
        self.submit(|reply| OwnerCommand::Action {
            binding,
            room_id,
            round_id,
            action,
            reply,
        })
    }

    pub(in crate::game::rooms) fn environment_settings(
        &self,
        binding: BindingKey,
        room_id: RoomId,
        round_id: u64,
        action: super::super::live_registry::EnvironmentSettingsAction,
    ) -> Reply<()> {
        self.submit(|reply| OwnerCommand::EnvironmentSettings {
            binding,
            room_id,
            round_id,
            action,
            reply,
        })
    }

    pub(in crate::game::rooms) fn leave(
        &self,
        binding: BindingKey,
        room_id: RoomId,
        round_id: u64,
    ) -> Reply<()> {
        self.submit(|reply| OwnerCommand::Leave {
            binding,
            room_id,
            round_id,
            reply,
        })
    }

    fn submit<T>(
        &self,
        command: impl FnOnce(oneshot::Sender<Result<T, RoomError>>) -> OwnerCommand,
    ) -> Reply<T> {
        let (reply, response) = oneshot::channel();
        match self.commands.try_send(command(reply)) {
            Ok(()) => {}
            Err(tokio::sync::mpsc::error::TrySendError::Full(command)) => {
                command.reject(RoomError::RateLimited);
            }
            Err(tokio::sync::mpsc::error::TrySendError::Closed(command)) => {
                command.reject_unavailable();
            }
        }
        response
    }
}

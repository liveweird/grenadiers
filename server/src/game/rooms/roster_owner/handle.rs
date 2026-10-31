use grenadiers_core::protocol::{Counter, RoomId, v14::ResumeToken};
use tokio::sync::oneshot;

use super::super::{
    roster_runtime::RosterAction,
    types::{BindingKey, RoomError},
};
use super::types::{CpuDebugCommand, OwnerCommand, Reply, RosterAdmission, RosterOwnerHandle};

impl RosterOwnerHandle {
    pub(in crate::game::rooms) fn cpu_debug(
        &self,
        token: ResumeToken,
    ) -> oneshot::Receiver<
        Result<crate::cpu_debug::CpuDebugResponse, crate::cpu_debug::CpuDebugQueryError>,
    > {
        let (reply, response) = oneshot::channel();
        match self
            .cpu_debug_commands
            .try_send(CpuDebugCommand { token, reply })
        {
            Ok(()) => {}
            Err(tokio::sync::mpsc::error::TrySendError::Full(command))
            | Err(tokio::sync::mpsc::error::TrySendError::Closed(command)) => command.reject(),
        }
        response
    }

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

    pub(in crate::game::rooms) fn lair_settings(
        &self,
        binding: BindingKey,
        room_id: RoomId,
        round_id: u64,
        action: super::super::live_registry::LairSettingsAction,
    ) -> Reply<()> {
        self.submit(|reply| OwnerCommand::LairSettings {
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

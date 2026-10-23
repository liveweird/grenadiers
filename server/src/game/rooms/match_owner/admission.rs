use grenadiers_core::protocol::{Counter, RoomId, v10::ResumeToken};
use tokio::sync::oneshot;

use super::super::{
    match_runtime::MatchAction,
    types::{BindingKey, RoomError},
};
use super::types::{MatchAdmission, MatchOwnerHandle, OwnerCommand, Reply};

impl MatchOwnerHandle {
    pub(in crate::game::rooms) fn join_sequenced(
        &self,
        binding: BindingKey,
        sequence: Counter,
    ) -> Reply<MatchAdmission> {
        self.submit(|reply| OwnerCommand::Join {
            binding,
            sequence,
            reply,
        })
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn join(&self, binding: BindingKey) -> Reply<MatchAdmission> {
        self.join_sequenced(binding, Counter::new(1).expect("test sequence"))
    }

    pub(in crate::game::rooms) fn resume(
        &self,
        binding: BindingKey,
        sequence: Counter,
        room_id: RoomId,
        token: ResumeToken,
    ) -> Reply<MatchAdmission> {
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
        action: MatchAction,
    ) -> Reply<()> {
        self.submit(|reply| OwnerCommand::Action {
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
        let command = command(reply);
        if let Err(error) = self.commands.try_send(command) {
            match error {
                tokio::sync::mpsc::error::TrySendError::Full(command) => {
                    command.reject(RoomError::RateLimited);
                }
                tokio::sync::mpsc::error::TrySendError::Closed(command) => {
                    command.reject_unavailable();
                }
            }
        }
        response
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn submit_canceled_join(&self, binding: BindingKey) {
        let (reply, response) = oneshot::channel();
        drop(response);
        let _ = self.commands.try_send(OwnerCommand::Join {
            binding,
            sequence: Counter::new(1).expect("test sequence"),
            reply,
        });
    }
}

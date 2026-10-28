use grenadiers_core::{
    game::SeatId,
    room::{DepartureReason, InterruptReason},
};

use super::{MatchNotice, MatchRuntime, MatchRuntimeError, map_lifecycle_error};

impl MatchRuntime {
    pub(in crate::game::rooms) const fn mode(&self) -> grenadiers_core::room::RoomMode {
        self.identity.mode
    }

    pub(in crate::game::rooms) const fn cpu_seat(&self) -> Option<SeatId> {
        super::cpu_seat_for_mode(self.identity.mode)
    }

    pub(in crate::game::rooms) fn cpu_ready(
        &mut self,
        cpu_seat: SeatId,
    ) -> Result<Option<MatchNotice>, MatchRuntimeError> {
        self.require_cpu_seat(cpu_seat)?;
        let epoch = self.lifecycle.round_attempt();
        self.action(
            cpu_seat,
            epoch,
            super::super::MatchAction::ReadyChecked {
                ready: true,
                settings_revision: self.settings_revision,
            },
        )
    }

    pub(in crate::game::rooms) fn cpu_rematch(
        &mut self,
        cpu_seat: SeatId,
    ) -> Result<Option<MatchNotice>, MatchRuntimeError> {
        self.require_cpu_seat(cpu_seat)?;
        self.action(
            cpu_seat,
            self.lifecycle.round_attempt(),
            super::super::MatchAction::Rematch(true),
        )
    }

    pub(in crate::game::rooms) fn next_cpu_manual_action(&self, cpu_seat: SeatId) -> Option<u32> {
        if self.require_cpu_seat(cpu_seat).is_err() {
            return None;
        }
        self.inputs
            .last_manual_action(cpu_seat)
            .unwrap_or(0)
            .checked_add(1)
            .filter(|value| grenadiers_core::protocol::Counter::new(*value).is_ok())
    }

    pub(in crate::game::rooms) fn submit_cpu_intent(
        &mut self,
        cpu_seat: SeatId,
        expected_epoch: u64,
        action: super::super::MatchAction,
    ) -> Result<Option<MatchNotice>, MatchRuntimeError> {
        self.require_cpu_seat(cpu_seat)?;
        if !matches!(
            action,
            super::super::MatchAction::Movement(_)
                | super::super::MatchAction::Bomb(_)
                | super::super::MatchAction::Equipment(_)
        ) {
            return Err(MatchRuntimeError::InvalidPhase);
        }
        self.action(cpu_seat, expected_epoch, action)
    }

    fn require_cpu_seat(&self, cpu_seat: SeatId) -> Result<(), MatchRuntimeError> {
        if self.cpu_seat() == Some(cpu_seat) {
            Ok(())
        } else {
            Err(MatchRuntimeError::InvalidPhase)
        }
    }

    pub(in crate::game::rooms) const fn lifecycle(&self) -> &grenadiers_core::room::MatchLifecycle {
        &self.lifecycle
    }

    pub(in crate::game::rooms) fn initial_notice(
        &mut self,
    ) -> Result<MatchNotice, MatchRuntimeError> {
        let next = self.next_revision()?;
        let room = super::super::publication::room_view(
            &self.identity,
            &self.lifecycle,
            next,
            self.settings_revision,
            None,
        )?;
        let notice = self.view_notice(room)?;
        self.revision = next;
        Ok(notice)
    }

    pub(in crate::game::rooms) fn current_notice(
        &mut self,
    ) -> Result<MatchNotice, MatchRuntimeError> {
        self.publish_current()
    }

    pub(in crate::game::rooms) fn stop_seat_inputs(&mut self, seat: SeatId) {
        self.inputs.stop(seat);
        self.capture(
            self.lifecycle.tick(),
            crate::game::rooms::replay::Operation::InputFence(seat),
        );
    }

    pub(in crate::game::rooms) fn interrupt_and_depart(
        &mut self,
        reason: InterruptReason,
        departures: [Option<DepartureReason>; 2],
    ) -> Result<MatchNotice, MatchRuntimeError> {
        let tick = self.lifecycle.tick();
        self.ensure_revision()?;
        self.lifecycle
            .interrupt(reason)
            .map_err(map_lifecycle_error)?;
        self.inputs.clear_pending();
        self.lifecycle
            .depart_batch(departures)
            .map_err(map_lifecycle_error)?;
        self.release_departed_editor(departures);
        for (seat, departure) in SeatId::DUEL.into_iter().zip(departures) {
            if departure.is_some() {
                self.inputs.stop(seat);
            }
        }
        let notice = self.publish_current()?;
        self.capture(
            tick,
            crate::game::rooms::replay::Operation::InterruptAndDepart { reason, departures },
        );
        Ok(notice)
    }
    #[cfg(test)]
    pub(in crate::game::rooms) fn depart_batch(
        &mut self,
        departures: [Option<DepartureReason>; 2],
    ) -> Result<Option<MatchNotice>, MatchRuntimeError> {
        let tick = self.lifecycle.tick();
        self.ensure_revision()?;
        let change = self
            .lifecycle
            .depart_batch(departures)
            .map_err(map_lifecycle_error)?;
        self.release_departed_editor(departures);
        for (seat, departure) in SeatId::DUEL.into_iter().zip(departures) {
            if departure.is_some() {
                self.inputs.stop(seat);
            }
        }
        let notice = self.notice_for_change(change, true)?;
        self.capture(
            tick,
            crate::game::rooms::replay::Operation::Depart(departures),
        );
        Ok(notice)
    }

    pub(in crate::game::rooms) fn interrupt(
        &mut self,
        reason: InterruptReason,
    ) -> Result<Option<MatchNotice>, MatchRuntimeError> {
        let tick = self.lifecycle.tick();
        self.ensure_revision()?;
        let change = self
            .lifecycle
            .interrupt(reason)
            .map_err(map_lifecycle_error)?;
        self.inputs.clear_pending();
        let notice = self.notice_for_change(change, true)?;
        self.capture(
            tick,
            crate::game::rooms::replay::Operation::Interrupt(reason),
        );
        Ok(notice)
    }
}

#[cfg(test)]
mod tests {
    use grenadiers_core::{
        game::{Direction, InputAction, SeatId},
        room::{CpuProfile, MatchPhase, RoomMode},
    };

    use super::*;
    use crate::{
        game::rooms::{match_runtime::test_support::identity, profile::RoomProfile},
        round_seed::RoundSeedSource,
    };

    #[test]
    fn cpu_intent_authority_rejects_human_seat_and_queues_configured_cpu_seat() {
        let mut identity = identity(RoomProfile::ExpandedV5);
        identity.mode = RoomMode::Solo(CpuProfile::CURRENT);
        identity.private_cpu_seed = Some([9; 32]);
        let mut runtime = MatchRuntime::new(
            identity,
            RoundSeedSource::with_entropy(|bytes| {
                bytes.fill(4);
                Ok(())
            }),
        );
        let creator = runtime.prepare_solo_creator(SeatId::One).unwrap();
        runtime.commit_admission(creator);
        runtime
            .action(
                SeatId::One,
                runtime.lifecycle().round_attempt(),
                super::super::super::MatchAction::ReadyChecked {
                    ready: true,
                    settings_revision: 1,
                },
            )
            .unwrap();
        while runtime.lifecycle().phase() != MatchPhase::Running {
            runtime.settle_boundary([None; 2]).unwrap();
        }
        let epoch = runtime.lifecycle().round_attempt();

        assert_eq!(
            runtime.submit_cpu_intent(
                SeatId::One,
                epoch,
                super::super::super::MatchAction::Movement(Some(Direction::East)),
            ),
            Err(MatchRuntimeError::InvalidPhase)
        );
        assert!(runtime.inputs.take(runtime.lifecycle().tick()).is_empty());

        runtime
            .submit_cpu_intent(
                SeatId::Two,
                epoch,
                super::super::super::MatchAction::Movement(Some(Direction::East)),
            )
            .unwrap();
        let queued = runtime.inputs.take(runtime.lifecycle().tick());
        assert!(queued.iter().any(|input| {
            input.seat == SeatId::Two && input.action == InputAction::Press(Direction::East)
        }));
        assert!(!queued.iter().any(|input| input.seat == SeatId::One));
    }
}

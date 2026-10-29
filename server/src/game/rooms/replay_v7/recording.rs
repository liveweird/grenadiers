use grenadiers_core::{
    game::SeatId,
    room::{DepartureReason, InterruptReason, LairRoomConfiguration, MatchPhase, RoomSettings},
};

use crate::{
    game::rooms::{replay_v4::runner::ReplayError, roster_runtime::RosterRuntime},
    round_seed::RoundSeedSource,
};

use super::{
    record::{
        ActionSource, CpuProvenance, Journal, Operation, ReplayAction, ReplayHeader, ReplayStatus,
    },
    runner::{domain_action, runtime_from_header},
};

pub(in crate::game::rooms) struct RecordingRosterRuntime {
    runtime: RosterRuntime,
    journal: Journal,
}

impl RecordingRosterRuntime {
    pub(in crate::game::rooms) fn new(
        configuration: LairRoomConfiguration,
        settings: RoomSettings,
        generation_seeds: RoundSeedSource,
        cpus: Vec<CpuProvenance>,
    ) -> Result<Self, ReplayError> {
        let header = ReplayHeader::new(configuration, settings, [false; 4], cpus)
            .map_err(|()| ReplayError::InvalidHeader)?;
        let runtime = runtime_from_header(&header, generation_seeds)?;
        Ok(Self {
            runtime,
            journal: Journal::new(header),
        })
    }

    pub(in crate::game::rooms) const fn runtime(&self) -> &RosterRuntime {
        &self.runtime
    }
    pub(in crate::game::rooms) const fn journal(&self) -> &Journal {
        &self.journal
    }
    pub(in crate::game::rooms) fn into_parts(self) -> (RosterRuntime, Journal) {
        (self.runtime, self.journal)
    }

    pub(in crate::game::rooms) fn admit(
        &mut self,
        seat: SeatId,
        resume: bool,
    ) -> Result<(), ReplayError> {
        let prepared = if resume {
            self.runtime.prepare_resume(seat)
        } else {
            self.runtime.prepare_join(seat)
        }
        .map_err(|_| ReplayError::Runtime)?;
        self.runtime.commit_admission(prepared);
        self.record(Operation::Admit { seat, resume });
        Ok(())
    }

    pub(in crate::game::rooms) fn human_action(
        &mut self,
        seat: SeatId,
        epoch: u64,
        action: ReplayAction,
    ) -> Result<(), ReplayError> {
        match action {
            ReplayAction::UpdateSettings {
                settings_revision,
                settings,
                cloud,
                tuning,
            } => self.runtime.update_lair_settings(
                seat,
                epoch,
                settings_revision,
                settings,
                cloud,
                tuning,
            ),
            action => self.runtime.action(seat, epoch, domain_action(action)?),
        }
        .map_err(|_| ReplayError::Runtime)?;
        self.record(Operation::Action {
            seat,
            epoch,
            source: ActionSource::Human,
            action,
        });
        Ok(())
    }

    pub(in crate::game::rooms) fn cpu_action(
        &mut self,
        seat: SeatId,
        epoch: u64,
        action: ReplayAction,
    ) -> Result<(), ReplayError> {
        self.runtime
            .submit_cpu_intent(seat, epoch, domain_action(action)?)
            .map_err(|_| ReplayError::Runtime)?;
        self.record(Operation::Action {
            seat,
            epoch,
            source: ActionSource::Cpu,
            action,
        });
        Ok(())
    }

    pub(in crate::game::rooms) fn input_fence(&mut self, seat: SeatId) {
        self.runtime.stop_seat_inputs(seat);
        self.record(Operation::InputFence(seat));
    }

    pub(in crate::game::rooms) fn settle(
        &mut self,
        departures: [Option<DepartureReason>; 4],
    ) -> Result<(), ReplayError> {
        let tick = self.runtime.replay_inspection().lifecycle.tick();
        let before = self
            .runtime
            .replay_inspection()
            .simulation
            .map(|sim| sim.epoch);
        if self.runtime.settle_boundary(departures).is_err() {
            if matches!(
                self.runtime.replay_inspection().lifecycle.phase(),
                MatchPhase::Countdown { .. }
            ) {
                self.journal.status = ReplayStatus::IncompleteRoundConstruction;
            }
            return Err(ReplayError::Runtime);
        }
        self.journal.record(tick, Operation::Settle(departures));
        let inspection = self.runtime.replay_inspection();
        if let Some(sim) = inspection
            .simulation
            .filter(|sim| Some(sim.epoch) != before)
        {
            self.journal.record(
                inspection.lifecycle.tick(),
                Operation::RoundStarted {
                    epoch: sim.epoch,
                    assignment: sim.state.round_assignment(),
                    generation: sim.generation,
                },
            );
        }
        Ok(())
    }

    pub(in crate::game::rooms) fn publish_flush(&mut self) -> Result<(), ReplayError> {
        self.runtime
            .current_notice()
            .map_err(|_| ReplayError::Runtime)?;
        self.record(Operation::PublishFlush);
        Ok(())
    }

    pub(in crate::game::rooms) fn interrupt(
        &mut self,
        reason: InterruptReason,
    ) -> Result<(), ReplayError> {
        self.runtime
            .interrupt(reason)
            .map_err(|_| ReplayError::Runtime)?;
        self.record(Operation::Interrupt(reason));
        Ok(())
    }

    fn record(&mut self, operation: Operation) {
        self.journal
            .record(self.runtime.replay_inspection().lifecycle.tick(), operation);
    }
}

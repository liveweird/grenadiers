use grenadiers_core::{
    game::SeatId,
    room::{CpuDifficulty, DepartureReason, InterruptReason, RoomSettings, SpikeRoomConfiguration},
};

use super::{Journal, ReplayAction, ReplayHeader};
use crate::{
    game::rooms::{replay_v4::runner::ReplayError, replay_v8, roster_runtime::RosterRuntime},
    round_seed::RoundSeedSource,
};

pub(in crate::game::rooms) struct RecordingRosterRuntime {
    body: replay_v8::RecordingRosterRuntime,
    header: ReplayHeader,
}

impl RecordingRosterRuntime {
    pub(in crate::game::rooms) fn new(
        configuration: SpikeRoomConfiguration,
        settings: RoomSettings,
        generation_seeds: RoundSeedSource,
        cpus: Vec<(SeatId, CpuDifficulty, [u8; 32])>,
    ) -> Result<Self, ReplayError> {
        let policy = super::cpu_policy_version(&configuration);
        let cpus = cpus
            .into_iter()
            .map(|(seat, difficulty, seed)| {
                replay_v8::CpuProvenance::new_with_policy(seat, difficulty, seed, policy)
            })
            .collect::<Vec<_>>();
        let (header, journal) = ReplayHeader::new(configuration, settings, cpus)?;
        let runtime = super::runtime_from_header(&header, &journal, generation_seeds)?;
        Ok(Self {
            body: replay_v8::RecordingRosterRuntime::from_parts(runtime, journal),
            header,
        })
    }

    pub(in crate::game::rooms) const fn runtime(&self) -> &RosterRuntime {
        self.body.runtime()
    }
    pub(in crate::game::rooms) fn admit(
        &mut self,
        seat: SeatId,
        resume: bool,
    ) -> Result<(), ReplayError> {
        self.body.admit(seat, resume)
    }
    pub(in crate::game::rooms) fn human_action(
        &mut self,
        seat: SeatId,
        epoch: u64,
        action: ReplayAction,
    ) -> Result<(), ReplayError> {
        self.body.human_action(seat, epoch, action)
    }
    pub(in crate::game::rooms) fn settle(
        &mut self,
        departures: [Option<DepartureReason>; 4],
    ) -> Result<(), ReplayError> {
        self.body.settle(departures)
    }
    pub(in crate::game::rooms) fn into_parts(self) -> (RosterRuntime, Journal) {
        let (runtime, body) = self.body.into_parts();
        (
            runtime,
            Journal {
                header: self.header,
                body,
            },
        )
    }

    pub(in crate::game::rooms) fn cpu_action(
        &mut self,
        seat: SeatId,
        epoch: u64,
        action: ReplayAction,
    ) -> Result<(), ReplayError> {
        self.body.cpu_action(seat, epoch, action)
    }
    pub(in crate::game::rooms) fn input_fence(&mut self, seat: SeatId) {
        self.body.input_fence(seat);
    }
    pub(in crate::game::rooms) fn publish_flush(&mut self) -> Result<(), ReplayError> {
        self.body.publish_flush()
    }
    pub(in crate::game::rooms) fn interrupt(
        &mut self,
        reason: InterruptReason,
    ) -> Result<(), ReplayError> {
        self.body.interrupt(reason)
    }
    pub(in crate::game::rooms) fn interrupt_and_depart(
        &mut self,
        reason: InterruptReason,
        departures: [Option<DepartureReason>; 4],
    ) -> Result<(), ReplayError> {
        self.body.interrupt_and_depart(reason, departures)
    }
}

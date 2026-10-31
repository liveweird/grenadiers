use grenadiers_core::{
    game::{MobNavigationPolicy, SeatId},
    room::{DepartureReason, LairRoomConfiguration, RoomSettings},
};

use super::{CpuProvenance, Journal, ReplayAction};
use crate::{
    game::rooms::{replay_v4::runner::ReplayError, replay_v8, roster_runtime::RosterRuntime},
    round_seed::RoundSeedSource,
};

pub(in crate::game::rooms) struct RecordingRosterRuntime {
    body: replay_v8::RecordingRosterRuntime,
}

impl RecordingRosterRuntime {
    pub(in crate::game::rooms) fn new(
        configuration: LairRoomConfiguration,
        settings: RoomSettings,
        generation_seeds: RoundSeedSource,
        cpus: Vec<CpuProvenance>,
    ) -> Result<Self, ReplayError> {
        let body = replay_v8::RecordingRosterRuntime::new_with_mob_navigation(
            configuration,
            settings,
            generation_seeds,
            cpus,
            MobNavigationPolicy::EscapeV2,
        )?;
        Ok(Self { body })
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

    pub(in crate::game::rooms) fn cpu_action(
        &mut self,
        seat: SeatId,
        epoch: u64,
        action: ReplayAction,
    ) -> Result<(), ReplayError> {
        self.body.cpu_action(seat, epoch, action)
    }

    pub(in crate::game::rooms) fn settle(
        &mut self,
        departures: [Option<DepartureReason>; 4],
    ) -> Result<(), ReplayError> {
        self.body.settle(departures)
    }

    pub(in crate::game::rooms) fn runtime(&self) -> &RosterRuntime {
        self.body.runtime()
    }

    pub(in crate::game::rooms) fn into_parts(self) -> (RosterRuntime, Journal) {
        let (runtime, body) = self.body.into_parts();
        (runtime, Journal::new(body))
    }
}

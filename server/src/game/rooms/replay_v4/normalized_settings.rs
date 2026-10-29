use grenadiers_core::{
    game::{SeatId, StormCloudConfig},
    room::{LairTuning, RoomSettings},
};

use crate::game::rooms::roster_runtime::RosterRuntime;

use super::runner::ReplayError;

#[derive(Clone, Copy)]
pub(in crate::game::rooms) enum NormalizedSettings {
    StormV7 {
        seat: SeatId,
        epoch: u64,
        revision: u32,
        settings: RoomSettings,
        cloud: StormCloudConfig,
    },
    LairsV8 {
        seat: SeatId,
        epoch: u64,
        revision: u32,
        settings: RoomSettings,
        cloud: StormCloudConfig,
        tuning: LairTuning,
    },
}

impl NormalizedSettings {
    pub(super) fn apply(self, runtime: &mut RosterRuntime) -> Result<(), ReplayError> {
        let result = match self {
            Self::StormV7 {
                seat,
                epoch,
                revision,
                settings,
                cloud,
            } => runtime.update_environment_settings(seat, epoch, revision, settings, cloud),
            Self::LairsV8 {
                seat,
                epoch,
                revision,
                settings,
                cloud,
                tuning,
            } => runtime.update_lair_settings(seat, epoch, revision, settings, cloud, tuning),
        };
        result.map(|_| ()).map_err(|_| ReplayError::Runtime)
    }
}

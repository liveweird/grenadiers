use grenadiers_core::{
    game::SeatId,
    room::{CpuDifficulty, RoomSettings, SpikeRoomConfiguration},
};

use super::{RecordingRosterRuntime, ReplayHeader};
use crate::{
    game::rooms::{replay_v4::runner::ReplayError, replay_v8, roster_seed::SimulationRoot},
    round_seed::RoundSeedSource,
};

impl RecordingRosterRuntime {
    pub(in crate::game::rooms) fn new_for_evaluation(
        configuration: SpikeRoomConfiguration,
        settings: RoomSettings,
        generation_seeds: RoundSeedSource,
        simulation_root_seed: [u8; 32],
        cpus: Vec<(SeatId, CpuDifficulty, [u8; 32])>,
    ) -> Result<Self, ReplayError> {
        let policy = super::cpu_policy_version(&configuration);
        let cpus = cpus
            .into_iter()
            .map(|(seat, difficulty, seed)| {
                replay_v8::CpuProvenance::new_with_policy(seat, difficulty, seed, policy)
            })
            .collect::<Vec<_>>();
        let (header, mut journal) = ReplayHeader::new(configuration, settings, cpus)?;
        journal.header.simulation_root = SimulationRoot::from_seed_for_test(simulation_root_seed);
        super::validation::validate_header(&header, &journal)
            .map_err(|()| ReplayError::InvalidHeader)?;
        let runtime = super::runtime_from_header(&header, &journal, generation_seeds)?;
        Ok(Self {
            body: replay_v8::RecordingRosterRuntime::from_parts(runtime, journal),
            header,
        })
    }
}

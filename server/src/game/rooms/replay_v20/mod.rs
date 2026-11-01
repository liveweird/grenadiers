//! Private format20 envelope for the contact-topology CPU policy pairing.
//! Production recording and export intentionally remain disabled.
mod recording;
#[cfg(test)]
mod tests;
mod validation;

use grenadiers_core::{
    game::{MobNavigationPolicy, SPIKE_PIT_LAYOUT_VERSION},
    room::{ROSTER_LIFECYCLE_PROFILE, RoomSettings, SpikeRoomConfiguration},
};

use super::{
    replay_v4::runner::ReplayError,
    replay_v8,
    roster_runtime::{RosterIdentity, RosterRuntime, RosterRuntimeInspection},
};
use crate::{game::rooms::roster_seed::SimulationRoot, round_seed::RoundSeedSource};

pub(in crate::game::rooms) use recording::RecordingRosterRuntime;
pub(in crate::game::rooms) use replay_v8::ReplayAction;

const FORMAT_VERSION: u16 = 20;
const RULES_VERSION: u16 = 10;
const GAMEPLAY_VERSION: u16 = 19;
const MOB_NAVIGATION_VERSION: u16 = 3;

#[derive(Clone, Debug)]
pub(in crate::game::rooms) struct ReplayHeader {
    format_version: u16,
    lifecycle_profile: u16,
    rules_version: u16,
    cpu_policy_version: u16,
    gameplay_version: u16,
    generator_version: u16,
    simulation_randomness_version: u16,
    simulation_seed_derivation_version: u16,
    environment_randomness_version: u16,
    lair_layout_version: u16,
    spike_pit_layout_version: u16,
    mob_navigation_version: u16,
    configuration: SpikeRoomConfiguration,
    initial_settings: RoomSettings,
    initial_settings_revision: u32,
    initially_occupied_humans: [bool; 4],
    cpus: Box<[replay_v8::CpuProvenance]>,
}

impl ReplayHeader {
    fn new(
        configuration: SpikeRoomConfiguration,
        settings: RoomSettings,
        cpus: Vec<replay_v8::CpuProvenance>,
    ) -> Result<(Self, replay_v8::Journal), ReplayError> {
        let generator_version = if configuration.base().base().board()
            == grenadiers_core::room::BoardSize::FifteenByThirteen
        {
            grenadiers_core::game::MULTIPLAYER_GENERATOR_VERSION
        } else {
            grenadiers_core::game::VARIABLE_GENERATOR_VERSION
        };
        let mut body = replay_v8::Journal::new_unvalidated(
            configuration.base().clone(),
            settings,
            [false; 4],
            cpus.clone(),
        )
        .map_err(|()| ReplayError::InvalidHeader)?;
        let policy = cpu_policy_version(&configuration);
        body.header.cpu_policy_version = policy;
        let header = Self {
            format_version: FORMAT_VERSION,
            lifecycle_profile: ROSTER_LIFECYCLE_PROFILE,
            rules_version: RULES_VERSION,
            cpu_policy_version: policy,
            gameplay_version: GAMEPLAY_VERSION,
            generator_version,
            simulation_randomness_version: replay_v8::record::SIMULATION_RANDOMNESS_VERSION,
            simulation_seed_derivation_version:
                replay_v8::record::SIMULATION_SEED_DERIVATION_VERSION,
            environment_randomness_version: replay_v8::record::ENVIRONMENT_RANDOMNESS_VERSION,
            lair_layout_version: replay_v8::record::LAIR_LAYOUT_VERSION,
            spike_pit_layout_version: SPIKE_PIT_LAYOUT_VERSION,
            mob_navigation_version: MOB_NAVIGATION_VERSION,
            configuration,
            initial_settings: settings,
            initial_settings_revision: 1,
            initially_occupied_humans: [false; 4],
            cpus: cpus.into_boxed_slice(),
        };
        validation::validate_header(&header, &body).map_err(|()| ReplayError::InvalidHeader)?;
        Ok((header, body))
    }
}

#[derive(Clone, Debug)]
pub(in crate::game::rooms) struct Journal {
    header: ReplayHeader,
    body: replay_v8::Journal,
}

pub(in crate::game::rooms) fn replay(
    journal: &Journal,
) -> Result<RosterRuntimeInspection, ReplayError> {
    validation::validate(journal).map_err(|()| ReplayError::InvalidHeader)?;
    match journal.body.status {
        replay_v8::ReplayStatus::Complete => {}
        replay_v8::ReplayStatus::Truncated => return Err(ReplayError::Truncated),
        replay_v8::ReplayStatus::IncompleteRoundConstruction => {
            return Err(ReplayError::Incomplete);
        }
    }
    let (source, seeds) = replay_v8::runner::generation_seed_context(&journal.body);
    let runtime = runtime_from_header(&journal.header, &journal.body, source)?;
    replay_v8::runner::execute_journal(runtime, &journal.body, seeds)
}

pub(in crate::game::rooms) fn verify_exact(
    journal: &Journal,
    expected: &RosterRuntimeInspection,
) -> Result<(), String> {
    let actual = replay(journal).map_err(|error| format!("replay failed: {error:?}"))?;
    (actual == *expected)
        .then_some(())
        .ok_or_else(|| "replay20 state mismatch".to_owned())
}

fn runtime_from_header(
    header: &ReplayHeader,
    body: &replay_v8::Journal,
    seeds: RoundSeedSource,
) -> Result<RosterRuntime, ReplayError> {
    let root: SimulationRoot = body.header.simulation_root.clone();
    let base = *header.configuration.base().base();
    let lairs =
        grenadiers_core::room::RespawnLairTuning::try_from(header.configuration.base().lairs())
            .map_err(|_| ReplayError::InvalidHeader)?;
    RosterRuntime::new_spike_v19_with_navigation(
        RosterIdentity {
            id: grenadiers_core::protocol::RoomId::new("00000000000000000000000000000020").unwrap(),
            code: grenadiers_core::protocol::RoomCode::new("REPLAYV4").unwrap(),
            registry_serial: 20,
            configuration: base,
        },
        header.initial_settings,
        header.configuration.base().storm_cloud(),
        lairs,
        header.configuration.spike_pits(),
        seeds,
        std::sync::Arc::new(move |epoch| root.for_round(epoch)),
        MobNavigationPolicy::TimedEscapeV3,
    )
    .map_err(|_| ReplayError::Runtime)
}

fn cpu_policy_version(configuration: &SpikeRoomConfiguration) -> u16 {
    use grenadiers_core::room::CpuDifficulty;
    match configuration.base().base().cpu_difficulty() {
        Some(CpuDifficulty::Hard) => 41,
        None | Some(CpuDifficulty::Easy | CpuDifficulty::Standard) => 40,
    }
}

//! Private format36 envelope for Hard CPU owned-bomb combinations.
//! Production recording and export intentionally remain disabled.
mod evaluation_fixture;
#[path = "../replay_v24/recording.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "Replay36 binds the shared recorder to its closed owned-combination identity"
)]
mod recording;
#[cfg(test)]
mod tests;
#[path = "../replay_v34/validation.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "Replay36 shares header grammar while binding its own closed policy identity"
)]
mod validation;

use grenadiers_core::{
    game::MobNavigationPolicy,
    room::{ROSTER_LIFECYCLE_PROFILE, RoomSettings, SpikeRoomConfiguration},
};

use super::{
    replay_v4::runner::ReplayError,
    replay_v8,
    roster_runtime::{
        RosterIdentity, RosterRuntime, RosterRuntimeInspection, SpikeRuntimeEnvironment,
    },
};
use crate::{game::rooms::roster_seed::SimulationRoot, round_seed::RoundSeedSource};

pub(in crate::game::rooms) use recording::RecordingRosterRuntime;
pub(in crate::game::rooms) use replay_v8::ReplayAction;

const FORMAT_VERSION: u16 = 36;
const RULES_VERSION: u16 = 10;
const GAMEPLAY_VERSION: u16 = 21;
const GENERATOR_VERSION: u16 = grenadiers_core::game::WEIGHTED_LAIR_CLEARANCE_GENERATOR_VERSION;
const SPIKE_PIT_LAYOUT_VERSION: u16 =
    grenadiers_core::game::LAIR_CLEARANCE_SPIKE_PIT_LAYOUT_VERSION;
const MOB_NAVIGATION_VERSION: u16 = 3;
const PUNCH_ACTIVATION_VERSION: u16 = 2;

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
    punch_activation_version: u16,
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
            generator_version: GENERATOR_VERSION,
            simulation_randomness_version: replay_v8::record::SIMULATION_RANDOMNESS_VERSION,
            simulation_seed_derivation_version:
                replay_v8::record::SIMULATION_SEED_DERIVATION_VERSION,
            environment_randomness_version: replay_v8::record::ENVIRONMENT_RANDOMNESS_VERSION,
            lair_layout_version: replay_v8::record::LAIR_LAYOUT_VERSION,
            spike_pit_layout_version: SPIKE_PIT_LAYOUT_VERSION,
            mob_navigation_version: MOB_NAVIGATION_VERSION,
            punch_activation_version: PUNCH_ACTIVATION_VERSION,
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
        .ok_or_else(|| "replay36 state mismatch".to_owned())
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
    RosterRuntime::new_spike_v21_combination_with_navigation(
        RosterIdentity {
            id: grenadiers_core::protocol::RoomId::new("00000000000000000000000000000036").unwrap(),
            code: grenadiers_core::protocol::RoomCode::new("REPLAYXY").unwrap(),
            registry_serial: 36,
            configuration: base,
        },
        header.initial_settings,
        SpikeRuntimeEnvironment {
            cloud: header.configuration.base().storm_cloud(),
            lairs,
            spike_pits: header.configuration.spike_pits(),
            navigation: MobNavigationPolicy::TimedEscapeV3,
        },
        seeds,
        std::sync::Arc::new(move |epoch| root.for_round(epoch)),
    )
    .map_err(|_| ReplayError::Runtime)
}

fn cpu_policy_version(configuration: &SpikeRoomConfiguration) -> u16 {
    use grenadiers_core::room::CpuDifficulty;
    match configuration.base().base().cpu_difficulty() {
        Some(CpuDifficulty::Hard) => 67,
        None | Some(CpuDifficulty::Easy | CpuDifficulty::Standard) => 62,
    }
}

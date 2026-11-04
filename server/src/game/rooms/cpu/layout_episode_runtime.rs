use std::sync::Arc;

use grenadiers_core::{
    game::{ArenaConstructionProfile, MobNavigationPolicy, SimulationSeed, StormCloudConfig},
    protocol::{RoomCode, RoomId},
    room::{
        CpuDifficulty, RespawnLairSettings, RespawnLairTuning, RoomConfiguration, RoomSettings,
        SeatControl, SpikePitTuning,
    },
};

use super::{LayoutCase, witness::TraceIdentity};
use crate::{
    game::rooms::roster_runtime::{RosterIdentity, RosterRuntime, SpikeRuntimeEnvironment},
    round_seed::RoundSeedSource,
};

pub(super) fn runtime(case: LayoutCase, identity: &TraceIdentity) -> RosterRuntime {
    let configuration = RoomConfiguration::new(
        &[
            SeatControl::Human,
            SeatControl::Cpu,
            SeatControl::Cpu,
            SeatControl::Cpu,
        ],
        Some(CpuDifficulty::Hard),
        case.board,
    )
    .unwrap();
    let roster_identity = RosterIdentity {
        id: RoomId::new(identity.id).unwrap(),
        code: RoomCode::new(identity.code).unwrap(),
        registry_serial: identity.instance,
        configuration,
    };
    let settings =
        RoomSettings::DEFAULT.with_enabled_items(grenadiers_core::game::EnabledItems::ALL);
    let environment = SpikeRuntimeEnvironment {
        cloud: StormCloudConfig::Single,
        lairs: RespawnLairTuning::new(case.lairs, RespawnLairSettings::DEFAULT),
        spike_pits: SpikePitTuning::new(true, case.pit_settings),
        navigation: MobNavigationPolicy::TimedEscapeV3,
    };
    let seeds = RoundSeedSource::with_entropy(move |bytes| {
        bytes.fill(case.seed);
        Ok(())
    });
    let simulation_seed = Arc::new(move |epoch| {
        Ok(SimulationSeed::new(
            [case.seed.wrapping_add(epoch as u8); 32],
        ))
    });
    match case.profile {
        ArenaConstructionProfile::ReflectedV3V4 => RosterRuntime::new_spike_v20_with_navigation(
            roster_identity,
            settings,
            environment.cloud,
            environment.lairs,
            environment.spike_pits,
            seeds,
            simulation_seed,
            environment.navigation,
        ),
        ArenaConstructionProfile::LairClearanceV5 => {
            RosterRuntime::new_spike_v20_clearance_with_navigation(
                roster_identity,
                settings,
                environment,
                seeds,
                simulation_seed,
            )
        }
        _ => panic!("unsupported diagnostic construction profile"),
    }
    .unwrap()
}

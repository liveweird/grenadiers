use std::sync::Arc;

use grenadiers_core::{
    game::{MobNavigationPolicy, SimulationSeed, StormCloudConfig},
    protocol::{RoomCode, RoomId},
    room::{
        BoardSize, CpuDifficulty, CpuProfile, RespawnLairTuning, RoomConfiguration, RoomSettings,
        SeatControl, SpikePitTuning,
    },
};

use super::validate_cpus;
use crate::{
    game::rooms::{
        cpu::{CpuController, CpuPlanner},
        roster_runtime::{RosterIdentity, RosterRuntime, RosterRuntimeError},
    },
    round_seed::RoundSeedSource,
};

#[test]
fn live_spike_owner_accepts_only_exact_cpu27_difficulty() {
    use grenadiers_core::game::SeatId;
    for (difficulty, accepted, archived, wrong_tier) in [
        (
            CpuDifficulty::Standard,
            CpuProfile::RELIABLE_V27_STANDARD,
            CpuProfile::SPIKE_PITS_STANDARD,
            CpuProfile::RELIABLE_V27_EASY,
        ),
        (
            CpuDifficulty::Easy,
            CpuProfile::RELIABLE_V27_EASY,
            CpuProfile::SPIKE_PITS_EASY,
            CpuProfile::RELIABLE_V27_STANDARD,
        ),
    ] {
        let configuration = RoomConfiguration::new(
            &[SeatControl::Human, SeatControl::Cpu],
            Some(difficulty),
            BoardSize::FifteenByThirteen,
        )
        .unwrap();
        let runtime = RosterRuntime::new_spike_with_navigation(
            RosterIdentity {
                id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
                code: RoomCode::new("ABCDEFGH").unwrap(),
                registry_serial: 127,
                configuration,
            },
            RoomSettings::DEFAULT,
            StormCloudConfig::Single,
            RespawnLairTuning::DEFAULT,
            SpikePitTuning::DEFAULT,
            RoundSeedSource::with_entropy(|bytes| {
                bytes.fill(27);
                Ok(())
            }),
            Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
            MobNavigationPolicy::TimedEscapeV3,
        )
        .unwrap();
        let controller = |profile| {
            CpuController::new(
                CpuPlanner::without_workers(),
                SeatId::Two,
                127,
                [27; 32],
                profile,
            )
        };
        assert_eq!(validate_cpus(&runtime, &[controller(accepted)]), Ok(()));
        for rejected in [archived, wrong_tier] {
            assert_eq!(
                validate_cpus(&runtime, &[controller(rejected)]),
                Err(RosterRuntimeError::UnsupportedConfiguration)
            );
        }
    }
}

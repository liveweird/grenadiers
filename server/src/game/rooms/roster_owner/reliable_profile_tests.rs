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
        roster_runtime::{
            RosterIdentity, RosterRuntime, RosterRuntimeError, SpikeRuntimeEnvironment,
        },
    },
    round_seed::RoundSeedSource,
};

#[test]
fn live_spike_owner_accepts_only_exact_current_difficulty() {
    use grenadiers_core::game::SeatId;
    for (difficulty, accepted, archived, wrong_tier, other_archived) in [
        (
            CpuDifficulty::Standard,
            CpuProfile::CONTACT_PUNCH_V60_STANDARD,
            CpuProfile::CORNER_REFUGES_V56_STANDARD,
            CpuProfile::CONTACT_PUNCH_V60_EASY,
            CpuProfile::TELEPORTER_V46_STANDARD,
        ),
        (
            CpuDifficulty::Easy,
            CpuProfile::CONTACT_PUNCH_V60_EASY,
            CpuProfile::CORNER_REFUGES_V56_EASY,
            CpuProfile::CONTACT_PUNCH_V60_STANDARD,
            CpuProfile::TELEPORTER_V46_EASY,
        ),
        (
            CpuDifficulty::Hard,
            CpuProfile::HARD_V61,
            CpuProfile::HARD_V59,
            CpuProfile::CONTACT_PUNCH_V60_STANDARD,
            CpuProfile::HARD_V47,
        ),
    ] {
        let configuration = RoomConfiguration::new(
            &[SeatControl::Human, SeatControl::Cpu],
            Some(difficulty),
            BoardSize::FifteenByThirteen,
        )
        .unwrap();
        let runtime = RosterRuntime::new_spike_v21_with_navigation(
            RosterIdentity {
                id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
                code: RoomCode::new("ABCDEFGH").unwrap(),
                registry_serial: 127,
                configuration,
            },
            RoomSettings::DEFAULT.with_bomb_capacity_drop_multiplier(
                grenadiers_core::room::BombCapacityDropMultiplier::DEFAULT,
            ),
            SpikeRuntimeEnvironment {
                cloud: StormCloudConfig::Single,
                lairs: RespawnLairTuning::DEFAULT,
                spike_pits: SpikePitTuning::DEFAULT,
                navigation: MobNavigationPolicy::TimedEscapeV3,
            },
            RoundSeedSource::with_entropy(|bytes| {
                bytes.fill(27);
                Ok(())
            }),
            Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
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
        for rejected in [archived, wrong_tier, other_archived] {
            assert_eq!(
                validate_cpus(&runtime, &[controller(rejected)]),
                Err(RosterRuntimeError::UnsupportedConfiguration)
            );
        }
    }
}

#[test]
fn future_clearance_owner_accepts_only_its_versioned_cpu_family() {
    use grenadiers_core::game::SeatId;
    for (difficulty, accepted, old_family, wrong_tier, prototype) in [
        (
            CpuDifficulty::Standard,
            CpuProfile::RECOVERY_V62_STANDARD,
            CpuProfile::CONTACT_PUNCH_V60_STANDARD,
            CpuProfile::RECOVERY_V62_EASY,
            CpuProfile::HARD_V63,
        ),
        (
            CpuDifficulty::Easy,
            CpuProfile::RECOVERY_V62_EASY,
            CpuProfile::CONTACT_PUNCH_V60_EASY,
            CpuProfile::RECOVERY_V62_STANDARD,
            CpuProfile::HARD_V63,
        ),
        (
            CpuDifficulty::Hard,
            CpuProfile::HARD_V64,
            CpuProfile::HARD_V61,
            CpuProfile::RECOVERY_V62_STANDARD,
            CpuProfile::HARD_V63,
        ),
    ] {
        let configuration = RoomConfiguration::new(
            &[SeatControl::Human, SeatControl::Cpu],
            Some(difficulty),
            BoardSize::FifteenByThirteen,
        )
        .unwrap();
        let runtime = RosterRuntime::new_spike_v21_clearance_with_navigation(
            RosterIdentity {
                id: RoomId::new("1123456789abcdef0123456789abcdef").unwrap(),
                code: RoomCode::new("HBCDEFGH").unwrap(),
                registry_serial: 128,
                configuration,
            },
            RoomSettings::DEFAULT.with_bomb_capacity_drop_multiplier(
                grenadiers_core::room::BombCapacityDropMultiplier::DEFAULT,
            ),
            SpikeRuntimeEnvironment {
                cloud: StormCloudConfig::Single,
                lairs: RespawnLairTuning::DEFAULT,
                spike_pits: SpikePitTuning::DEFAULT,
                navigation: MobNavigationPolicy::TimedEscapeV3,
            },
            RoundSeedSource::with_entropy(|bytes| {
                bytes.fill(28);
                Ok(())
            }),
            Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
        )
        .unwrap();
        let controller = |profile| {
            CpuController::new(
                CpuPlanner::without_workers(),
                SeatId::Two,
                128,
                [28; 32],
                profile,
            )
        };
        assert_eq!(validate_cpus(&runtime, &[controller(accepted)]), Ok(()));
        for rejected in [old_family, wrong_tier, prototype] {
            assert_eq!(
                validate_cpus(&runtime, &[controller(rejected)]),
                Err(RosterRuntimeError::UnsupportedConfiguration)
            );
        }
    }
}

#[test]
fn reactive_owner_changes_only_the_hard_profile() {
    use grenadiers_core::game::SeatId;
    for (difficulty, accepted, rejected) in [
        (
            CpuDifficulty::Standard,
            CpuProfile::RECOVERY_V62_STANDARD,
            CpuProfile::HARD_V65,
        ),
        (
            CpuDifficulty::Easy,
            CpuProfile::RECOVERY_V62_EASY,
            CpuProfile::HARD_V65,
        ),
        (
            CpuDifficulty::Hard,
            CpuProfile::HARD_V65,
            CpuProfile::HARD_V64,
        ),
    ] {
        let configuration = RoomConfiguration::new(
            &[SeatControl::Human, SeatControl::Cpu],
            Some(difficulty),
            BoardSize::FifteenByThirteen,
        )
        .unwrap();
        let runtime = RosterRuntime::new_spike_v21_reactive_with_navigation(
            RosterIdentity {
                id: RoomId::new("2123456789abcdef0123456789abcdef").unwrap(),
                code: RoomCode::new("RBCDEFGH").unwrap(),
                registry_serial: 129,
                configuration,
            },
            RoomSettings::DEFAULT.with_bomb_capacity_drop_multiplier(
                grenadiers_core::room::BombCapacityDropMultiplier::DEFAULT,
            ),
            SpikeRuntimeEnvironment {
                cloud: StormCloudConfig::Single,
                lairs: RespawnLairTuning::DEFAULT,
                spike_pits: SpikePitTuning::DEFAULT,
                navigation: MobNavigationPolicy::TimedEscapeV3,
            },
            RoundSeedSource::with_entropy(|bytes| {
                bytes.fill(29);
                Ok(())
            }),
            Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
        )
        .unwrap();
        let controller = |profile| {
            CpuController::new(
                CpuPlanner::without_workers(),
                SeatId::Two,
                129,
                [29; 32],
                profile,
            )
        };
        assert_eq!(validate_cpus(&runtime, &[controller(accepted)]), Ok(()));
        assert_eq!(
            validate_cpus(&runtime, &[controller(rejected)]),
            Err(RosterRuntimeError::UnsupportedConfiguration)
        );
    }
}

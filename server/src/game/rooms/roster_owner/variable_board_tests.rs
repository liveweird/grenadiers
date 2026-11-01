use std::sync::Arc;

use grenadiers_core::{
    game::{SeatId, SimulationSeed},
    protocol::{RoomCode, RoomId},
    room::{BoardSize, CpuDifficulty, CpuProfile, RoomConfiguration, RoomSettings, SeatControl},
};

use crate::{
    game::rooms::{
        cpu::{CpuController, CpuPlanner},
        roster_runtime::{RosterIdentity, RosterRuntime, RosterRuntimeError},
    },
    round_seed::RoundSeedSource,
};

#[test]
fn small_two_player_cpus_use_policy_seventeen_and_reject_legacy_profile()
-> Result<(), RosterRuntimeError> {
    for difficulty in [CpuDifficulty::Easy, CpuDifficulty::Standard] {
        let configuration = RoomConfiguration::new(
            &[SeatControl::Human, SeatControl::Cpu],
            Some(difficulty),
            BoardSize::ElevenByNine,
        )
        .unwrap();
        let runtime = RosterRuntime::new(
            RosterIdentity {
                id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
                code: RoomCode::new("ABCDEFGH").unwrap(),
                registry_serial: 9,
                configuration,
            },
            RoomSettings::DEFAULT,
            RoundSeedSource::with_entropy(|bytes| {
                bytes.fill(3);
                Ok(())
            }),
            Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
        )
        .unwrap();
        let (profile, legacy) = archived_profiles(difficulty)?;
        let controller = |profile| {
            CpuController::new(
                CpuPlanner::without_workers(),
                SeatId::Two,
                9,
                [77; 32],
                profile,
            )
        };

        assert_eq!(profile.policy.version(), 17);
        assert_eq!(
            super::super::validate_cpus(&runtime, &[controller(profile)]),
            Ok(())
        );
        assert_eq!(
            super::super::validate_cpus(&runtime, &[controller(legacy)]),
            Err(RosterRuntimeError::UnsupportedConfiguration)
        );
    }
    Ok(())
}

fn archived_profiles(
    difficulty: CpuDifficulty,
) -> Result<(CpuProfile, CpuProfile), RosterRuntimeError> {
    match difficulty {
        CpuDifficulty::Easy => Ok((CpuProfile::MULTIPLAYER_EASY, CpuProfile::CURRENT_EASY)),
        CpuDifficulty::Standard => Ok((CpuProfile::MULTIPLAYER_STANDARD, CpuProfile::CURRENT)),
        CpuDifficulty::Hard => Err(RosterRuntimeError::UnsupportedConfiguration),
    }
}

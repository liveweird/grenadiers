use std::sync::Arc;

use grenadiers_core::{
    game::{SeatId, SimulationSeed, StormCloudConfig},
    protocol::{RoomCode, RoomId},
    room::{BoardSize, CpuDifficulty, CpuProfile, RoomConfiguration, RoomSettings, SeatControl},
};
use tokio::{sync::watch, time::Instant};

use super::{super::RandomFill, PreparedRosterOwner};
use crate::{
    game::rooms::{
        cpu::{CpuController, CpuPlanner},
        roster_runtime::{RosterIdentity, RosterRuntime},
    },
    round_seed::RoundSeedSource,
};

#[test]
fn disabled_environment_owner_requires_cpu18_profile() {
    let configuration = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu],
        Some(CpuDifficulty::Standard),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let runtime = || {
        RosterRuntime::new_environment(
            RosterIdentity {
                id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
                code: RoomCode::new("ABCDEFGH").unwrap(),
                registry_serial: 9,
                configuration,
            },
            RoomSettings::DEFAULT,
            StormCloudConfig::Disabled,
            RoundSeedSource::with_entropy(|bytes| {
                bytes.fill(3);
                Ok(())
            }),
            Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
        )
        .unwrap()
    };
    let cpu = |profile| {
        vec![CpuController::new(
            CpuPlanner::without_workers(),
            SeatId::Two,
            9,
            [2; 32],
            profile,
        )]
    };
    let random_fill: Arc<RandomFill> = Arc::new(|bytes| {
        bytes.fill(1);
        Ok(())
    });
    let (_shutdown, receiver) = watch::channel(false);
    assert!(
        PreparedRosterOwner::new(
            runtime(),
            cpu(CpuProfile::ENVIRONMENT_STANDARD),
            Instant::now(),
            Arc::clone(&random_fill),
            receiver.clone(),
        )
        .is_ok()
    );
    assert!(
        PreparedRosterOwner::new(
            runtime(),
            cpu(CpuProfile::MULTIPLAYER_STANDARD),
            Instant::now(),
            random_fill,
            receiver,
        )
        .is_err()
    );
}

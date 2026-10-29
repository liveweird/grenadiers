use std::sync::Arc;

use grenadiers_core::{
    game::{SeatId, SimulationSeed, StormCloudConfig},
    protocol::{Counter, RoomCode, RoomId, v15::ResumeToken},
    room::{BoardSize, CpuDifficulty, RoomConfiguration, RoomSettings, SeatControl},
};

use super::{RosterAction, RosterIdentity, RosterRuntime};
use crate::round_seed::RoundSeedSource;

#[test]
fn running_environment_resume_prepares_an_atomic_current_epoch_baseline() {
    for board in [
        BoardSize::FifteenByThirteen,
        BoardSize::ThirteenByEleven,
        BoardSize::ElevenByNine,
    ] {
        for environment in [StormCloudConfig::Disabled, StormCloudConfig::Single] {
            let configuration = RoomConfiguration::new(
                &[SeatControl::Human, SeatControl::Cpu, SeatControl::Human],
                Some(CpuDifficulty::Easy),
                board,
            )
            .unwrap();
            let mut runtime = RosterRuntime::new_environment(
                RosterIdentity {
                    id: RoomId::new("0123456789abcdef0123456789abcdef").unwrap(),
                    code: RoomCode::new("ABCDEFGH").unwrap(),
                    registry_serial: 3114,
                    configuration,
                },
                RoomSettings::DEFAULT,
                environment,
                RoundSeedSource::with_entropy(|bytes| {
                    bytes.fill(37);
                    Ok(())
                }),
                Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
            )
            .unwrap();
            for seat in [SeatId::One, SeatId::Three] {
                let admission = runtime.prepare_join(seat).unwrap();
                runtime.commit_admission(admission);
                runtime
                    .action(
                        seat,
                        1,
                        RosterAction::Ready {
                            ready: true,
                            settings_revision: 1,
                        },
                    )
                    .unwrap();
            }
            for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
                runtime.settle_boundary([None; 4]).unwrap();
            }
            runtime
                .action(SeatId::Three, 1, RosterAction::Bomb(1))
                .unwrap();
            runtime.settle_boundary([None; 4]).unwrap();
            let before = runtime.replay_inspection();
            assert!(runtime.prepare_resume(SeatId::Two).is_err());
            assert_eq!(runtime.replay_inspection(), before);

            let prepared = runtime.prepare_resume(SeatId::Three).unwrap();
            assert!(prepared.is_replacement);
            let token = ResumeToken::new("b".repeat(64)).unwrap();
            let frame = prepared.encode(Counter::new(9).unwrap(), token).unwrap();
            let value: serde_json::Value = serde_json::from_str(&frame).unwrap();
            assert_eq!(value["type"], "admitted");
            assert_eq!(value["youSeat"], "three");
            assert_eq!(value["lastActionId"], 1);
            assert_eq!(value["room"]["rulesVersion"], 7);
            assert_eq!(value["room"]["phase"]["type"], "running");
            assert_eq!(value["room"]["roundId"], 1);
            assert_eq!(value["state"]["physicalTick"], value["room"]["tick"]);
            assert_eq!(value["state"]["actors"][2]["type"], "visible");
            assert_eq!(value["state"]["actors"][2]["seat"], "three");
            assert_eq!(value["state"]["bombs"].as_array().unwrap().len(), 1);
            assert_eq!(
                value["state"]["stormCloud"].is_null(),
                environment == StormCloudConfig::Disabled
            );
            assert!(value.get("events").is_none());
            assert_eq!(runtime.replay_inspection(), before);

            runtime.commit_admission(prepared);
            let after = runtime.replay_inspection();
            assert_eq!(after.simulation, before.simulation);
            assert_eq!(after.revision, before.revision + 1);
            assert_eq!(after.settings_revision, before.settings_revision);
            assert_eq!(after.environment, Some(environment));
        }
    }
}

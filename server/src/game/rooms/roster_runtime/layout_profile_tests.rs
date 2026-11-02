use std::{num::NonZeroU32, sync::Arc};

use grenadiers_core::{
    game::{
        ArenaConstructionProfile, EnabledItems, RoundAssignment, SeatId, SimulationSeed,
        SpikePitLayoutPolicy, StormCloudConfig,
    },
    protocol::{Counter, v14::ResumeToken},
    room::{
        BoardSize, CpuDifficulty, RespawnLairTuning, RoomConfiguration, RoomSettings, SeatControl,
        SpikePitTuning,
    },
};

use super::{RosterAction, RosterRuntime};
use crate::{game::rooms::roster_runtime::tests::identity, round_seed::RoundSeedSource};

fn runtime(configuration: RoomConfiguration) -> RosterRuntime {
    RosterRuntime::new_spike_v20_profiled_with_navigation(
        identity(configuration),
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        StormCloudConfig::Single,
        RespawnLairTuning::DEFAULT,
        SpikePitTuning::DEFAULT,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(73);
            Ok(())
        }),
        Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
        grenadiers_core::game::MobNavigationPolicy::TimedEscapeV3,
        ArenaConstructionProfile::LairClearanceV5,
    )
    .unwrap()
}

#[test]
fn gameplay20_keeps_v5_for_rounds_resume_and_cpu_observation() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        let configuration = RoomConfiguration::new(
            &[SeatControl::Human, SeatControl::Cpu],
            Some(CpuDifficulty::Standard),
            board,
        )
        .unwrap();
        let mut runtime = runtime(configuration);
        assert_eq!(
            runtime.replay_inspection().construction_profile,
            ArenaConstructionProfile::LairClearanceV5
        );
        let joined = runtime.prepare_join(SeatId::One).unwrap();
        runtime.commit_admission(joined);
        runtime
            .action(
                SeatId::One,
                1,
                RosterAction::Ready {
                    ready: true,
                    settings_revision: 1,
                },
            )
            .unwrap();
        for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
            runtime.settle_boundary([None; 4]).unwrap();
        }

        let simulation = runtime.replay_inspection().simulation.unwrap();
        assert_eq!(simulation.generation.version(), 5);
        assert_eq!(
            simulation.state.rules().spike_pit_layout_policy(),
            SpikePitLayoutPolicy::LairClearanceV2
        );
        let expected_pits = simulation.state.spike_pits().unwrap().to_vec();
        let observation = grenadiers_core::protocol::v20::observe_cpu_recipient(
            runtime
                .current_notice()
                .unwrap()
                .spike_v20_cpu_publication()
                .unwrap(),
            grenadiers_core::protocol::v20::Seat::Two,
            grenadiers_core::cpu::WorldId {
                room_instance: 32,
                physical_round: runtime.lifecycle.round_attempt(),
            },
        )
        .unwrap();
        assert_eq!(observation.spike_pits().unwrap().pits(), expected_pits);

        runtime
            .prepare_resume(SeatId::One)
            .unwrap()
            .encode(
                Counter::new(32).unwrap(),
                ResumeToken::new(
                    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                )
                .unwrap(),
            )
            .unwrap();

        runtime.reset_for_rematch();
        runtime
            .start_round(RoundAssignment::new(NonZeroU32::new(2).unwrap()))
            .unwrap();
        let later = runtime.replay_inspection().simulation.unwrap();
        assert_eq!(later.generation.version(), 5);
        assert_eq!(
            later.state.rules().spike_pit_layout_policy(),
            SpikePitLayoutPolicy::LairClearanceV2
        );
    }
}

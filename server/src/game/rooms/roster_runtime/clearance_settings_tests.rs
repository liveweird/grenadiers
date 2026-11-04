use super::{RosterAction, RosterRuntime, SpikeRuntimeEnvironment, tests::identity};
use crate::round_seed::RoundSeedSource;
use grenadiers_core::{
    game::{
        ArenaConstructionProfile, EnabledItems, MobNavigationPolicy, SeatId, SimulationSeed,
        SpikePitLayoutPolicy, StormCloudConfig,
    },
    protocol::{Counter, v14::ResumeToken, v21},
    room::{
        BoardSize, BombCapacityDropMultiplier, InterruptReason, MatchPhase, RespawnLairTuning,
        RoomConfiguration, RoomSettings, SeatControl, SpikePitTuning,
    },
};
use std::sync::Arc;

fn settings(multiplier: u64) -> RoomSettings {
    RoomSettings::DEFAULT
        .with_enabled_items(EnabledItems::ALL)
        .with_bomb_capacity_drop_multiplier(BombCapacityDropMultiplier::new(multiplier).unwrap())
}
fn runtime(board: BoardSize, count: usize, multiplier: u64) -> RosterRuntime {
    RosterRuntime::new_spike_v21_clearance_with_navigation(
        identity(RoomConfiguration::new(&vec![SeatControl::Human; count], None, board).unwrap()),
        settings(multiplier),
        SpikeRuntimeEnvironment {
            cloud: StormCloudConfig::Single,
            lairs: RespawnLairTuning::DEFAULT,
            spike_pits: SpikePitTuning::DEFAULT,
            navigation: MobNavigationPolicy::TimedEscapeV3,
        },
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(73);
            Ok(())
        }),
        Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
    )
    .unwrap()
}
fn ready(runtime: &mut RosterRuntime, count: usize) {
    for &seat in &SeatId::ALL[..count] {
        runtime
            .action(
                seat,
                runtime.lifecycle.round_attempt(),
                RosterAction::Ready {
                    ready: true,
                    settings_revision: runtime.settings_revision,
                },
            )
            .unwrap();
    }
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    assert_eq!(runtime.lifecycle.phase(), MatchPhase::Running);
}
fn assert_body(runtime: &mut RosterRuntime, multiplier: u64) {
    let inspection = runtime.replay_inspection();
    assert_eq!(
        inspection.construction_profile,
        ArenaConstructionProfile::WeightedLairClearanceV7
    );
    assert!(runtime.uses_clearance_cpu_profiles());
    let sim = inspection.simulation.unwrap();
    assert_eq!(sim.generation.version(), 7);
    assert_eq!(
        sim.state.rules().spike_pit_layout_policy(),
        SpikePitLayoutPolicy::LairClearanceV2
    );
    let notice = runtime.current_notice().unwrap();
    let room = notice.spike_v21_room().unwrap();
    assert_eq!(
        room.settings.bomb_capacity_drop_multiplier().value(),
        multiplier
    );
    let capture = v21::capture_expanded(&sim.state, &[]).unwrap();
    assert!(
        v21::bind_expanded(
            room,
            Counter::new(sim.epoch as u32).unwrap(),
            &capture,
            sim.generation
        )
        .is_err()
    );
    let prepared = runtime.prepare_resume(SeatId::One).unwrap();
    let encoded = prepared
        .encode(
            Counter::new(9).unwrap(),
            ResumeToken::new("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .unwrap(),
        )
        .unwrap();
    let admitted: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(
        admitted["room"]["settings"]["bombCapacityDropMultiplier"],
        multiplier
    );
    assert!(admitted["state"].is_object());
    runtime.commit_admission(prepared);
}

#[test]
fn clearance21_snapshots_and_resumes_bind_all_boards_rosters_and_weights() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        for count in 2..=4 {
            for multiplier in [1, 5, 10] {
                let mut runtime = runtime(board, count, multiplier);
                super::tests::admit_all(&mut runtime, count);
                ready(&mut runtime, count);
                assert_body(&mut runtime, multiplier);
            }
        }
    }
}

#[test]
fn clearance21_preserves_construction_through_settings_and_rematch() {
    let mut runtime = runtime(BoardSize::ThirteenByEleven, 2, 5);
    super::tests::admit_all(&mut runtime, 2);
    runtime
        .update_spike_settings(
            SeatId::One,
            1,
            1,
            settings(10),
            StormCloudConfig::Single,
            RespawnLairTuning::DEFAULT,
            SpikePitTuning::DEFAULT,
        )
        .unwrap();
    ready(&mut runtime, 2);
    assert_body(&mut runtime, 10);
    runtime.interrupt(InterruptReason::ServerStopped).unwrap();
    for _ in 0..grenadiers_core::room::series::RESULT_HOLD_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    for seat in SeatId::DUEL {
        runtime
            .action(
                seat,
                runtime.lifecycle.round_attempt(),
                RosterAction::Rematch(true),
            )
            .unwrap();
    }
    assert_eq!(runtime.lifecycle.phase(), MatchPhase::Waiting);
    ready(&mut runtime, 2);
    assert_body(&mut runtime, 10);
}

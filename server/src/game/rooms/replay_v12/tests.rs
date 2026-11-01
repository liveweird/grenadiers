use grenadiers_core::{
    game::{Direction, EnabledItems, MobNavigationPolicy, SeatId, SpeedLevel, StormCloudConfig},
    room::{
        BoardSize, CpuDifficulty, DepartureReason, LairRoomConfiguration, RespawnLairSettings,
        RespawnLairTuning, RoomConfiguration, RoomSettings, SeatControl, SpikePitSettings,
        SpikePitTuning, SpikeRoomConfiguration,
    },
};

use super::*;
use crate::round_seed::RoundSeedSource;

fn configuration(spike_pits: SpikePitTuning) -> SpikeRoomConfiguration {
    SpikeRoomConfiguration::new(
        LairRoomConfiguration::new(
            RoomConfiguration::new(
                &[SeatControl::Human, SeatControl::Cpu],
                Some(CpuDifficulty::Standard),
                BoardSize::FifteenByThirteen,
            )
            .unwrap(),
            StormCloudConfig::Single,
            RespawnLairTuning::new(true, RespawnLairSettings::new(5).unwrap()).as_legacy(),
        ),
        spike_pits,
    )
}

fn recorder(spike_pits: SpikePitTuning) -> RecordingRosterRuntime {
    RecordingRosterRuntime::new(
        configuration(spike_pits),
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(10);
            Ok(())
        }),
        vec![(SeatId::Two, CpuDifficulty::Standard, [27; 32])],
    )
    .unwrap()
}

fn start(recorder: &mut RecordingRosterRuntime, pits: SpikePitTuning) {
    recorder.admit(SeatId::One, false).unwrap();
    recorder
        .human_action(
            SeatId::One,
            1,
            ReplayAction::UpdateSpikeSettings {
                settings_revision: 1,
                settings: RoomSettings::new(SpeedLevel::One, 256, 3, 180)
                    .unwrap()
                    .with_enabled_items(EnabledItems::ALL),
                cloud: StormCloudConfig::Single,
                tuning: RespawnLairTuning::new(true, RespawnLairSettings::new(5).unwrap()),
                spike_pits: pits,
            },
        )
        .unwrap();
    recorder
        .human_action(
            SeatId::One,
            1,
            ReplayAction::Ready {
                ready: true,
                settings_revision: 2,
            },
        )
        .unwrap();
    for _ in 0..181 {
        recorder.settle([None; 4]).unwrap();
    }
}

#[test]
fn replay12_reconstructs_rules10_generation_lifecycle_and_accepted_cpu_inputs() {
    let pits = SpikePitTuning::new(true, SpikePitSettings::new(6, 60).unwrap());
    let mut recorder = recorder(pits);
    start(&mut recorder, pits);
    recorder
        .cpu_action(
            SeatId::Two,
            1,
            ReplayAction::Movement(Some(Direction::West)),
        )
        .unwrap();
    recorder
        .cpu_action(SeatId::Two, 1, ReplayAction::Bomb(1))
        .unwrap();
    for _ in 0..12 {
        recorder.settle([None; 4]).unwrap();
    }
    recorder
        .cpu_action(SeatId::Two, 1, ReplayAction::Movement(None))
        .unwrap();
    recorder.input_fence(SeatId::One);
    recorder.publish_flush().unwrap();
    recorder
        .settle([Some(DepartureReason::TransportLoss), None, None, None])
        .unwrap();
    recorder.admit(SeatId::One, true).unwrap();
    for _ in 0..360 {
        recorder.settle([None; 4]).unwrap();
    }

    let expected = recorder.runtime().replay_inspection();
    let state = &expected.simulation.as_ref().unwrap().state;
    assert_eq!(state.rules().version(), 10);
    assert_eq!(
        state.rules().mob_navigation_policy(),
        MobNavigationPolicy::TimedEscapeV3
    );
    assert!(state.storm_cloud().is_some());
    assert!(state.lair_statuses().is_some());
    assert!(state.spike_pits().is_some());
    let (_, journal) = recorder.into_parts();
    verify_exact(&journal, &expected).unwrap();

    assert!(super::super::replay_v8::replay(&journal.body).is_err());
}

#[test]
fn replay12_retains_disabled_spike_numbers_without_constructing_pits() {
    let retained = SpikePitTuning::new(false, SpikePitSettings::new(12, 240).unwrap());
    let mut recorder = recorder(retained);
    start(&mut recorder, retained);
    for _ in 0..20 {
        recorder.settle([None; 4]).unwrap();
    }
    let expected = recorder.runtime().replay_inspection();
    assert!(
        expected
            .simulation
            .as_ref()
            .unwrap()
            .state
            .spike_pits()
            .is_none()
    );
    let (_, journal) = recorder.into_parts();
    assert_eq!(journal.header.configuration.spike_pits(), retained);
    verify_exact(&journal, &expected).unwrap();
}

#[test]
fn replay12_rejects_cross_profile_headers_provenance_and_incomplete_records() {
    let pits = SpikePitTuning::DEFAULT;
    let base_recorder = recorder(pits);
    let (_, journal) = base_recorder.into_parts();
    for field in ["format", "rules", "cpu", "navigation", "layout"] {
        let mut invalid = journal.clone();
        match field {
            "format" => invalid.header.format_version = 11,
            "rules" => invalid.header.rules_version = 9,
            "cpu" => invalid.header.cpu_policy_version = 26,
            "navigation" => invalid.header.mob_navigation_version = 2,
            "layout" => invalid.header.spike_pit_layout_version = 2,
            _ => unreachable!(),
        }
        assert!(replay(&invalid).is_err(), "accepted invalid {field}");
    }
    let mut mixed = journal.clone();
    mixed.body.header.cpus[0].policy_version = 26;
    assert!(replay(&mixed).is_err());

    let mut malformed_recorder = recorder(pits);
    malformed_recorder.admit(SeatId::One, false).unwrap();
    let (_, mut malformed) = malformed_recorder.into_parts();
    malformed.body.records[0].order = 1;
    assert_eq!(replay(&malformed), Err(ReplayError::InvalidOrder));

    let mut truncated = journal.clone();
    truncated.body.status = replay_v8::ReplayStatus::Truncated;
    assert_eq!(replay(&truncated), Err(ReplayError::Truncated));

    let mut incomplete = journal;
    incomplete.body.status = replay_v8::ReplayStatus::IncompleteRoundConstruction;
    assert_eq!(replay(&incomplete), Err(ReplayError::Incomplete));
}

#[test]
fn replay12_rejects_consistent_hard_header_and_cpu_provenance() {
    let (_, mut journal) = recorder(SpikePitTuning::DEFAULT).into_parts();
    let old = &journal.header.configuration;
    let hard = SpikeRoomConfiguration::new(
        LairRoomConfiguration::new(
            RoomConfiguration::new(
                &[SeatControl::Human, SeatControl::Cpu],
                Some(CpuDifficulty::Hard),
                old.base().base().board(),
            )
            .unwrap(),
            old.base().storm_cloud(),
            old.base().lairs(),
        ),
        old.spike_pits(),
    );
    journal.header.configuration = hard.clone();
    journal.body.header.configuration = hard.base().clone();
    journal.header.cpus[0].difficulty = CpuDifficulty::Hard;
    journal.body.header.cpus[0].difficulty = CpuDifficulty::Hard;
    assert_eq!(replay(&journal), Err(ReplayError::InvalidHeader));
    assert!(
        RecordingRosterRuntime::new(
            hard,
            RoomSettings::DEFAULT,
            RoundSeedSource::with_entropy(|bytes| {
                bytes.fill(12);
                Ok(())
            }),
            vec![(SeatId::Two, CpuDifficulty::Hard, [12; 32])],
        )
        .is_err()
    );
}

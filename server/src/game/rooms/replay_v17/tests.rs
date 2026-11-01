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
        vec![(SeatId::Two, CpuDifficulty::Standard, [33; 32])],
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
fn replay17_reconstructs_rules10_generation_lifecycle_and_accepted_cpu_inputs() {
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
fn replay17_retains_disabled_spike_numbers_without_constructing_pits() {
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
fn replay17_rejects_cross_profile_headers_provenance_and_incomplete_records() {
    let pits = SpikePitTuning::DEFAULT;
    let base_recorder = recorder(pits);
    let (_, journal) = base_recorder.into_parts();
    for field in ["format", "rules", "cpu", "navigation", "layout"] {
        let mut invalid = journal.clone();
        match field {
            "format" => invalid.header.format_version = 13,
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
fn replay17_binds_gameplay19_and_exact_easy_standard_hard_policy_pairs() {
    for (difficulty, policy) in [
        (CpuDifficulty::Easy, 33),
        (CpuDifficulty::Standard, 33),
        (CpuDifficulty::Hard, 35),
    ] {
        let pits = SpikePitTuning::DEFAULT;
        let original = configuration(pits);
        let selected = SpikeRoomConfiguration::new(
            LairRoomConfiguration::new(
                RoomConfiguration::new(
                    &[SeatControl::Human, SeatControl::Cpu],
                    Some(difficulty),
                    original.base().base().board(),
                )
                .unwrap(),
                original.base().storm_cloud(),
                original.base().lairs(),
            ),
            pits,
        );
        let mut recording = RecordingRosterRuntime::new(
            selected,
            RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
            RoundSeedSource::with_entropy(|bytes| {
                bytes.fill(13);
                Ok(())
            }),
            vec![(SeatId::Two, difficulty, [13; 32])],
        )
        .unwrap();
        assert!(recording.runtime().uses_gameplay19());
        start(&mut recording, pits);
        recording
            .cpu_action(
                SeatId::Two,
                1,
                ReplayAction::Movement(Some(Direction::West)),
            )
            .unwrap();
        recording
            .cpu_action(SeatId::Two, 1, ReplayAction::Bomb(1))
            .unwrap();
        for _ in 0..12 {
            recording.settle([None; 4]).unwrap();
        }
        recording
            .cpu_action(SeatId::Two, 1, ReplayAction::Movement(None))
            .unwrap();
        let mut runtime = recording.runtime().clone();
        let notice = runtime.current_notice().unwrap();
        assert!(notice.spike_room().is_none());
        let room = notice.spike_v19_room().unwrap();
        assert_eq!(
            room.configuration.domain().base().base().cpu_difficulty(),
            Some(difficulty)
        );
        let expected = recording.runtime().replay_inspection();
        let (_, journal) = recording.into_parts();
        assert_eq!(journal.header.cpu_policy_version, policy);
        assert_eq!(journal.header.gameplay_version, 19);
        verify_exact(&journal, &expected).unwrap();

        for changed in ["gameplay", "headerPolicy", "bodyPolicy", "provenance"] {
            let mut bad = journal.clone();
            let wrong_policy = if policy == 33 { 35 } else { 33 };
            match changed {
                "gameplay" => bad.header.gameplay_version = 18,
                "headerPolicy" => bad.header.cpu_policy_version = wrong_policy,
                "bodyPolicy" => bad.body.header.cpu_policy_version = wrong_policy,
                "provenance" => {
                    bad.header.cpus[0].policy_version = wrong_policy;
                    bad.body.header.cpus[0].policy_version = wrong_policy;
                }
                _ => unreachable!(),
            }
            assert_eq!(
                replay(&bad),
                Err(ReplayError::InvalidHeader),
                "{difficulty:?} {changed}"
            );
        }
    }
}

#[test]
fn replay17_records_human_only_rooms_with_no_cpu_provenance() {
    let pits = SpikePitTuning::DEFAULT;
    let original = configuration(pits);
    let selected = SpikeRoomConfiguration::new(
        LairRoomConfiguration::new(
            RoomConfiguration::new(
                &[SeatControl::Human, SeatControl::Human],
                None,
                original.base().base().board(),
            )
            .unwrap(),
            original.base().storm_cloud(),
            original.base().lairs(),
        ),
        pits,
    );
    let mut recording = RecordingRosterRuntime::new(
        selected,
        RoomSettings::DEFAULT,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(15);
            Ok(())
        }),
        vec![],
    )
    .unwrap();
    for seat in [SeatId::One, SeatId::Two] {
        recording.admit(seat, false).unwrap();
        recording
            .human_action(
                seat,
                1,
                ReplayAction::Ready {
                    ready: true,
                    settings_revision: 1,
                },
            )
            .unwrap();
    }
    for _ in 0..181 {
        recording.settle([None; 4]).unwrap();
    }
    recording
        .human_action(
            SeatId::One,
            1,
            ReplayAction::Movement(Some(Direction::East)),
        )
        .unwrap();
    recording.settle([None; 4]).unwrap();
    let expected = recording.runtime().replay_inspection();
    let (_, journal) = recording.into_parts();
    assert_eq!(journal.header.cpu_policy_version, 33);
    assert!(journal.header.cpus.is_empty());
    verify_exact(&journal, &expected).unwrap();
    let mut forged = journal;
    forged.header.cpu_policy_version = 35;
    forged.body.header.cpu_policy_version = 35;
    assert_eq!(replay(&forged), Err(ReplayError::InvalidHeader));
}

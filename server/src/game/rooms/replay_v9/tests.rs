use super::*;
use crate::round_seed::RoundSeedSource;
use grenadiers_core::{
    game::{Direction, EnabledItems, MobNavigationPolicy, SeatId, StormCloudConfig},
    room::{
        BoardSize, CpuDifficulty, LairRoomConfiguration, RespawnLairSettings, RespawnLairTuning,
        RoomConfiguration, RoomSettings, SeatControl,
    },
};

fn recorder() -> RecordingRosterRuntime {
    RecordingRosterRuntime::new(
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
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(3);
            Ok(())
        }),
        vec![CpuProvenance::new(
            SeatId::Two,
            CpuDifficulty::Standard,
            [8; 32],
        )],
    )
    .unwrap()
}

#[test]
fn replay9_reconstructs_current_navigation_and_cpu_inputs_without_rerunning_ai() {
    let mut recorder = recorder();
    recorder.admit(SeatId::One, false).unwrap();
    recorder
        .human_action(
            SeatId::One,
            1,
            ReplayAction::UpdateSettings {
                settings_revision: 1,
                settings: RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
                cloud: StormCloudConfig::Disabled,
                tuning: RespawnLairTuning::new(true, RespawnLairSettings::new(5).unwrap()),
            },
        )
        .unwrap();
    assert_eq!(
        recorder.runtime().replay_inspection().mob_navigation,
        MobNavigationPolicy::EscapeV2
    );
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
    recorder
        .cpu_action(
            SeatId::Two,
            1,
            ReplayAction::Movement(Some(Direction::West)),
        )
        .unwrap();
    for _ in 0..12 {
        recorder.settle([None; 4]).unwrap();
    }
    recorder
        .cpu_action(SeatId::Two, 1, ReplayAction::Movement(None))
        .unwrap();
    for _ in 0..360 {
        recorder.settle([None; 4]).unwrap();
    }
    let expected = recorder.runtime().replay_inspection();
    assert_eq!(expected.mob_navigation, MobNavigationPolicy::EscapeV2);
    assert_eq!(
        expected
            .simulation
            .as_ref()
            .unwrap()
            .state
            .rules()
            .mob_navigation_policy(),
        MobNavigationPolicy::EscapeV2
    );
    let (_, journal) = recorder.into_parts();
    verify_exact(&journal, &expected).unwrap();
    for version in [23, 24] {
        let mut archived = journal.clone();
        archived.body = archived.body.with_cpu_policy_for_test(version);
        verify_exact(&archived, &expected).unwrap();
    }
    for version in [22, 25] {
        let mut invalid = journal.clone();
        invalid.body = invalid.body.with_cpu_policy_for_test(version);
        assert!(replay(&invalid).is_err());
    }
    assert!(replay_v8::verify_exact(&journal.body, &expected).is_err());
    for (format, navigation) in [(8, 2), (10, 2), (9, 1), (9, 3)] {
        let mut invalid = journal.clone();
        invalid.format_version = format;
        invalid.mob_navigation_version = navigation;
        assert!(replay(&invalid).is_err());
    }
}

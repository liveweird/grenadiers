use grenadiers_core::{
    game::{EnabledItems, MobNavigationPolicy, SeatId, StormCloudConfig},
    room::{
        BombCapacityDropMultiplier, CpuDifficulty, LairRoomConfiguration, RespawnLairTuning,
        RoomConfiguration, RoomSettings, SeatControl, SpikePitSettings, SpikePitTuning,
        SpikeRoomConfiguration,
    },
};

use super::*;
use crate::round_seed::RoundSeedSource;

fn configuration(difficulty: Option<CpuDifficulty>) -> SpikeRoomConfiguration {
    let controllers = if difficulty.is_some() {
        vec![SeatControl::Human, SeatControl::Cpu]
    } else {
        vec![SeatControl::Human, SeatControl::Human]
    };
    SpikeRoomConfiguration::new(
        LairRoomConfiguration::new(
            RoomConfiguration::new(
                &controllers,
                difficulty,
                grenadiers_core::room::BoardSize::FifteenByThirteen,
            )
            .unwrap(),
            StormCloudConfig::Single,
            RespawnLairTuning::DEFAULT.as_legacy(),
        ),
        SpikePitTuning::new(true, SpikePitSettings::new(36, 180).unwrap()),
    )
}

fn recorder(difficulty: Option<CpuDifficulty>) -> RecordingRosterRuntime {
    let cpus = difficulty
        .map(|difficulty| vec![(SeatId::Two, difficulty, [40; 32])])
        .unwrap_or_default();
    RecordingRosterRuntime::new(
        configuration(difficulty),
        RoomSettings::DEFAULT
            .with_enabled_items(EnabledItems::ALL)
            .with_bomb_capacity_drop_multiplier(BombCapacityDropMultiplier::DEFAULT),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(40);
            Ok(())
        }),
        cpus,
    )
    .unwrap()
}

fn start(recording: &mut RecordingRosterRuntime) {
    recording.admit(SeatId::One, false).unwrap();
    recording
        .human_action(
            SeatId::One,
            1,
            ReplayAction::Ready {
                ready: true,
                settings_revision: 1,
            },
        )
        .unwrap();
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS + 3 {
        recording.settle([None; 4]).unwrap();
    }
}

#[test]
fn replay40_closes_reached_target_navigation_and_current_cpu_profiles() {
    for (difficulty, policy) in [
        (CpuDifficulty::Easy, 68),
        (CpuDifficulty::Standard, 62),
        (CpuDifficulty::Hard, 70),
    ] {
        let mut recording = recorder(Some(difficulty));
        start(&mut recording);
        let expected = recording.runtime().replay_inspection();
        assert_eq!(
            expected.mob_navigation,
            MobNavigationPolicy::ReachedTargetV4
        );
        let (_, journal) = recording.into_parts();
        assert_eq!(journal.header.format_version, 40);
        assert_eq!(journal.header.mob_navigation_version, 4);
        assert_eq!(journal.header.cpu_policy_version, policy);
        verify_exact(&journal, &expected).unwrap();

        let mut old_navigation = journal.clone();
        old_navigation.header.mob_navigation_version = 3;
        assert_eq!(replay(&old_navigation), Err(ReplayError::InvalidHeader));

        let mut old_format = journal.clone();
        old_format.header.format_version = 39;
        assert_eq!(replay(&old_format), Err(ReplayError::InvalidHeader));
    }
}

#[test]
fn replay39_remains_closed_to_timed_escape_navigation() {
    let archived = super::super::replay_v39::RecordingRosterRuntime::new_for_evaluation(
        configuration(Some(CpuDifficulty::Standard)),
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(39);
            Ok(())
        }),
        [113; 32],
        vec![(SeatId::Two, CpuDifficulty::Standard, [71; 32])],
    )
    .unwrap();
    assert_eq!(
        archived.runtime().replay_inspection().mob_navigation,
        MobNavigationPolicy::TimedEscapeV3
    );
    assert_eq!(
        archived.runtime().hard_cpu_profile(),
        Some(grenadiers_core::room::CpuProfile::HARD_V70)
    );
}

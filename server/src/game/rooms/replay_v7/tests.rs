use grenadiers_core::{
    game::{EnabledItems, SeatId, SpeedLevel, StormCloudConfig},
    room::{
        BoardSize, LairRoomConfiguration, LairSettings, LairTuning, RoomConfiguration,
        RoomSettings, SeatControl,
    },
};

use crate::round_seed::RoundSeedSource;

use super::{CpuProvenance, RecordingRosterRuntime, ReplayAction, replay, verify_exact};

fn recorder(board: BoardSize, tuning: LairTuning) -> RecordingRosterRuntime {
    let base =
        RoomConfiguration::new(&[SeatControl::Human, SeatControl::Human], None, board).unwrap();
    RecordingRosterRuntime::new(
        LairRoomConfiguration::new(base, StormCloudConfig::Single, tuning),
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(37);
            Ok(())
        }),
        vec![],
    )
    .unwrap()
}

fn start(recorder: &mut RecordingRosterRuntime, revision: u32) {
    for seat in SeatId::DUEL {
        recorder.admit(seat, false).unwrap();
        recorder
            .human_action(
                seat,
                1,
                ReplayAction::Ready {
                    ready: true,
                    settings_revision: revision,
                },
            )
            .unwrap();
    }
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        recorder.settle([None; 4]).unwrap();
    }
}

#[test]
fn replay7_reconstructs_full_rules8_state_after_mob_spawns() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        let mut recorder = recorder(
            board,
            LairTuning::new(true, LairSettings::new(5, 2).unwrap()),
        );
        start(&mut recorder, 1);
        for _ in 0..360 {
            recorder.settle([None; 4]).unwrap();
        }
        let expected = recorder.runtime().replay_inspection();
        let state = &expected.simulation.as_ref().unwrap().state;
        assert_eq!(state.rules().version(), 8);
        assert!(!state.mobs().is_empty());
        let (_, journal) = recorder.into_parts();
        verify_exact(&journal, &expected).unwrap();
        assert_eq!(replay(&journal).unwrap(), expected);
    }
}

#[test]
fn atomic_base_cloud_and_lair_settings_replay_with_retained_disabled_values() {
    let initial = LairTuning::new(false, LairSettings::new(17, 4).unwrap());
    let mut recorder = recorder(BoardSize::ThirteenByEleven, initial);
    recorder.admit(SeatId::One, false).unwrap();
    recorder.admit(SeatId::Two, false).unwrap();
    let changed = LairTuning::new(false, LairSettings::new(29, 7).unwrap());
    let settings = RoomSettings::new(SpeedLevel::Two, 64, 2, 90)
        .unwrap()
        .with_enabled_items(EnabledItems::KICK);
    recorder
        .human_action(
            SeatId::One,
            1,
            ReplayAction::UpdateSettings {
                settings_revision: 1,
                settings,
                cloud: StormCloudConfig::Disabled,
                tuning: changed,
            },
        )
        .unwrap();
    for seat in SeatId::DUEL {
        recorder
            .human_action(
                seat,
                1,
                ReplayAction::Ready {
                    ready: true,
                    settings_revision: 2,
                },
            )
            .unwrap();
    }
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS + 30 {
        recorder.settle([None; 4]).unwrap();
    }
    let expected = recorder.runtime().replay_inspection();
    let (_, journal) = recorder.into_parts();
    assert_eq!(journal.header.configuration.lairs(), initial);
    verify_exact(&journal, &expected).unwrap();
    let actual = replay(&journal).unwrap();
    assert_eq!(actual.lifecycle.settings(), settings);
    assert_eq!(actual.environment, Some(StormCloudConfig::Disabled));
}

#[test]
fn replay7_header_is_closed_to_exact_profile_versions() {
    let recorder = recorder(
        BoardSize::FifteenByThirteen,
        LairTuning::new(true, LairSettings::DEFAULT),
    );
    let (_, journal) = recorder.into_parts();
    let mut malformed = journal.clone();
    malformed.header.format_version = 6;
    assert!(replay(&malformed).is_err());
    malformed = journal.clone();
    malformed.header.rules_version = 7;
    assert!(replay(&malformed).is_err());
    malformed = journal.clone();
    malformed.header.cpu_policy_version = 18;
    assert!(replay(&malformed).is_err());
    malformed = journal;
    malformed.header.lair_layout_version = 2;
    assert!(replay(&malformed).is_err());
}

#[test]
fn pending_departure_and_interrupt_are_recorded_as_one_exact_operation() {
    let mut recorder = recorder(
        BoardSize::ElevenByNine,
        LairTuning::new(true, LairSettings::DEFAULT),
    );
    start(&mut recorder, 1);
    recorder
        .interrupt_and_depart(
            grenadiers_core::room::InterruptReason::InternalError,
            [
                None,
                Some(grenadiers_core::room::DepartureReason::TransportLoss),
                None,
                None,
            ],
        )
        .unwrap();
    let expected = recorder.runtime().replay_inspection();
    let (_, journal) = recorder.into_parts();
    assert!(matches!(
        journal.records.last().unwrap().operation,
        super::record::Operation::InterruptAndDepart { .. }
    ));
    verify_exact(&journal, &expected).unwrap();
}

#[test]
fn replay7_rejects_source_order_marker_and_terminal_status_mutations() {
    let mut recorder = recorder(
        BoardSize::FifteenByThirteen,
        LairTuning::new(true, LairSettings::DEFAULT),
    );
    start(&mut recorder, 1);
    let (_, journal) = recorder.into_parts();

    let mut wrong_source = journal.clone();
    let action = wrong_source
        .records
        .iter_mut()
        .find(|record| matches!(record.operation, super::record::Operation::Action { .. }))
        .unwrap();
    let super::record::Operation::Action { source, .. } = &mut action.operation else {
        unreachable!()
    };
    *source = super::record::ActionSource::Cpu;
    assert!(replay(&wrong_source).is_err());

    let mut wrong_order = journal.clone();
    wrong_order.records[1].order += 1;
    assert!(replay(&wrong_order).is_err());

    let mut missing_marker = journal.clone();
    missing_marker.records.retain(|record| {
        !matches!(
            record.operation,
            super::record::Operation::RoundStarted { .. }
        )
    });
    assert!(replay(&missing_marker).is_err());

    let mut truncated = journal.clone();
    truncated.status = super::ReplayStatus::Truncated;
    assert!(replay(&truncated).is_err());
    let mut incomplete = journal;
    incomplete.status = super::ReplayStatus::IncompleteRoundConstruction;
    assert!(replay(&incomplete).is_err());
}

#[test]
fn replay7_debug_redacts_private_cpu_seed() {
    let provenance = CpuProvenance::new(
        SeatId::Two,
        grenadiers_core::room::CpuDifficulty::Standard,
        [173; 32],
    );
    let debug = format!("{provenance:?}");
    assert!(debug.contains("<redacted>"));
    assert!(!debug.contains("173"));
}

#[test]
fn replay7_preserves_archived_cpu19_and_records_cpu20_without_mixed_provenance() {
    let base = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu],
        Some(grenadiers_core::room::CpuDifficulty::Standard),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let mut recorder = RecordingRosterRuntime::new(
        LairRoomConfiguration::new(
            base,
            StormCloudConfig::Disabled,
            LairTuning::new(false, LairSettings::DEFAULT),
        ),
        RoomSettings::DEFAULT,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(37);
            Ok(())
        }),
        vec![CpuProvenance::new(
            SeatId::Two,
            grenadiers_core::room::CpuDifficulty::Standard,
            [173; 32],
        )],
    )
    .unwrap();
    recorder.admit(SeatId::One, false).unwrap();
    recorder
        .human_action(
            SeatId::One,
            1,
            ReplayAction::Ready {
                ready: true,
                settings_revision: 1,
            },
        )
        .unwrap();
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS + 30 {
        recorder.settle([None; 4]).unwrap();
    }
    recorder
        .cpu_action(SeatId::Two, 1, ReplayAction::Bomb(1))
        .unwrap();
    recorder.settle([None; 4]).unwrap();
    recorder
        .cpu_action(
            SeatId::Two,
            1,
            ReplayAction::Movement(Some(grenadiers_core::game::Direction::West)),
        )
        .unwrap();
    for _ in 0..12 {
        recorder.settle([None; 4]).unwrap();
    }
    let expected = recorder.runtime().replay_inspection();
    let (_, journal) = recorder.into_parts();
    assert_eq!(journal.header.cpu_policy_version, 20);
    assert_eq!(journal.header.cpus[0].policy_version, 20);
    verify_exact(&journal, &expected).unwrap();

    let mut archived = journal.clone();
    archived.header.cpu_policy_version = 19;
    archived.header.cpus[0].policy_version = 19;
    verify_exact(&archived, &expected).unwrap();
    for header_policy in [18, 19, 21] {
        let mut malformed = journal.clone();
        malformed.header.cpu_policy_version = header_policy;
        assert!(replay(&malformed).is_err());
    }
    let mut mixed = archived;
    mixed.header.cpus[0].policy_version = 20;
    assert!(replay(&mixed).is_err());
}

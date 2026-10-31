use grenadiers_core::{
    game::{Direction, EnabledItems, SeatId, StormCloudConfig},
    room::{
        BoardSize, CpuDifficulty, LairRoomConfiguration, RespawnLairSettings, RespawnLairTuning,
        RoomConfiguration, RoomSettings,
    },
};

use crate::round_seed::RoundSeedSource;

use super::{
    CpuProvenance, RecordingRosterRuntime, ReplayAction, ReplayHeader, replay, verify_exact,
};

fn configuration() -> LairRoomConfiguration {
    LairRoomConfiguration::new(
        RoomConfiguration::new(
            &[
                grenadiers_core::room::SeatControl::Human,
                grenadiers_core::room::SeatControl::Cpu,
            ],
            Some(CpuDifficulty::Standard),
            BoardSize::FifteenByThirteen,
        )
        .unwrap(),
        StormCloudConfig::Single,
        RespawnLairTuning::new(true, RespawnLairSettings::new(5).unwrap()).as_legacy(),
    )
}

fn recorder() -> RecordingRosterRuntime {
    RecordingRosterRuntime::new(
        configuration(),
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(4);
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

fn start(runtime: &mut RecordingRosterRuntime) {
    runtime.admit(SeatId::One, false).unwrap();
    runtime
        .human_action(
            SeatId::One,
            1,
            ReplayAction::Ready {
                ready: true,
                settings_revision: 1,
            },
        )
        .unwrap();
    for _ in 0..181 {
        runtime.settle([None; 4]).unwrap();
    }
}

#[test]
fn replay8_reconstructs_rules9_birth_and_accepted_cpu_inputs_exactly() {
    let mut recorder = recorder();
    start(&mut recorder);
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
    for _ in 0..310 {
        recorder.settle([None; 4]).unwrap();
    }
    let (runtime, journal) = recorder.into_parts();
    assert_eq!(
        runtime.respawn_tuning().unwrap().settings().spawn_seconds(),
        5
    );
    verify_exact(&journal, &runtime.replay_inspection()).unwrap();
}

#[test]
fn replay8_reconstructs_bomb_death_and_terminal_physics_exactly() {
    let mut recorder = recorder();
    start(&mut recorder);
    recorder
        .cpu_action(SeatId::Two, 1, ReplayAction::Bomb(1))
        .unwrap();
    for _ in 0..190 {
        recorder.settle([None; 4]).unwrap();
    }
    let (runtime, journal) = recorder.into_parts();
    let inspection = runtime.replay_inspection();
    assert!(
        !inspection
            .simulation
            .as_ref()
            .unwrap()
            .state
            .actor(SeatId::Two)
            .unwrap()
            .alive()
    );
    verify_exact(&journal, &inspection).unwrap();
}

#[test]
fn replay8_header_is_closed_to_rules9_supported_cpu_profiles_and_cap_one() {
    let mut header = ReplayHeader::new(
        configuration(),
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        [false; 4],
        vec![CpuProvenance::new(
            SeatId::Two,
            CpuDifficulty::Standard,
            [1; 32],
        )],
    )
    .unwrap();
    let empty = super::record::Journal::new(header.clone());
    assert!(replay(&empty).is_ok());

    header.rules_version = 8;
    assert!(super::validation::validate_header(&header).is_err());
    header.rules_version = 9;
    header.cpu_policy_version = 20;
    assert!(super::validation::validate_header(&header).is_err());

    let invalid = LairRoomConfiguration::new(
        *configuration().base(),
        StormCloudConfig::Single,
        grenadiers_core::room::LairTuning::new(true, grenadiers_core::room::LairSettings::DEFAULT),
    );
    assert!(
        ReplayHeader::new(
            invalid,
            RoomSettings::DEFAULT,
            [false; 4],
            vec![CpuProvenance::new(
                SeatId::Two,
                CpuDifficulty::Standard,
                [2; 32],
            )],
        )
        .is_err()
    );
}

#[test]
fn replay8_debug_redacts_private_cpu_seed() {
    let header = ReplayHeader::new(
        configuration(),
        RoomSettings::DEFAULT,
        [false; 4],
        vec![CpuProvenance::new(
            SeatId::Two,
            CpuDifficulty::Standard,
            [9; 32],
        )],
    )
    .unwrap();
    let debug = format!("{header:?}");
    assert!(debug.contains("<redacted>"));
    assert!(!debug.contains("9, 9, 9"));
}

#[test]
fn replay8_keeps_cpu21_and_cpu22_inputs_exact_without_mixed_provenance() {
    let mut runtime = recorder();
    start(&mut runtime);
    runtime
        .cpu_action(
            SeatId::Two,
            1,
            ReplayAction::Movement(Some(Direction::West)),
        )
        .unwrap();
    for _ in 0..12 {
        runtime.settle([None; 4]).unwrap();
    }
    runtime
        .cpu_action(SeatId::Two, 1, ReplayAction::Movement(None))
        .unwrap();
    runtime.settle([None; 4]).unwrap();
    let (runtime, journal) = runtime.into_parts();
    let expected = runtime.replay_inspection();
    assert_eq!(journal.header.cpu_policy_version, 22);
    assert_eq!(journal.header.cpus[0].policy_version, 22);
    verify_exact(&journal, &expected).unwrap();
    let mut archived = journal.clone();
    archived.header.cpu_policy_version = 21;
    archived.header.cpus[0].policy_version = 21;
    verify_exact(&archived, &expected).unwrap();
    for (header_version, provenance) in [(20, 20), (23, 23), (21, 22), (22, 21)] {
        let mut mixed = journal.clone();
        mixed.header.cpu_policy_version = header_version;
        mixed.header.cpus[0].policy_version = provenance;
        assert!(replay(&mixed).is_err());
    }
}

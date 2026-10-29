use grenadiers_core::{
    game::{Direction, SeatId, StormCloudConfig},
    room::{
        BoardSize, CpuDifficulty, EnvironmentRoomConfiguration, RoomConfiguration, RoomSettings,
        SeatControl,
    },
};

use crate::round_seed::RoundSeedSource;

use super::{CpuProvenance, Operation, RecordingRosterRuntime, ReplayAction, replay, verify_exact};

fn recorder(environment: StormCloudConfig) -> RecordingRosterRuntime {
    let base = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu],
        Some(CpuDifficulty::Standard),
        BoardSize::ElevenByNine,
    )
    .unwrap();
    RecordingRosterRuntime::new(
        EnvironmentRoomConfiguration::new(base, environment),
        RoomSettings::DEFAULT,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(73);
            Ok(())
        }),
        vec![CpuProvenance::new(
            SeatId::Two,
            CpuDifficulty::Standard,
            [151; 32],
        )],
    )
    .unwrap()
}

fn start(recorder: &mut RecordingRosterRuntime) {
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
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        recorder.settle([None; 4]).unwrap();
    }
}

#[test]
fn exact_replay_uses_cpu18_provenance_and_accepted_inputs_without_a_planner() {
    for environment in [StormCloudConfig::Disabled, StormCloudConfig::Single] {
        let mut recorder = recorder(environment);
        start(&mut recorder);
        let initial = recorder
            .runtime()
            .replay_inspection()
            .simulation
            .unwrap()
            .state
            .actors()[SeatId::Two.index()]
        .position();
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
        recorder.settle([None; 4]).unwrap();

        let inspection = recorder.runtime().replay_inspection();
        let state = &inspection.simulation.as_ref().unwrap().state;
        assert_ne!(state.actors()[SeatId::Two.index()].position(), initial);
        assert!(
            state
                .world()
                .bombs()
                .values()
                .any(|bomb| { bomb.owner() == state.actors()[SeatId::Two.index()].id() })
        );
        let (runtime, journal) = recorder.into_parts();
        assert!(journal.records.iter().any(|record| {
            matches!(
                record.operation,
                Operation::Action {
                    seat: SeatId::Two,
                    source: super::record::ActionSource::Cpu,
                    action: ReplayAction::Bomb(1),
                    ..
                }
            )
        }));
        verify_exact(&journal, &runtime.replay_inspection()).unwrap();
        assert_eq!(replay(&journal).unwrap(), inspection);
    }
}

#[test]
fn provenance_profile_and_action_source_mutations_are_rejected() {
    let mut recorder = recorder(StormCloudConfig::Single);
    start(&mut recorder);
    recorder
        .cpu_action(SeatId::Two, 1, ReplayAction::Movement(None))
        .unwrap();
    recorder.settle([None; 4]).unwrap();
    let (_, journal) = recorder.into_parts();

    let mut wrong_policy = journal.clone();
    wrong_policy.header.cpus[0].policy_version = 17;
    assert!(replay(&wrong_policy).is_err());

    let mut wrong_profile = journal.clone();
    wrong_profile.header.cpus[0].difficulty = CpuDifficulty::Easy;
    assert!(replay(&wrong_profile).is_err());

    let mut wrong_seat = journal.clone();
    wrong_seat.header.cpus[0].seat = SeatId::One;
    assert!(replay(&wrong_seat).is_err());

    let mut wrong_source = journal;
    let action = wrong_source
        .records
        .iter_mut()
        .find(|record| {
            matches!(
                record.operation,
                Operation::Action {
                    seat: SeatId::Two,
                    source: super::record::ActionSource::Cpu,
                    ..
                }
            )
        })
        .unwrap();
    let Operation::Action {
        seat,
        epoch,
        action: replay_action,
        ..
    } = action.operation
    else {
        unreachable!()
    };
    action.operation = Operation::Action {
        seat,
        epoch,
        source: super::record::ActionSource::Human,
        action: replay_action,
    };
    assert!(replay(&wrong_source).is_err());
}

#[test]
fn rejected_stale_and_locked_commands_leave_full_private_state_and_journal_unchanged() {
    let mut recorder = recorder(StormCloudConfig::Single);
    recorder.admit(SeatId::One, false).unwrap();
    let before_stale = recorder.runtime().replay_inspection();
    let records_before_stale = recorder.journal().records.len();
    assert!(
        recorder
            .human_action(
                SeatId::One,
                1,
                ReplayAction::UpdateSettings {
                    settings_revision: 2,
                    settings: RoomSettings::DEFAULT,
                    environment: StormCloudConfig::Disabled,
                },
            )
            .is_err()
    );
    assert_eq!(recorder.runtime().replay_inspection(), before_stale);
    assert_eq!(recorder.journal().records.len(), records_before_stale);

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
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        recorder.settle([None; 4]).unwrap();
    }
    let before_locked = recorder.runtime().replay_inspection();
    let records_before_locked = recorder.journal().records.len();
    assert!(
        recorder
            .human_action(
                SeatId::One,
                1,
                ReplayAction::UpdateSettings {
                    settings_revision: 1,
                    settings: RoomSettings::DEFAULT,
                    environment: StormCloudConfig::Disabled,
                },
            )
            .is_err()
    );
    assert_eq!(recorder.runtime().replay_inspection(), before_locked);
    assert_eq!(recorder.journal().records.len(), records_before_locked);

    assert!(
        recorder
            .cpu_action(
                SeatId::Two,
                2,
                ReplayAction::Movement(Some(Direction::East))
            )
            .is_err()
    );
    assert_eq!(recorder.runtime().replay_inspection(), before_locked);
    assert_eq!(recorder.journal().records.len(), records_before_locked);
}

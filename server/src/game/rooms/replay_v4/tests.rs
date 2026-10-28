use grenadiers_core::{
    game::{Direction, EnabledItems, SeatId, SpeedLevel},
    room::{BoardSize, CpuDifficulty, RoomConfiguration, RoomSettings, SeatControl},
};

use crate::round_seed::RoundSeedSource;

use super::{
    CpuProvenance, RecordingRosterRuntime, ReplayAction, ReplayError, ReplayStatus, replay,
    verify_exact,
};

fn recorder(controls: &[SeatControl]) -> RecordingRosterRuntime {
    let difficulty = controls
        .contains(&SeatControl::Cpu)
        .then_some(CpuDifficulty::Standard);
    recorder_with_difficulty(controls, difficulty)
}

fn recorder_with_difficulty(
    controls: &[SeatControl],
    difficulty: Option<CpuDifficulty>,
) -> RecordingRosterRuntime {
    let configuration =
        RoomConfiguration::new(controls, difficulty, BoardSize::FifteenByThirteen).unwrap();
    let cpus = configuration
        .cpu_seats()
        .map(|seat| CpuProvenance::new(seat, difficulty.unwrap(), [seat.index() as u8 + 1; 32]))
        .collect();
    RecordingRosterRuntime::new(
        configuration,
        RoomSettings::DEFAULT,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(9);
            Ok(())
        }),
        cpus,
    )
    .unwrap()
}

#[test]
fn replay_four_exactly_reconstructs_two_three_and_four_player_rounds() {
    for count in 2..=4 {
        let controls = vec![SeatControl::Human; count];
        let mut recorder = recorder(&controls);
        for &seat in &SeatId::ALL[..count] {
            recorder.admit(seat, false).unwrap();
            recorder
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
        for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS + 3 {
            recorder.settle([None; 4]).unwrap();
        }
        let (runtime, journal) = recorder.into_parts();
        verify_exact(&journal, &runtime.replay_inspection()).unwrap();
        assert!(journal.allocated_bytes() < 4 * 1024 * 1024);
    }
}

#[test]
fn replay_uses_accepted_cpu_actions_without_invoking_a_policy() {
    for difficulty in [CpuDifficulty::Easy, CpuDifficulty::Standard] {
        let mut recorder = recorder_with_difficulty(
            &[SeatControl::Human, SeatControl::Cpu, SeatControl::Human],
            Some(difficulty),
        );
        for seat in [SeatId::One, SeatId::Three] {
            recorder.admit(seat, false).unwrap();
            recorder
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
        for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
            recorder.settle([None; 4]).unwrap();
        }
        recorder
            .cpu_action(
                SeatId::Two,
                1,
                ReplayAction::Movement(Some(grenadiers_core::game::Direction::West)),
            )
            .unwrap();
        recorder.settle([None; 4]).unwrap();
        let (runtime, journal) = recorder.into_parts();
        verify_exact(&journal, &runtime.replay_inspection()).unwrap();
        assert!(!format!("{journal:?}").contains("[2, 2, 2"));
    }
}

#[test]
fn malformed_provenance_small_boards_and_private_debug_are_rejected_or_redacted() {
    let configuration = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu],
        Some(CpuDifficulty::Easy),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    assert!(
        super::ReplayV4Header::new(
            configuration,
            RoomSettings::DEFAULT,
            [false; 4],
            vec![CpuProvenance::new(
                SeatId::Two,
                CpuDifficulty::Standard,
                [0xa5; 32],
            )],
        )
        .is_err()
    );
    let small = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Human],
        None,
        BoardSize::ThirteenByEleven,
    )
    .unwrap();
    assert!(super::ReplayV4Header::new(small, RoomSettings::DEFAULT, [false; 4], vec![]).is_err());

    let journal = recorder(&[SeatControl::Human, SeatControl::Cpu]);
    let debug = format!("{:?}", journal.journal());
    assert!(debug.contains("<redacted>"));
    assert!(!debug.contains("165, 165"));
}

#[test]
fn missing_extra_truncated_and_incomplete_archives_are_rejected() {
    let mut recorder = recorder(&[SeatControl::Human, SeatControl::Human]);
    for seat in [SeatId::One, SeatId::Two] {
        recorder.admit(seat, false).unwrap();
        recorder
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
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        recorder.settle([None; 4]).unwrap();
    }
    let (_, journal) = recorder.into_parts();
    let marker = journal.records.last().copied().unwrap();

    let mut missing = journal.clone();
    missing.records.pop();
    assert!(matches!(
        replay(&missing),
        Err(ReplayError::Runtime | ReplayError::MissingRoundMarker)
    ));

    let mut extra = journal.clone();
    extra.records.push(super::record::Record {
        order: marker.order + 1,
        ..marker
    });
    assert_eq!(replay(&extra), Err(ReplayError::ExtraRoundMarker));

    let mut truncated = journal.clone();
    truncated.status = ReplayStatus::Truncated;
    assert_eq!(replay(&truncated), Err(ReplayError::Truncated));
    truncated.status = ReplayStatus::IncompleteRoundConstruction;
    assert_eq!(replay(&truncated), Err(ReplayError::Incomplete));
}

#[test]
fn teleporter_random_state_is_exercised_and_replayed_exactly() {
    let settings = RoomSettings::new(SpeedLevel::Base, 256, 3, 180)
        .unwrap()
        .with_enabled_items(EnabledItems::TELEPORTER);
    let configuration = RoomConfiguration::human_duel();
    let mut recorder = RecordingRosterRuntime::new(
        configuration,
        settings,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(31);
            Ok(())
        }),
        vec![],
    )
    .unwrap();
    for seat in [SeatId::One, SeatId::Two] {
        recorder.admit(seat, false).unwrap();
        recorder
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
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        recorder.settle([None; 4]).unwrap();
    }
    move_cells(&mut recorder, Direction::East, 1);
    recorder
        .human_action(SeatId::One, 1, ReplayAction::Bomb(1))
        .unwrap();
    recorder.settle([None; 4]).unwrap();
    move_cells(&mut recorder, Direction::West, 1);
    move_cells(&mut recorder, Direction::South, 1);
    for _ in 0..240 {
        let state = &recorder
            .runtime()
            .replay_inspection()
            .simulation
            .unwrap()
            .state;
        if state.world().bombs().is_empty() && state.world().hazards().is_empty() {
            break;
        }
        recorder.settle([None; 4]).unwrap();
    }
    move_cells(&mut recorder, Direction::North, 1);
    move_cells(&mut recorder, Direction::East, 2);

    let inspection = recorder.runtime().replay_inspection();
    let state = &inspection.simulation.as_ref().unwrap().state;
    assert!(
        state
            .actors()
            .iter()
            .any(|actor| actor.last_teleport_tick().is_some())
    );
    let (_, journal) = recorder.into_parts();
    verify_exact(&journal, &inspection).unwrap();
}

#[test]
fn cancellation_departure_resume_and_input_fence_replay_in_order() {
    let mut recorder = recorder(&[SeatControl::Human; 3]);
    for &seat in &SeatId::ALL[..3] {
        recorder.admit(seat, false).unwrap();
        recorder
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
    recorder.settle([None; 4]).unwrap();
    recorder
        .human_action(
            SeatId::Three,
            1,
            ReplayAction::Ready {
                ready: false,
                settings_revision: 1,
            },
        )
        .unwrap();
    let epoch = recorder
        .runtime()
        .replay_inspection()
        .lifecycle
        .round_attempt();
    assert!(epoch > 1);
    recorder
        .human_action(
            SeatId::Three,
            epoch,
            ReplayAction::Ready {
                ready: true,
                settings_revision: 1,
            },
        )
        .unwrap();
    recorder
        .settle([
            None,
            Some(grenadiers_core::room::DepartureReason::TransportLoss),
            None,
            None,
        ])
        .unwrap();
    recorder.input_fence(SeatId::Two);
    recorder.admit(SeatId::Two, true).unwrap();
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        recorder.settle([None; 4]).unwrap();
    }
    let (runtime, journal) = recorder.into_parts();
    verify_exact(&journal, &runtime.replay_inspection()).unwrap();
}

fn move_cells(recorder: &mut RecordingRosterRuntime, direction: Direction, cells: usize) {
    recorder
        .human_action(SeatId::One, 1, ReplayAction::Movement(Some(direction)))
        .unwrap();
    for update in 0..cells * 20 {
        if update > 0 && update.is_multiple_of(10) {
            recorder
                .human_action(SeatId::One, 1, ReplayAction::Movement(Some(direction)))
                .unwrap();
        }
        recorder.settle([None; 4]).unwrap();
    }
    recorder
        .human_action(SeatId::One, 1, ReplayAction::Movement(None))
        .unwrap();
    recorder.settle([None; 4]).unwrap();
}

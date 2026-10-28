use grenadiers_core::{
    game::{Direction, EnabledItems, SeatId, SpeedLevel, VARIABLE_GENERATOR_VERSION},
    room::{
        BoardSize, CpuDifficulty, InterruptReason, MatchPhase, RoomConfiguration, RoomSettings,
        SeatControl,
    },
};

use crate::round_seed::RoundSeedSource;

use super::{
    CpuProvenance, RecordingRosterRuntime, ReplayAction, ReplayError, replay, replay_v5,
    verify_exact_v5,
};

fn recorder(
    board: BoardSize,
    controls: &[SeatControl],
    settings: RoomSettings,
    seed: u8,
) -> RecordingRosterRuntime {
    let difficulty = controls
        .contains(&SeatControl::Cpu)
        .then_some(CpuDifficulty::Standard);
    let configuration = RoomConfiguration::new(controls, difficulty, board).unwrap();
    let cpus = configuration
        .cpu_seats()
        .map(|seat| CpuProvenance::new(seat, difficulty.unwrap(), [seat.index() as u8 + 1; 32]))
        .collect();
    RecordingRosterRuntime::new_v5(
        configuration,
        settings,
        RoundSeedSource::with_entropy(move |bytes| {
            bytes.fill(seed);
            Ok(())
        }),
        cpus,
    )
    .unwrap()
}

fn ready_humans(recorder: &mut RecordingRosterRuntime) {
    let configuration = recorder
        .runtime()
        .replay_inspection()
        .lifecycle
        .configuration();
    for seat in configuration.human_seats() {
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
}

fn start_round(recorder: &mut RecordingRosterRuntime) {
    ready_humans(recorder);
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        recorder.settle([None; 4]).unwrap();
    }
}

#[test]
fn replay_five_exactly_reconstructs_both_small_boards_and_all_roster_sizes() {
    for board in [BoardSize::ElevenByNine, BoardSize::ThirteenByEleven] {
        for count in 2..=4 {
            let mut recorder = recorder(
                board,
                &vec![SeatControl::Human; count],
                RoomSettings::DEFAULT,
                count as u8,
            );
            start_round(&mut recorder);
            for _ in 0..3 {
                recorder.settle([None; 4]).unwrap();
            }
            let inspection = recorder.runtime().replay_inspection();
            let simulation = inspection.simulation.as_ref().unwrap();
            assert_eq!(simulation.state.world().arena().board(), board);
            assert_eq!(simulation.generation.version(), VARIABLE_GENERATOR_VERSION);
            let (_, journal) = recorder.into_parts();
            verify_exact_v5(&journal, &inspection).unwrap();
            assert_eq!(replay(&journal), Err(ReplayError::InvalidHeader));
            assert!(journal.allocated_bytes() < 4 * 1024 * 1024);
        }
    }
}

#[test]
fn replay_five_replays_accepted_cpu_input_without_running_a_policy() {
    let mut recorder = recorder(
        BoardSize::ElevenByNine,
        &[SeatControl::Human, SeatControl::Cpu, SeatControl::Human],
        RoomSettings::DEFAULT,
        17,
    );
    start_round(&mut recorder);
    recorder
        .cpu_action(
            SeatId::Two,
            1,
            ReplayAction::Movement(Some(Direction::West)),
        )
        .unwrap();
    recorder.settle([None; 4]).unwrap();
    let inspection = recorder.runtime().replay_inspection();
    let (_, journal) = recorder.into_parts();
    verify_exact_v5(&journal, &inspection).unwrap();
}

#[test]
fn replay_five_preserves_teleporter_randomness_and_rematch_state() {
    let settings = RoomSettings::new(SpeedLevel::Base, 256, 3, 180)
        .unwrap()
        .with_enabled_items(EnabledItems::TELEPORTER);
    let mut recorder = recorder(
        BoardSize::ThirteenByEleven,
        &[SeatControl::Human, SeatControl::Human],
        settings,
        31,
    );
    start_round(&mut recorder);
    move_cells(&mut recorder, Direction::East, 1);
    recorder
        .human_action(SeatId::One, 1, ReplayAction::Bomb(1))
        .unwrap();
    recorder.settle([None; 4]).unwrap();
    move_cells(&mut recorder, Direction::West, 1);
    move_cells(&mut recorder, Direction::South, 1);
    for _ in 0..240 {
        let simulation = recorder.runtime().replay_inspection().simulation.unwrap();
        if simulation.state.world().bombs().is_empty()
            && simulation.state.world().hazards().is_empty()
        {
            break;
        }
        recorder.settle([None; 4]).unwrap();
    }
    move_cells(&mut recorder, Direction::North, 1);
    move_cells(&mut recorder, Direction::East, 2);
    assert!(
        recorder
            .runtime()
            .replay_inspection()
            .simulation
            .as_ref()
            .unwrap()
            .state
            .actors()
            .iter()
            .any(|actor| actor.last_teleport_tick().is_some())
    );

    recorder.interrupt(InterruptReason::ServerStopped).unwrap();
    for _ in 0..grenadiers_core::room::series::RESULT_HOLD_TICKS {
        recorder.settle([None; 4]).unwrap();
    }
    for seat in [SeatId::One, SeatId::Two] {
        recorder
            .human_action(seat, 1, ReplayAction::Rematch(true))
            .unwrap();
    }
    assert_eq!(
        recorder.runtime().replay_inspection().lifecycle.phase(),
        MatchPhase::Waiting
    );
    let inspection = recorder.runtime().replay_inspection();
    let (_, journal) = recorder.into_parts();
    assert!(replay_v5(&journal).is_ok());
    verify_exact_v5(&journal, &inspection).unwrap();
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

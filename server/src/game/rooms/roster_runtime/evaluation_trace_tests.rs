use std::sync::Arc;

use grenadiers_core::{
    game::{Direction, MobNavigationPolicy, SeatId, SimulationSeed, StormCloudConfig},
    room::{
        BoardSize, CpuDifficulty, RespawnLairTuning, RoomConfiguration, RoomSettings, SeatControl,
        SpikePitTuning,
    },
};

use crate::round_seed::RoundSeedSource;

use super::{
    EvaluationOperation, EvaluationTraceError, RosterAction, RosterRuntime, RosterRuntimeError,
    SpikeRuntimeEnvironment, tests,
};

#[test]
fn accepted_operations_are_ordered_and_trace_does_not_change_runtime_state() {
    let mut traced = tests::runtime(2);
    tests::start(&mut traced, 2);
    let mut untraced = traced.clone();
    let tick = traced.lifecycle.tick();

    assert_eq!(
        traced.drain_evaluation_trace_for_test(),
        Err(EvaluationTraceError::Disabled)
    );
    let before = traced.replay_inspection();
    traced.enable_evaluation_trace_for_test();
    assert_eq!(traced.replay_inspection(), before);

    for runtime in [&mut traced, &mut untraced] {
        runtime
            .action(
                SeatId::One,
                1,
                RosterAction::Movement(Some(Direction::East)),
            )
            .unwrap();
        runtime
            .action(
                SeatId::One,
                1,
                RosterAction::Movement(Some(Direction::East)),
            )
            .unwrap();
        runtime
            .action(SeatId::One, 1, RosterAction::Bomb(17))
            .unwrap();
        assert_eq!(
            runtime.action(SeatId::One, 1, RosterAction::Bomb(17)),
            Err(RosterRuntimeError::DuplicateAction)
        );
        runtime.stop_seat_inputs(SeatId::One);
        runtime.stop_seat_inputs(SeatId::One);
        runtime.settle_boundary([None; 4]).unwrap();
        runtime.current_notice().unwrap();
    }

    assert_eq!(traced.replay_inspection(), untraced.replay_inspection());
    assert_eq!(
        traced.drain_evaluation_trace_for_test().unwrap(),
        [
            EvaluationOperation::HumanAction {
                seat: SeatId::One,
                epoch: 1,
                action: RosterAction::Movement(Some(Direction::East)),
            },
            EvaluationOperation::HumanAction {
                seat: SeatId::One,
                epoch: 1,
                action: RosterAction::Movement(Some(Direction::East)),
            },
            EvaluationOperation::HumanAction {
                seat: SeatId::One,
                epoch: 1,
                action: RosterAction::Bomb(17),
            },
            EvaluationOperation::InputFence(SeatId::One),
            EvaluationOperation::InputFence(SeatId::One),
            EvaluationOperation::Settle {
                tick,
                departures: [None; 4],
            },
            EvaluationOperation::PublicationFlush,
        ]
    );
    assert!(traced.drain_evaluation_trace_for_test().unwrap().is_empty());
}

#[test]
fn cpu_actions_are_distinct_and_overflow_is_latched() {
    let configuration = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu, SeatControl::Cpu],
        Some(CpuDifficulty::Hard),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let mut runtime = RosterRuntime::new_spike_v21_pickup_with_navigation(
        tests::identity(configuration),
        RoomSettings::DEFAULT,
        SpikeRuntimeEnvironment {
            cloud: StormCloudConfig::Single,
            lairs: RespawnLairTuning::DEFAULT,
            spike_pits: SpikePitTuning::DEFAULT,
            navigation: MobNavigationPolicy::TimedEscapeV3,
        },
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(4);
            Ok(())
        }),
        Arc::new(|_| Ok(SimulationSeed::new([6; 32]))),
    )
    .unwrap();
    let prepared = runtime.prepare_join(SeatId::One).unwrap();
    runtime.commit_admission(prepared);
    runtime
        .action(
            SeatId::One,
            1,
            RosterAction::Ready {
                ready: true,
                settings_revision: 1,
            },
        )
        .unwrap();
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }

    runtime.enable_evaluation_trace_for_test();
    runtime
        .submit_cpu_intent(
            SeatId::Two,
            1,
            RosterAction::Movement(Some(Direction::West)),
        )
        .unwrap();
    assert_eq!(
        runtime.drain_evaluation_trace_for_test().unwrap(),
        [EvaluationOperation::CpuAction {
            seat: SeatId::Two,
            epoch: 1,
            action: RosterAction::Movement(Some(Direction::West)),
        }]
    );

    let mut untraced = runtime.clone();
    untraced.evaluation_trace = None;
    for _ in 0..=128 {
        runtime.stop_seat_inputs(SeatId::Two);
        untraced.stop_seat_inputs_internal(SeatId::Two);
    }
    assert_eq!(runtime.replay_inspection(), untraced.replay_inspection());
    assert_eq!(
        runtime.drain_evaluation_trace_for_test(),
        Err(EvaluationTraceError::Overflow)
    );
    runtime.enable_evaluation_trace_for_test();
    assert_eq!(
        runtime.drain_evaluation_trace_for_test(),
        Err(EvaluationTraceError::Overflow)
    );
}

#[test]
fn replacement_admission_preserves_state_and_rejects_partial_trace() {
    let mut traced = tests::runtime(2);
    tests::start(&mut traced, 2);
    let mut untraced = traced.clone();
    traced.enable_evaluation_trace_for_test();

    for runtime in [&mut traced, &mut untraced] {
        runtime
            .action(
                SeatId::One,
                1,
                RosterAction::Movement(Some(Direction::South)),
            )
            .unwrap();
        let replacement = runtime.prepare_resume(SeatId::One).unwrap();
        assert!(replacement.is_replacement);
        runtime.commit_admission(replacement);
    }

    assert_eq!(traced.replay_inspection(), untraced.replay_inspection());
    assert_eq!(
        traced.drain_evaluation_trace_for_test(),
        Err(EvaluationTraceError::UnsupportedOperation)
    );
    traced.enable_evaluation_trace_for_test();
    assert_eq!(
        traced.drain_evaluation_trace_for_test(),
        Err(EvaluationTraceError::UnsupportedOperation)
    );
}

#[test]
fn new_admission_preserves_state_and_rejects_partial_trace() {
    let mut traced = tests::runtime(2);
    let mut untraced = traced.clone();
    traced.enable_evaluation_trace_for_test();

    for runtime in [&mut traced, &mut untraced] {
        let admission = runtime.prepare_join(SeatId::One).unwrap();
        assert!(!admission.is_replacement);
        runtime.commit_admission(admission);
    }

    assert_eq!(traced.replay_inspection(), untraced.replay_inspection());
    assert_eq!(
        traced.drain_evaluation_trace_for_test(),
        Err(EvaluationTraceError::UnsupportedOperation)
    );
}

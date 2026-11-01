use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::UtilityMemory;
use crate::cpu::routing::{CONTINUATION_RENEWAL_ALLOWANCE, ContinuationCertificate};
use crate::cpu::{DecisionId, MemoryKey, MovementIntent, Observation, WorkBudget};
use crate::game::{Direction, InputAction, SeatId, SeatInput, Tick};

use super::continuation_execution::{
    CommittedContinuation, ContinuationCommand, ContinuationPublication, ContinuationStatus,
};
use super::continuation_pending::PendingContinuation;
use super::continuation_scene_fixture::{ContinuationScene, no_lair_scene, scene};
use super::continuation_selection::{OriginRecoveryChoice, plan_lairs};
use super::continuation_test_fixture::{CPU, observe, observe_for, step_until};
use super::{PlantEscapeOrigin, WORK_ALLOWANCE};

fn certificate(
    basis: &Observation,
    key: MemoryKey,
    origin: &PlantEscapeOrigin,
) -> Box<ContinuationCertificate> {
    let choice = plan_lairs(
        basis,
        &ObservedHazardMemory::new(key),
        basis.parts().tick,
        &UtilityMemory::new(key),
        origin,
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    let OriginRecoveryChoice::Continuation(certificate) = choice else {
        panic!("fixture must select continuation")
    };
    certificate
}

fn publication(
    decision: DecisionId,
    key: MemoryKey,
    sequence: u64,
    tick: Tick,
) -> ContinuationPublication {
    ContinuationPublication::new(decision, key, sequence, tick).unwrap()
}

fn input(seat: SeatId, command: &ContinuationCommand) -> SeatInput {
    SeatInput {
        seat,
        action: match command.movement() {
            MovementIntent::Direction(direction) => InputAction::Press(direction),
            MovementIntent::Stop => InputAction::Clear,
        },
    }
}

fn next_command(
    executor: &mut CommittedContinuation,
    observation: &Observation,
    decision: DecisionId,
    key: MemoryKey,
    sequence: &mut u64,
) -> ContinuationCommand {
    match executor.poll_command(observation.parts().tick, decision.command_epoch) {
        ContinuationStatus::Submit(command) => {
            assert!(executor.commit_command(command.clone()));
            command
        }
        ContinuationStatus::RenewalRequired => {
            *sequence += 1;
            let mut prepared = executor
                .prepare_grant(
                    observation,
                    publication(decision, key, *sequence, observation.parts().tick),
                    &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
                )
                .unwrap();
            let command = prepared.command().unwrap();
            assert!(executor.commit_grant(prepared, command.clone()));
            command
        }
        status => panic!("unexpected continuation status: {status:?}"),
    }
}

#[test]
fn grants_cannot_splice_between_mirrored_episodes_at_a_stop_boundary() {
    assert_mirrored_authority(scene());
}

#[test]
fn no_lair_grants_keep_independent_mirrored_authority() {
    assert_mirrored_authority(no_lair_scene());
}

fn assert_mirrored_authority(
    ContinuationScene {
        mut state,
        settings,
        basis,
        rival_basis,
        origin,
        rival_origin,
        key,
        rival_key,
    }: ContinuationScene,
) {
    let decision = DecisionId {
        world: basis.parts().world,
        command_epoch: 4,
        observation_tick: basis.parts().tick,
        checked_request_serial: 9,
    };
    let mut cpu_pending =
        PendingContinuation::new(certificate(&basis, key, &origin), decision, key, 28).unwrap();
    let mut rival_pending = PendingContinuation::new(
        certificate(&rival_basis, rival_key, &rival_origin),
        decision,
        rival_key,
        28,
    )
    .unwrap();
    step_until(&mut state, Tick(199));
    for (pending, observation, identity) in [
        (
            &mut cpu_pending,
            observe(&state, settings),
            publication(decision, key, 29, Tick(199)),
        ),
        (
            &mut rival_pending,
            observe_for(&state, settings, SeatId::One),
            publication(decision, rival_key, 29, Tick(199)),
        ),
    ] {
        assert!(
            pending
                .observe(
                    &observation,
                    identity,
                    &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
                )
                .unwrap()
                .is_none()
        );
    }
    step_until(&mut state, Tick(202));
    let cpu = admit(&mut cpu_pending, observe(&state, settings), decision, key);
    let rival = admit(
        &mut rival_pending,
        observe_for(&state, settings, SeatId::One),
        decision,
        rival_key,
    );
    let mut cpu_command = cpu.1;
    let mut rival_command = rival.1;
    let mut cpu_executor = cpu.0;
    let mut rival_executor = rival.0;
    let mut cpu_sequence = 30;
    let mut rival_sequence = 30;
    while state.tick() < Tick(244) {
        state
            .step(&[input(CPU, &cpu_command), input(SeatId::One, &rival_command)])
            .unwrap();
        if state.tick() == Tick(244) {
            break;
        }
        cpu_command = next_command(
            &mut cpu_executor,
            &observe(&state, settings),
            decision,
            key,
            &mut cpu_sequence,
        );
        rival_command = next_command(
            &mut rival_executor,
            &observe_for(&state, settings, SeatId::One),
            decision,
            rival_key,
            &mut rival_sequence,
        );
    }

    let mut cpu_splice = cpu_executor.fork_for_test();
    let mut rival_splice = rival_executor.fork_for_test();
    let cpu_prepared = cpu_splice
        .prepare_grant(
            &observe(&state, settings),
            publication(decision, key, cpu_sequence + 1, Tick(244)),
            &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
        )
        .unwrap();
    let mut rival_prepared = rival_splice
        .prepare_grant(
            &observe_for(&state, settings, SeatId::One),
            publication(decision, rival_key, rival_sequence + 1, Tick(244)),
            &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
        )
        .unwrap();
    assert_eq!(
        cpu_prepared.movements_for_test(),
        Some([
            MovementIntent::Stop,
            MovementIntent::Direction(Direction::East),
            MovementIntent::Direction(Direction::East),
        ])
    );
    assert_eq!(
        rival_prepared.movements_for_test(),
        Some([
            MovementIntent::Stop,
            MovementIntent::Direction(Direction::West),
            MovementIntent::Direction(Direction::West),
        ])
    );
    let foreign = rival_prepared.command().unwrap();
    assert!(!cpu_splice.commit_grant(rival_prepared, foreign));
}

fn admit(
    pending: &mut PendingContinuation,
    observation: Observation,
    decision: DecisionId,
    key: MemoryKey,
) -> (CommittedContinuation, ContinuationCommand) {
    let mut prepared = pending
        .observe(
            &observation,
            publication(decision, key, 30, Tick(202)),
            &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
        )
        .unwrap()
        .unwrap();
    let command = prepared.command().unwrap();
    let executor = prepared.commit(command.clone()).unwrap();
    (executor, command)
}

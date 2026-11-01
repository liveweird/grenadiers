use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::UtilityMemory;
use crate::cpu::routing::{CONTINUATION_RENEWAL_ALLOWANCE, ContinuationCertificate};
use crate::cpu::{DecisionId, MemoryKey, MovementIntent, Observation, WorkBudget};
use crate::game::{Direction, InputAction, SeatId, SeatInput, Tick};

use super::continuation_execution::{ContinuationPublication, ContinuationStatus};
use super::continuation_execution_guard_tests::assert_renewal_bound_and_rejections;
use super::continuation_pending::PendingContinuation;
use super::continuation_scene_fixture::{ContinuationScene, no_lair_scene, scene};
use super::continuation_selection::{OriginRecoveryChoice, plan_lairs};
use super::continuation_test_fixture::{CPU, observe, step_until};
use super::{PlantEscapeOrigin, WORK_ALLOWANCE};

fn build_certificate(
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
    .expect("continuation selector");
    let OriginRecoveryChoice::Continuation(certificate) = choice else {
        panic!("fixture must fit the continuation reservation")
    };
    certificate
}

fn publication(decision: DecisionId, sequence: u64, tick: Tick) -> ContinuationPublication {
    ContinuationPublication {
        decision,
        key: MemoryKey {
            world: decision.world,
            cpu_seat: CPU,
        },
        sequence,
        tick,
    }
}

fn input(seat: SeatId, movement: MovementIntent) -> SeatInput {
    SeatInput {
        seat,
        action: match movement {
            MovementIntent::Direction(direction) => InputAction::Press(direction),
            MovementIntent::Stop => InputAction::Clear,
        },
    }
}

#[test]
fn charged_grants_execute_the_full_physical_continuation() {
    execute_scene(scene());
}

#[test]
fn no_lair_grants_execute_the_full_physical_continuation() {
    execute_scene(no_lair_scene());
}

#[test]
fn continuation_contact_rejects_lair_mode_changes() {
    let mut disabled = no_lair_scene();
    let mut enabled = scene();
    let decision = DecisionId {
        world: disabled.basis.parts().world,
        command_epoch: 4,
        observation_tick: disabled.basis.parts().tick,
        checked_request_serial: 9,
    };
    let certificate = build_certificate(&disabled.basis, disabled.key, &disabled.origin);
    let mut pending = PendingContinuation::new(certificate, decision, disabled.key, 28).unwrap();
    step_until(&mut disabled.state, Tick(199));
    pending
        .observe(
            &observe(&disabled.state, disabled.settings),
            publication(decision, 29, Tick(199)),
            &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
        )
        .unwrap();
    step_until(&mut disabled.state, Tick(202));
    step_until(&mut enabled.state, Tick(202));
    assert!(
        pending
            .observe(
                &observe(&enabled.state, enabled.settings),
                publication(decision, 30, Tick(202)),
                &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
            )
            .is_err()
    );
}

#[test]
fn no_lair_renewal_rejects_an_enabled_overlay() {
    let mut disabled = no_lair_scene();
    let mut enabled = scene();
    let decision = DecisionId {
        world: disabled.basis.parts().world,
        command_epoch: 4,
        observation_tick: disabled.basis.parts().tick,
        checked_request_serial: 9,
    };
    let certificate = build_certificate(&disabled.basis, disabled.key, &disabled.origin);
    let rival = build_certificate(
        &disabled.rival_basis,
        disabled.rival_key,
        &disabled.rival_origin,
    );
    let rival_checkpoints = rival.checkpoints_for_test();
    let mut pending = PendingContinuation::new(certificate, decision, disabled.key, 28).unwrap();
    for (tick, sequence) in [(Tick(199), 29), (Tick(202), 30)] {
        step_until(&mut disabled.state, tick);
        step_until(&mut enabled.state, tick);
        let prepared = pending
            .observe(
                &observe(&disabled.state, disabled.settings),
                publication(decision, sequence, tick),
                &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
            )
            .unwrap();
        if tick == Tick(199) {
            assert!(prepared.is_none());
            continue;
        }
        let prepared = prepared.unwrap();
        let first = prepared.command();
        let mut executor = prepared.commit(first.clone()).unwrap();
        let mut next = first;
        while disabled.state.tick() < Tick(205) {
            let update = Tick(disabled.state.tick().0 + 1);
            let rival = rival_checkpoints
                .iter()
                .find(|checkpoint| checkpoint.tick == update)
                .map_or(MovementIntent::Stop, |checkpoint| checkpoint.movement);
            let inputs = [input(CPU, next.movement()), input(SeatId::One, rival)];
            disabled.state.step(&inputs).unwrap();
            enabled.state.step(&inputs).unwrap();
            if disabled.state.tick() < Tick(205) {
                let ContinuationStatus::Submit(command) =
                    executor.poll_command(disabled.state.tick(), decision.command_epoch)
                else {
                    panic!("next initial grant input must be due")
                };
                assert!(executor.commit_command(command.clone()));
                next = command;
            }
        }
        assert!(
            executor
                .prepare_grant(
                    &observe(&enabled.state, enabled.settings),
                    publication(decision, 31, Tick(205)),
                    &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
                )
                .is_err()
        );
    }
}

fn execute_scene(
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
    let certificate = build_certificate(&basis, key, &origin);
    let rival = build_certificate(&rival_basis, rival_key, &rival_origin);
    let rival_checkpoints = rival.checkpoints_for_test();
    let decision = DecisionId {
        world: basis.parts().world,
        command_epoch: 4,
        observation_tick: basis.parts().tick,
        checked_request_serial: 9,
    };
    let mut pending = PendingContinuation::new(certificate.clone(), decision, key, 28).unwrap();
    step_until(&mut state, Tick(199));
    let first_receipt = observe(&state, settings);
    let mut early = pending.fork_for_test();
    assert!(
        early
            .observe(
                &first_receipt,
                publication(decision, 29, Tick(198)),
                &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
            )
            .is_err()
    );
    assert!(
        early
            .observe(
                &first_receipt,
                publication(decision, 29, Tick(199)),
                &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
            )
            .is_err()
    );
    assert!(
        pending
            .observe(
                &first_receipt,
                publication(decision, 29, Tick(199)),
                &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
            )
            .unwrap()
            .is_none()
    );

    step_until(&mut state, Tick(202));
    let current = observe(&state, settings);
    let initial = publication(decision, 30, Tick(202));
    let mut partial_past_horizon = pending.fork_for_test();
    partial_past_horizon.shorten_horizon_for_test(Tick(204));
    assert!(
        partial_past_horizon
            .observe(
                &current,
                initial,
                &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
            )
            .is_err()
    );
    let mut measured = WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE);
    let prepared = pending
        .fork_for_test()
        .observe(&current, initial, &mut measured)
        .expect("charged admission")
        .expect("second receipt mints admission");
    let required = measured.spent();
    assert!((1..=CONTINUATION_RENEWAL_ALLOWANCE).contains(&required));
    assert!(
        pending
            .fork_for_test()
            .observe(&current, initial, &mut WorkBudget::new(required))
            .is_ok()
    );
    assert!(
        pending
            .fork_for_test()
            .observe(&current, initial, &mut WorkBudget::new(required - 1))
            .is_err()
    );

    let foreign = pending
        .observe(
            &current,
            initial,
            &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
        )
        .unwrap()
        .unwrap();
    assert!(prepared.commit(foreign.command()).is_err());
    let prepared = foreign;

    let first = prepared.command();
    let mut committed = prepared.commit(first.clone()).unwrap();
    let mut expired = committed.fork_for_test();
    expired.shorten_horizon_for_test(Tick(203));
    assert!(matches!(
        expired.poll_command(Tick(203), decision.command_epoch),
        ContinuationStatus::Invalidated
    ));
    let mut missed = committed.fork_for_test();
    let ContinuationStatus::Submit(second) = missed.poll_command(Tick(203), decision.command_epoch)
    else {
        panic!("second exact input must be due")
    };
    assert_eq!(second.admission_tick(), Tick(203));
    assert!(matches!(
        missed.poll_command(Tick(204), decision.command_epoch),
        ContinuationStatus::Invalidated
    ));
    assert!(!missed.commit_command(second.clone()));
    let mut rejected = committed.fork_for_test();
    rejected.revoke();
    assert!(!rejected.commit_command(second));
    assert!(matches!(
        rejected.poll_command(Tick(203), decision.command_epoch),
        ContinuationStatus::Invalidated
    ));
    let mut sequence = initial.sequence;
    let mut maximum_work = required;
    let mut next = first;
    loop {
        let update = Tick(state.tick().0 + 1);
        let rival_movement = rival_checkpoints
            .iter()
            .find(|checkpoint| checkpoint.tick == update)
            .map_or(MovementIntent::Stop, |checkpoint| checkpoint.movement);
        state
            .step(&[
                input(CPU, next.movement()),
                input(SeatId::One, rival_movement),
            ])
            .unwrap();
        if state.tick() == Tick(312) {
            break;
        }
        if matches!(state.tick(), Tick(203) | Tick(204)) {
            let mut early = committed.fork_for_test();
            assert!(
                early
                    .prepare_grant(
                        &observe(&state, settings),
                        publication(decision, sequence + 1, state.tick()),
                        &mut WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE),
                    )
                    .is_err()
            );
            assert!(matches!(
                early.poll_command(state.tick(), decision.command_epoch),
                ContinuationStatus::Invalidated
            ));
        }
        match committed.poll_command(state.tick(), decision.command_epoch) {
            ContinuationStatus::Submit(command) => {
                assert!(committed.commit_command(command.clone()));
                next = command;
            }
            ContinuationStatus::RenewalRequired => {
                sequence += 1;
                let current = observe(&state, settings);
                let identity = publication(decision, sequence, state.tick());
                if state.tick() == Tick(205) {
                    assert_renewal_bound_and_rejections(&committed, &current, identity);
                }
                let mut work = WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE);
                let prepared = committed
                    .prepare_grant(&current, identity, &mut work)
                    .expect("fresh charged renewal");
                if state.tick() == Tick(241) {
                    assert_eq!(
                        prepared.movements_for_test(),
                        Some([
                            MovementIntent::Direction(Direction::South),
                            MovementIntent::Stop,
                            MovementIntent::Stop,
                        ])
                    );
                }
                maximum_work = maximum_work.max(work.spent());
                let command = prepared.command();
                assert!(committed.commit_grant(prepared, command.clone()));
                next = command;
            }
            ContinuationStatus::Continue | ContinuationStatus::Wait => {
                panic!("every physical update must have its exact submitted input")
            }
            ContinuationStatus::Completed => {
                step_until(&mut state, Tick(312));
                break;
            }
            ContinuationStatus::Invalidated => panic!("certified physical scene invalidated"),
        }
    }
    eprintln!("continuation renewal max work={maximum_work}");
    assert!(maximum_work <= CONTINUATION_RENEWAL_ALLOWANCE);
    assert!(state.actor(CPU).unwrap().alive());
    assert!(state.world().bombs().is_empty());
    assert!(state.public_view().flames.is_empty());
}

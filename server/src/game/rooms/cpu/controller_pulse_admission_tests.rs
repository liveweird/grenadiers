use super::*;

fn queued_rules_ten_pulse() -> (
    CpuController,
    grenadiers_core::cpu::Observation,
    Intent,
    CpuRng,
    Tick,
) {
    let mut controller = CpuController::new(
        CpuPlanner::without_workers(),
        SeatId::Two,
        7,
        [5; 32],
        grenadiers_core::room::CpuProfile::HARD_V55,
    );
    controller.reset_world(3);
    let basis = pit_basis();
    assert!(controller.accept_observation(&basis));
    controller.settled_stop_tick = Some(basis.parts().tick);
    let key = key_for(&controller);
    let mut next_rng = CpuRng::from_seed(key, [37; 32]);
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&basis).unwrap();
    let planned = grenadiers_core::cpu::survival::plan_hard_pit_avoidance_v53_with_origin(
        &basis,
        &UtilityMemory::new(key),
        &history,
        Tick(20),
        None,
        &mut next_rng,
        &mut WorkBudget::new(grenadiers_core::cpu::survival::WORK_ALLOWANCE),
    )
    .unwrap();
    let mut survival = planned.survival;
    if survival.lease.is_none() {
        panic!("rules-ten pulse fixture: {survival:?}");
    }
    let lease = survival.lease.as_mut().unwrap();
    assert_eq!(lease.start_window().start, Tick(21));
    let window_end = lease.start_window().end;
    let intent = survival.control.intent;
    let pulse_decision = decision(&controller);
    inject_plan(
        &mut controller,
        pulse_decision,
        survival,
        basis.clone(),
        next_rng.clone(),
    );
    (controller, basis, intent, next_rng, window_end)
}

fn pit_basis() -> grenadiers_core::cpu::Observation {
    let base = crate::game::rooms::cpu::tests::spike_observation();
    let environment = base.environment().unwrap().clone();
    let mut parts = base.into_parts();
    let arena = grenadiers_core::game::Arena::new(
        parts.arena.terrain.clone().try_into().unwrap(),
        parts.arena.blocks.iter().copied().collect(),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap();
    let pit_cells = grenadiers_core::game::spike_pit_layout_v1(&arena, []);
    let grenadiers_core::cpu::ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
        unreachable!()
    };
    cpu.position = Position::center(CellId::new(7, 7));
    parts.pickups.push(grenadiers_core::cpu::ObservedPickup {
        cell: CellId::new(5, 7),
        kind: grenadiers_core::game::PickupKind::GoldenFlame,
    });
    let pits = pit_cells
        .into_iter()
        .map(|cell| grenadiers_core::game::SpikePitView {
            cell,
            phase: grenadiers_core::game::SpikePitPhase::Idle,
            remaining_ticks: 0,
        })
        .collect();
    grenadiers_core::cpu::Observation::new_rules_v10(
        parts,
        environment,
        None,
        Some(grenadiers_core::cpu::ObservedSpikePits::new(
            grenadiers_core::room::SpikePitConfig::Enabled(
                grenadiers_core::room::SpikePitSettings::DEFAULT,
            ),
            pits,
        )),
    )
    .unwrap()
}

fn next_publication(
    basis: &grenadiers_core::cpu::Observation,
) -> grenadiers_core::cpu::Observation {
    basis
        .clone()
        .map_parts(|parts| {
            parts.tick = Tick(21);
            parts.rules.running_updates += 1;
            parts.rules.remaining_ticks -= 1;
        })
        .unwrap()
}

#[test]
fn rules_ten_pulse_waits_for_current_publication_then_admits_unchanged_plan() {
    let (mut controller, basis, intent, next_rng, _) = queued_rules_ten_pulse();
    let memory_before = controller.memory;
    let rng_draws_before = controller.rng.as_ref().unwrap().draws();

    assert!(controller.poll(Tick(21), 4).is_none());
    assert!(controller.has_pending());
    assert!(controller.pending.as_ref().unwrap().completed.is_some());
    assert!(controller.committed_pulse.is_none());
    assert_eq!(controller.memory, memory_before);
    assert_eq!(controller.rng.as_ref().unwrap().draws(), rng_draws_before);
    assert_eq!(controller.admission_trace_for_test(), Default::default());

    assert!(controller.accept_observation(&next_publication(&basis)));
    let accepted = controller.poll(Tick(21), 4).unwrap();
    assert_eq!(accepted.intent, intent);
    assert_eq!(accepted.next_rng.draws(), next_rng.draws());
    assert!(accepted.committed_pulse.is_some());
    assert!(!controller.has_pending());
    assert_eq!(controller.admission_trace_for_test(), Default::default());
}

#[test]
fn deferred_rules_ten_pulse_does_not_outlive_the_existing_age_limit() {
    let (mut controller, _, _, _, window_end) = queued_rules_ten_pulse();
    let memory_before = controller.memory;
    let rng_draws_before = controller.rng.as_ref().unwrap().draws();

    assert_eq!(window_end, Tick(28));
    assert!(controller.poll(Tick(21), 4).is_none());
    assert!(controller.has_pending());
    assert!(controller.poll(Tick(26), 4).is_none());
    assert!(controller.has_pending());
    assert!(controller.poll(Tick(27), 4).is_none());
    assert!(!controller.has_pending());
    assert_eq!(controller.memory, memory_before);
    assert_eq!(controller.rng.as_ref().unwrap().draws(), rng_draws_before);
    assert_eq!(
        controller
            .admission_trace_for_test()
            .expired_or_wrong_decision,
        1
    );
}

#[test]
fn fresh_changed_pose_rejects_a_deferred_rules_ten_pulse() {
    let (mut controller, basis, _, _, _) = queued_rules_ten_pulse();
    let memory_before = controller.memory;
    let rng_draws_before = controller.rng.as_ref().unwrap().draws();
    assert!(controller.poll(Tick(21), 4).is_none());
    let changed = next_publication(&basis)
        .map_parts(|parts| {
            let grenadiers_core::cpu::ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
                unreachable!()
            };
            cpu.position = Position::center(CellId::new(11, 9));
        })
        .unwrap();
    assert!(controller.accept_observation(&changed));

    assert!(controller.poll(Tick(21), 4).is_none());
    assert!(!controller.has_pending());
    assert_eq!(controller.memory, memory_before);
    assert_eq!(controller.rng.as_ref().unwrap().draws(), rng_draws_before);
    assert_eq!(controller.admission_trace_for_test().stale_scene, 1);
}

#[test]
fn fresh_changed_pit_phase_strictly_rejects_a_deferred_rules_ten_pulse() {
    let (mut controller, basis, _, _, _) = queued_rules_ten_pulse();
    let memory_before = controller.memory;
    let rng_draws_before = controller.rng.as_ref().unwrap().draws();
    assert!(controller.poll(Tick(21), 4).is_none());
    let fresh = next_publication(&basis);
    let pits = fresh.spike_pits().unwrap();
    let mut changed_pits = pits.pits().to_vec();
    changed_pits[0].phase = grenadiers_core::game::SpikePitPhase::Warning;
    changed_pits[0].remaining_ticks =
        grenadiers_core::room::SpikePitSettings::DEFAULT.warning_ticks();
    let changed = grenadiers_core::cpu::Observation::new_rules_v10(
        fresh.parts().clone(),
        fresh.environment().unwrap().clone(),
        None,
        Some(grenadiers_core::cpu::ObservedSpikePits::new(
            pits.config(),
            changed_pits,
        )),
    )
    .unwrap();
    assert!(controller.accept_observation(&changed));

    assert!(controller.poll(Tick(21), 4).is_none());
    assert!(!controller.has_pending());
    assert_eq!(controller.memory, memory_before);
    assert_eq!(controller.rng.as_ref().unwrap().draws(), rng_draws_before);
    assert_eq!(controller.admission_trace_for_test().pulse, 1);
}

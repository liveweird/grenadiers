use super::*;

#[test]
fn cpu24_pulse_invalidation_retains_strategy_but_revokes_pulse_authority() {
    let mut controller = controller();
    let basis = crate::game::rooms::cpu::tests::observation();
    assert!(controller.accept_observation(&basis));
    controller.settled_stop_tick = Some(basis.parts().tick);
    let key = key_for(&controller);
    let forecast = forecast(
        &basis,
        &ObservedHazardMemory::new(key),
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    let lease = certify_pulse(
        &basis,
        &forecast,
        PulseRequest {
            movement: grenadiers_core::cpu::MovementIntent::Direction(Direction::West),
            hold_ticks: 12,
            start_window: TickWindow {
                start: Tick(21),
                end: Tick(22),
            },
            stopped_safe_until: Tick(50),
            mode: RouteMode::CertifiedOnly,
            settled_stop_tick: Tick(20),
        },
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    let strategic = test_strategy();
    let intent = Intent::moving(Direction::West);
    let pulse_decision = decision(&controller);
    inject_plan(
        &mut controller,
        pulse_decision,
        SurvivalPlan {
            control: ControlPlan {
                intent,
                hold_ticks: 12,
                next_memory: UtilityMemory::new(key).with_strategic_intent(strategic),
            },
            lease: Some(lease),
            execution: None,
        },
        basis.clone(),
        CpuRng::from_seed(key, [8; 32]),
    );
    let accepted = controller.poll(Tick(20), 4).unwrap();
    controller.commit(accepted, intent);
    let invalid = basis
        .map_parts(|parts| {
            let grenadiers_core::cpu::ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
                unreachable!()
            };
            cpu.position = Position::center(CellId::new(11, 9));
        })
        .unwrap();
    assert_eq!(
        controller.validate_pulse(&invalid, Tick(20)),
        Some(PulseStatus::Invalidated)
    );
    assert!(controller.committed_pulse.is_none());
    assert_eq!(
        controller.memory.unwrap().strategic_intent(),
        Some(strategic)
    );
    assert!(
        controller
            .memory
            .unwrap()
            .route()
            .iter()
            .all(Option::is_none)
    );
}

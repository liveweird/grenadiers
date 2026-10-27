use grenadiers_core::cpu::{
    BombKnowledge, MemoryKey, ObservedBomb, ObservedBombTrigger, ObservedFlame, UnknownBombFacts,
    WorkBudget,
    forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast},
    policy::{ControlPlan, UtilityMemory},
    routing::{RouteLimits, RouteMode, RouteRequest, plan_route},
    survival::{ExecutionCertificate, SurvivalPlan},
};
use grenadiers_core::game::arena::ARENA_WIDTH;
use grenadiers_core::game::{BlastKind, CellId, EntityId, SeatId, Tick};

use super::CpuController;
use crate::game::rooms::cpu::worker::CpuPlanner;

#[test]
fn route_admission_accepts_predicted_detonation_but_rejects_an_extra_hazard() {
    let (basis, current, route, key) = detonation_fixture();
    let make_controller = || {
        let mut controller = CpuController::new(
            CpuPlanner::without_workers(),
            7,
            [5; 32],
            grenadiers_core::room::CpuProfile::CURRENT,
        );
        controller.reset_world(3);
        assert!(controller.accept_observation(&basis));
        controller.settled_stop_tick = Some(basis.parts().tick);
        assert!(controller.accept_observation(&current));
        controller
    };
    let plan = || SurvivalPlan {
        control: ControlPlan::stop(UtilityMemory::new(key)),
        lease: None,
        execution: Some(ExecutionCertificate::Route(Box::new(route.clone()))),
    };

    let mut accepted = make_controller();
    accepted.inject_plan_result_for_test(basis.clone(), 4, plan());
    assert!(accepted.poll(Tick(21), 4).is_some());

    let mut unexpected = current.clone().into_parts();
    unexpected.flames.push(ObservedFlame {
        cell: CellId::new(11, 9),
        remaining_ticks: 4,
    });
    let unexpected_observation = grenadiers_core::cpu::Observation::new(unexpected).unwrap();
    let mut rejected = CpuController::new(
        CpuPlanner::without_workers(),
        7,
        [5; 32],
        grenadiers_core::room::CpuProfile::CURRENT,
    );
    rejected.reset_world(3);
    assert!(rejected.accept_observation(&basis));
    rejected.settled_stop_tick = Some(basis.parts().tick);
    assert!(rejected.accept_observation(&unexpected_observation));
    rejected.inject_plan_result_for_test(basis, 4, plan());
    assert!(rejected.poll(Tick(21), 4).is_none());
    assert_eq!(rejected.rng_draws(), 0);
}

#[test]
fn clear_gap_requires_an_unencumbered_settled_stop() {
    let (mut completed, clear) = committed_wait_controller();
    assert_eq!(
        completed.observe_route(&clear),
        Some(grenadiers_core::cpu::survival::RouteStatus::Completed)
    );
    assert!(!completed.has_committed_route());
    assert_eq!(completed.observe_route(&clear), None);

    let (mut unsettled, clear) = committed_wait_controller();
    unsettled.settled_stop_tick = None;
    assert_ne!(
        unsettled.observe_route(&clear),
        Some(grenadiers_core::cpu::survival::RouteStatus::Completed)
    );
    assert!(unsettled.has_committed_route());

    let (mut too_early, clear) = committed_wait_controller();
    too_early.settled_stop_tick = Some(Tick(21));
    assert_ne!(
        too_early.observe_route(&clear),
        Some(grenadiers_core::cpu::survival::RouteStatus::Completed)
    );
    assert!(too_early.has_committed_route());

    let (mut legacy, clear) = committed_wait_controller();
    legacy.escaping = true;
    assert_ne!(
        legacy.observe_route(&clear),
        Some(grenadiers_core::cpu::survival::RouteStatus::Completed)
    );
    assert!(legacy.has_committed_route());
}

fn committed_wait_controller() -> (CpuController, grenadiers_core::cpu::Observation) {
    let (basis, current, route, key) = detonation_fixture();
    let mut controller = CpuController::new(
        CpuPlanner::without_workers(),
        7,
        [5; 32],
        grenadiers_core::room::CpuProfile::CURRENT,
    );
    controller.reset_world(3);
    assert!(controller.accept_observation(&basis));
    controller.settled_stop_tick = Some(basis.parts().tick);
    assert!(controller.accept_observation(&current));
    controller.inject_plan_result_for_test(
        basis,
        4,
        SurvivalPlan {
            control: ControlPlan::stop(UtilityMemory::new(key)),
            lease: None,
            execution: Some(ExecutionCertificate::Route(Box::new(route))),
        },
    );
    let accepted = controller.poll(Tick(21), 4).unwrap();
    let actual = accepted.intent;
    controller.commit(accepted, actual);
    controller.settled_stop_tick = Some(Tick(22));

    let mut clear = current.into_parts();
    clear.tick = Tick(52);
    clear.rules.running_updates += 31;
    clear.rules.remaining_ticks -= 31;
    clear.flames.clear();
    (
        controller,
        grenadiers_core::cpu::Observation::new(clear).unwrap(),
    )
}

fn detonation_fixture() -> (
    grenadiers_core::cpu::Observation,
    grenadiers_core::cpu::Observation,
    grenadiers_core::cpu::routing::RouteCertificate,
    MemoryKey,
) {
    let mut parts = crate::game::rooms::cpu::tests::observation().into_parts();
    parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
        id: EntityId(90),
        placed_at: grenadiers_core::game::Tick(0),
        cell: CellId::new(5, 5),
        owner: EntityId(1),
        range: 1,
        trigger: ObservedBombTrigger::Timed {
            remaining_ticks: 1,
            paused: false,
        },
        blast: BlastKind::Standard,
        unknown: UnknownBombFacts::ALL,
    }]);
    let basis = grenadiers_core::cpu::Observation::new(parts).unwrap();
    let key = MemoryKey {
        world: basis.parts().world,
        cpu_seat: SeatId::Two,
    };
    let hazards = forecast(
        &basis,
        &ObservedHazardMemory::new(key),
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    let route = plan_route(
        &basis,
        &hazards,
        RouteRequest {
            start_window: TickWindow {
                start: Tick(21),
                end: Tick(27),
            },
            settled_stop_tick: Tick(20),
            survive_until: Tick(60),
            mode: RouteMode::CertifiedOnly,
            limits: RouteLimits::default(),
        },
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    let mut current = basis.clone().into_parts();
    current.tick = Tick(21);
    current.rules.running_updates += 1;
    current.rules.remaining_ticks -= 1;
    current.bombs = BombKnowledge::Visible(Vec::new());
    current.flames = hazards
        .cells()
        .iter()
        .enumerate()
        .filter(|(_, cell)| cell.lethal.iter().any(|window| window.start == Tick(21)))
        .map(|(index, _)| ObservedFlame {
            cell: CellId::new(
                u16::try_from(index % usize::from(ARENA_WIDTH)).unwrap(),
                u16::try_from(index / usize::from(ARENA_WIDTH)).unwrap(),
            ),
            remaining_ticks: 30,
        })
        .collect();
    let current = grenadiers_core::cpu::Observation::new(current).unwrap();
    (basis, current, route, key)
}

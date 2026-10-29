use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::lair_contact::tests::observation;
use crate::cpu::policy::{PolicyError, UtilityMemory};
use crate::cpu::routing::{
    PlantFailure, PlantRequest, PulseRequest, RouteFailure, RouteLimits, RouteMode, RouteRequest,
    certify_pulse, plan_ordinary_plant, plan_route, plan_timed_weapon_plant,
};
use crate::cpu::survival;
use crate::cpu::{CpuRng, MemoryKey, MovementIntent, WorkBudget};
use crate::game::Tick;

fn key(observed: &crate::cpu::Observation) -> MemoryKey {
    MemoryKey {
        world: observed.parts().world,
        cpu_seat: observed.parts().recipient,
    }
}

#[test]
fn legacy_public_policies_reject_rules_v8_before_emitting_control() {
    let observed = observation(Vec::new(), Tick(200));
    let key = key(&observed);
    let memory = UtilityMemory::new(key);
    let history = ObservedHazardMemory::new(key);

    assert_eq!(
        crate::cpu::policy::plan(
            &observed,
            &memory,
            &mut CpuRng::from_seed(key, [31; 32]),
            &mut WorkBudget::new(65_536),
        ),
        Err(PolicyError::InvalidObservation)
    );
    assert_eq!(
        survival::plan(
            &observed,
            &memory,
            &history,
            Tick(100),
            &mut CpuRng::from_seed(key, [32; 32]),
            &mut WorkBudget::new(65_536),
        ),
        Err(PolicyError::InvalidObservation)
    );
    assert_eq!(
        survival::plan_with_origin(
            &observed,
            &memory,
            &history,
            Tick(100),
            None,
            &mut CpuRng::from_seed(key, [33; 32]),
            &mut WorkBudget::new(65_536),
        ),
        Err(PolicyError::InvalidObservation)
    );
}

#[test]
fn legacy_public_routing_rejects_rules_v8_without_spending_work() {
    let observed = observation(Vec::new(), Tick(200));
    let key = key(&observed);
    let history = ObservedHazardMemory::new(key);
    let hazards = forecast(
        &observed,
        &history,
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    let start_window = TickWindow {
        start: Tick(101),
        end: Tick(102),
    };
    let mut work = WorkBudget::new(65_536);
    assert_eq!(
        certify_pulse(
            &observed,
            &hazards,
            PulseRequest {
                movement: MovementIntent::Stop,
                hold_ticks: 1,
                start_window,
                stopped_safe_until: Tick(112),
                mode: RouteMode::CertifiedOnly,
                settled_stop_tick: Tick(100),
            },
            &mut work,
        ),
        Err(RouteFailure::InvalidRequest)
    );
    assert_eq!(
        plan_route(
            &observed,
            &hazards,
            RouteRequest {
                start_window,
                settled_stop_tick: Tick(100),
                survive_until: Tick(112),
                mode: RouteMode::CertifiedOnly,
                limits: RouteLimits::default(),
            },
            &mut work,
        ),
        Err(RouteFailure::InvalidRequest)
    );
    assert_eq!(work.spent(), 0);
}

#[test]
fn legacy_public_plant_wrappers_reject_rules_v8_without_spending_work() {
    let observed = observation(Vec::new(), Tick(200));
    let history = ObservedHazardMemory::new(key(&observed));
    let request = PlantRequest {
        execution_tick: Tick(107),
        settled_stop_tick: Tick(100),
        first_movement: MovementIntent::Stop,
        limits: RouteLimits::default(),
    };
    let mut work = WorkBudget::new(65_536);
    assert_eq!(
        plan_ordinary_plant(&observed, &history, request, &mut work),
        Err(PlantFailure::Unsupported)
    );
    assert_eq!(
        plan_timed_weapon_plant(&observed, &history, request, &mut work),
        Err(PlantFailure::Unsupported)
    );
    assert_eq!(work.spent(), 0);
}

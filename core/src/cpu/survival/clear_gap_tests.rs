use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::policy::tests::{key, observation};
use crate::cpu::routing::{RouteLimits, RouteMode, RouteRequest, plan_route};
use crate::cpu::{MovementIntent, Observation, ObservedActor, ObservedFlame, WorkBudget};
use crate::game::{CellId, Tick};

use super::route_tests::cleared_wait_route;
use super::{CommittedRoute, RouteStatus};

fn admitted() -> (
    crate::cpu::Observation,
    CommittedRoute,
    crate::cpu::Observation,
) {
    let (basis, route, clear) = cleared_wait_route();
    let committed =
        CommittedRoute::admit(route, basis.clone(), &basis, Tick(10), MovementIntent::Stop)
            .unwrap();
    (basis, committed, clear)
}

#[test]
fn clear_gap_requires_a_fresh_in_bounds_stopped_snapshot() {
    let (_, mut too_early, clear) = admitted();
    assert_eq!(
        too_early.observe_with_stop(&clear, Some(Tick(10))),
        RouteStatus::Wait
    );
    let (_, mut future, clear) = admitted();
    assert_eq!(
        future.observe_with_stop(&clear, Some(Tick(151))),
        RouteStatus::Wait
    );
    let (_, mut legacy, clear) = admitted();
    assert_eq!(legacy.observe(&clear), RouteStatus::Wait);
}

#[test]
fn a_still_visible_basis_bomb_retains_the_wait() {
    let (basis, mut committed, _) = admitted();
    let actor = basis
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == basis.parts().recipient => Some(actor),
            _ => None,
        })
        .unwrap();
    let current = super::tests::later(&basis, 50, actor.position, actor.heading);
    assert_eq!(
        committed.observe_with_stop(&current, Some(Tick(11))),
        RouteStatus::Wait
    );
}

#[test]
fn an_expired_finite_flame_without_a_basis_bomb_does_not_complete_early() {
    let mut parts = observation(CellId::new(2, 1)).into_parts();
    parts.flames.push(ObservedFlame {
        cell: CellId::new(10, 10),
        remaining_ticks: 20,
    });
    let basis = Observation::new(parts).unwrap();
    let hazards = forecast(
        &basis,
        &ObservedHazardMemory::new(key()),
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
                start: Tick(11),
                end: Tick(18),
            },
            settled_stop_tick: Tick(10),
            survive_until: hazards.horizon_end(),
            mode: RouteMode::CertifiedOnly,
            limits: RouteLimits::default(),
        },
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    let mut committed =
        CommittedRoute::admit(route, basis.clone(), &basis, Tick(10), MovementIntent::Stop)
            .unwrap();
    let ObservedActor::Visible(actor) = &basis.parts().actors[1] else {
        unreachable!()
    };
    let current = super::tests::later(&basis, 40, actor.position, actor.heading);
    assert_eq!(
        committed.observe_with_stop(&current, Some(Tick(11))),
        RouteStatus::Wait
    );
}

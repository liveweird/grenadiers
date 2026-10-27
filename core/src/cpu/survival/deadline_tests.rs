use crate::cpu::forecast::{ObservedHazardMemory, TickWindow};
use crate::cpu::policy::{UtilityMemory, tests::observation};
use crate::cpu::routing::RouteStep;
use crate::cpu::{CpuRng, MovementIntent, Observation, ObservedFlame, ObservedOil, WorkBudget};
use crate::game::{CellId, EnabledItems, OilBarrelState, PickupKind, Tick};

use super::deadline::{ordinary_movement_is_clear, ordinary_route_end};
use super::{ExecutionCertificate, WORK_ALLOWANCE, plan};

fn key() -> crate::cpu::MemoryKey {
    crate::cpu::policy::tests::key()
}

fn start_window() -> TickWindow {
    TickWindow {
        start: Tick(11),
        end: Tick(18),
    }
}

fn with_flames(remaining: &[u16]) -> Observation {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    parts.flames = remaining
        .iter()
        .enumerate()
        .map(|(index, remaining_ticks)| ObservedFlame {
            cell: CellId::new(8 + index as u16, 7),
            remaining_ticks: *remaining_ticks,
        })
        .collect();
    Observation::new(parts).unwrap()
}

fn with_oil(state: OilBarrelState, cell: CellId) -> Observation {
    let mut parts = observation(CellId::new(7, 7)).into_parts();
    parts.rules.settings = parts
        .rules
        .settings
        .with_enabled_items(EnabledItems::OIL_BARREL);
    parts.oil.push(ObservedOil { cell, state });
    Observation::new(parts).unwrap()
}

fn route_end(observation: &Observation, work: &mut WorkBudget) -> Option<Tick> {
    ordinary_route_end(observation, start_window(), Tick(850), work)
}

#[test]
fn finite_deadline_has_exact_minimum_flame_and_horizon_boundaries() {
    for (remaining, expected) in [(1, 39), (28, 39), (29, 39), (30, 40), (841, 850)] {
        assert_eq!(
            route_end(&with_flames(&[remaining]), &mut WorkBudget::new(1)),
            Some(Tick(expected)),
            "remaining={remaining}"
        );
    }
    assert_eq!(
        route_end(&with_flames(&[1, 30, 28]), &mut WorkBudget::new(3)),
        Some(Tick(40))
    );
}

#[test]
fn deadline_checked_arithmetic_rejects_each_clock_overflow() {
    let clear = observation(CellId::new(7, 7));
    assert_eq!(
        ordinary_route_end(
            &clear,
            TickWindow {
                start: Tick(0),
                end: Tick(0),
            },
            Tick(u64::MAX),
            &mut WorkBudget::new(0),
        ),
        None
    );
    assert_eq!(
        ordinary_route_end(
            &clear,
            TickWindow {
                start: Tick(u64::MAX - 1),
                end: Tick(u64::MAX),
            },
            Tick(u64::MAX),
            &mut WorkBudget::new(0),
        ),
        None
    );

    let mut parts = with_flames(&[2]).into_parts();
    parts.tick = Tick(u64::MAX - 1);
    let near_limit = Observation::new(parts).unwrap();
    assert_eq!(
        ordinary_route_end(
            &near_limit,
            start_window(),
            Tick(u64::MAX),
            &mut WorkBudget::new(1),
        ),
        None
    );
}

#[test]
fn ordinary_clear_classification_fails_closed_on_work_and_burning_oil() {
    let unlit = with_oil(OilBarrelState::Unlit, CellId::new(3, 3));
    let mut exhausted = WorkBudget::new(0);
    assert!(!ordinary_movement_is_clear(&unlit, true, &mut exhausted));
    assert_eq!(exhausted.spent(), 0);

    let burning = with_oil(OilBarrelState::Burning, CellId::new(3, 3));
    let mut exact = WorkBudget::new(1);
    assert!(!ordinary_movement_is_clear(&burning, true, &mut exact));
    assert_eq!(exact.spent(), 1);
}

#[test]
fn deadline_scans_are_metered_and_infinite_public_hazards_keep_the_full_horizon() {
    let mut parts = with_flames(&[28, 30]).into_parts();
    parts.rules.settings = parts
        .rules
        .settings
        .with_enabled_items(EnabledItems::OIL_BARREL);
    parts.oil = vec![
        ObservedOil {
            cell: CellId::new(3, 3),
            state: OilBarrelState::Unlit,
        },
        ObservedOil {
            cell: CellId::new(4, 3),
            state: OilBarrelState::Unlit,
        },
    ];
    let observed = Observation::new(parts).unwrap();
    let mut exact = WorkBudget::new(4);
    assert_eq!(route_end(&observed, &mut exact), Some(Tick(40)));
    assert_eq!(exact.spent(), 4);
    let mut short = WorkBudget::new(3);
    assert_eq!(route_end(&observed, &mut short), None);
    assert_eq!(short.spent(), 3);

    assert_eq!(
        route_end(
            &with_oil(OilBarrelState::Burning, CellId::new(3, 3)),
            &mut WorkBudget::new(1)
        ),
        Some(Tick(850))
    );
    for trigger in [
        crate::cpu::ObservedBombTrigger::Timed {
            remaining_ticks: 200,
            paused: false,
        },
        crate::cpu::ObservedBombTrigger::Timed {
            remaining_ticks: 200,
            paused: true,
        },
        crate::cpu::ObservedBombTrigger::Remote,
        crate::cpu::ObservedBombTrigger::Contact,
    ] {
        let bomb = crate::cpu::routing::tests::with_bomb(observation(CellId::new(7, 7)), trigger);
        let mut no_work = WorkBudget::new(0);
        assert_eq!(route_end(&bomb, &mut no_work), Some(Tick(850)));
    }
}

#[test]
fn planner_certificate_ends_at_the_latest_flame_and_keeps_all_start_lanes() {
    let mut parts = with_flames(&[1, 30, 28]).into_parts();
    parts.pickups.push(crate::cpu::ObservedPickup {
        cell: CellId::new(9, 7),
        kind: PickupKind::BombCapacity,
    });
    let observed = Observation::new(parts).unwrap();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = plan(
        &observed,
        &UtilityMemory::new(key()),
        &ObservedHazardMemory::new(key()),
        observed.parts().tick,
        &mut CpuRng::from_seed(key(), [88; 32]),
        &mut work,
    )
    .unwrap();
    let Some(ExecutionCertificate::Route(route)) = planned.execution else {
        panic!("expected finite route certificate: {planned:?}")
    };
    assert_eq!(route.survive_until(), Tick(40));
    assert!(matches!(
        route.steps().first(),
        Some(RouteStep::Wait { .. })
    ));
    for first_update in 11..18 {
        assert!(route.lane(Tick(first_update)).is_some());
    }
    eprintln!(
        "finite_hazard_metric basis={} survive_until={} span={} work_spent={}",
        observed.parts().tick.0,
        route.survive_until().0,
        route.survive_until().0 - observed.parts().tick.0,
        work.spent()
    );
    assert!(work.spent() < WORK_ALLOWANCE);
}

#[test]
fn unlit_oil_allows_ordinary_policy_but_remains_an_impassable_cell() {
    let run = |observed: &Observation| {
        plan(
            observed,
            &UtilityMemory::new(key()),
            &ObservedHazardMemory::new(key()),
            observed.parts().tick,
            &mut CpuRng::from_seed(key(), [88; 32]),
            &mut WorkBudget::new(WORK_ALLOWANCE),
        )
        .unwrap()
    };
    let mut remote = with_oil(OilBarrelState::Unlit, CellId::new(3, 3)).into_parts();
    remote.pickups.push(crate::cpu::ObservedPickup {
        cell: CellId::new(8, 7),
        kind: PickupKind::BombCapacity,
    });
    let remote = run(&Observation::new(remote).unwrap());
    assert_eq!(
        remote.control.intent.movement,
        MovementIntent::Direction(crate::game::Direction::East)
    );
    assert!(remote.lease.is_none() && remote.execution.is_none());

    let mut direct = with_oil(OilBarrelState::Unlit, CellId::new(8, 7)).into_parts();
    direct.pickups.push(crate::cpu::ObservedPickup {
        cell: CellId::new(9, 7),
        kind: PickupKind::BombCapacity,
    });
    let direct = run(&Observation::new(direct).unwrap());
    assert_ne!(
        direct.control.intent.movement,
        MovementIntent::Direction(crate::game::Direction::East)
    );
}

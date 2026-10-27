use crate::cpu::{
    BombKnowledge, ObservedBomb, ObservedBombTrigger, ObservedFlame, UnknownBombFacts, WorkBudget,
};
use crate::game::{BlastKind, CellId, EntityId, Tick};

use super::tests::{key, observation};
use super::{ForecastLimits, HypotheticalPlacement, ObservedHazardMemory, TickWindow, forecast};

const ORIGIN: CellId = CellId::new(2, 1);

fn placed_forecast(flame_remaining: u16) -> super::HazardForecast {
    let mut parts = observation(0, BombKnowledge::Visible(Vec::new())).into_parts();
    parts.flames = vec![ObservedFlame {
        cell: ORIGIN,
        remaining_ticks: flame_remaining,
    }];
    let observed = crate::cpu::Observation::new(parts).unwrap();
    forecast(
        &observed,
        &ObservedHazardMemory::new(key()),
        Some(HypotheticalPlacement {
            placed_at: Tick(7),
            owner: EntityId(20),
            cell: ORIGIN,
            range: 2,
            blast: BlastKind::Standard,
            detonates_at: Tick(60),
        }),
        ForecastLimits::default(),
        &mut WorkBudget::new(65_536),
    )
    .unwrap()
}

#[test]
fn expired_basis_flame_does_not_ignite_a_future_placement() {
    let forecast = placed_forecast(2);
    let hypothetical = forecast.hypothetical_id().unwrap();
    assert!(
        !forecast
            .evolution()
            .hypothetical_removed(hypothetical, Tick(8))
    );
    assert!(forecast.cell(ORIGIN).unwrap().lethal.iter().all(|window| {
        *window
            != (TickWindow {
                start: Tick(7),
                end: Tick(37),
            })
    }));
}

#[test]
fn surviving_basis_flame_ignites_at_the_placement_tick() {
    let forecast = placed_forecast(8);
    let hypothetical = forecast.hypothetical_id().unwrap();
    assert!(
        forecast
            .evolution()
            .hypothetical_removed(hypothetical, Tick(7))
    );
    assert!(
        forecast
            .cell(ORIGIN)
            .unwrap()
            .lethal
            .iter()
            .any(|window| window.contains(Tick(36)) && !window.contains(Tick(37)))
    );
}

#[test]
fn future_bomb_blocks_only_after_placement_movement() {
    let forecast = placed_forecast(2);
    let blocking = &forecast.cell(ORIGIN).unwrap().blocking;
    assert!(!blocking.iter().any(|window| window.contains(Tick(7))));
    assert!(blocking.iter().any(|window| window.contains(Tick(8))));
}

#[test]
fn future_candidate_can_shield_a_blast_that_reaches_farther_when_absent() {
    let mut parts = observation(0, BombKnowledge::Visible(Vec::new())).into_parts();
    parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
        id: EntityId(90),
        placed_at: Tick(0),
        cell: CellId::new(7, 1),
        owner: EntityId(10),
        range: 6,
        trigger: ObservedBombTrigger::Timed {
            remaining_ticks: 60,
            paused: false,
        },
        blast: BlastKind::Standard,
        unknown: UnknownBombFacts::ALL,
    }]);
    let observed = crate::cpu::Observation::new(parts).unwrap();
    let memory = ObservedHazardMemory::new(key());
    let mut work = WorkBudget::new(65_536);
    let absent = forecast(
        &observed,
        &memory,
        None,
        ForecastLimits::default(),
        &mut work,
    )
    .unwrap();
    let present = forecast(
        &observed,
        &memory,
        Some(HypotheticalPlacement {
            placed_at: Tick(7),
            owner: EntityId(20),
            cell: CellId::new(4, 1),
            range: 1,
            blast: BlastKind::Standard,
            detonates_at: Tick(120),
        }),
        ForecastLimits::default(),
        &mut work,
    )
    .unwrap();
    let refuge = CellId::new(2, 1);
    assert!(
        absent
            .cell(refuge)
            .unwrap()
            .lethal
            .iter()
            .any(|window| window.contains(Tick(60)))
    );
    assert!(
        present
            .cell(refuge)
            .unwrap()
            .lethal
            .iter()
            .all(|window| !window.contains(Tick(60)))
    );
}

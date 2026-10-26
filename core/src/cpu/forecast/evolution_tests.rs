use crate::cpu::{BombKnowledge, Observation, ObservedFlame};
use crate::game::arena::ARENA_WIDTH;
use crate::game::{BlastKind, CellId, EntityId, OilBarrelState, RulesConfig, Tick};
use crate::room::RoomSettings;

use super::ForecastLimits;
use super::tests::{observation, run, timed};

#[test]
fn evolution_keeps_physical_flame_expiry_past_forecast_horizon() {
    let basis = observation(
        0,
        BombKnowledge::Visible(vec![timed(1, CellId::new(5, 1), 830, BlastKind::Standard)]),
    );
    let forecast = run(&basis, ForecastLimits::default()).unwrap();
    let expiry = 830 + RulesConfig::expanded_v5(RoomSettings::DEFAULT).flame_ticks();
    let mut current = observation(831, BombKnowledge::Visible(Vec::new())).into_parts();
    current.rules.remaining_ticks = 169;
    current.flames = forecast
        .cells()
        .iter()
        .enumerate()
        .filter(|(_, cell)| cell.lethal.iter().any(|window| window.start == Tick(830)))
        .map(|(index, _)| ObservedFlame {
            cell: CellId::new(
                u16::try_from(index % usize::from(ARENA_WIDTH)).unwrap(),
                u16::try_from(index / usize::from(ARENA_WIDTH)).unwrap(),
            ),
            remaining_ticks: u16::try_from(expiry - 831).unwrap(),
        })
        .collect();
    let current = Observation::new(current).unwrap();

    assert!(forecast.evolution().matches_scene(&basis, &current));
    assert_eq!(
        forecast
            .evolution()
            .expected_live_bombs(EntityId(10), Tick(831)),
        Some(0)
    );
}

#[test]
fn evolution_accepts_countdown_but_rejects_displacement_and_new_flame() {
    let basis = observation(
        0,
        BombKnowledge::Visible(vec![timed(7, CellId::new(5, 1), 100, BlastKind::Standard)]),
    );
    let forecast = run(&basis, ForecastLimits::default()).unwrap();
    let mut current = basis.clone().into_parts();
    current.tick = Tick(10);
    current.rules.running_updates = 10;
    current.rules.remaining_ticks = 990;
    let BombKnowledge::Visible(bombs) = &mut current.bombs else {
        unreachable!()
    };
    bombs[0].trigger = crate::cpu::ObservedBombTrigger::Timed {
        remaining_ticks: 90,
        paused: false,
    };
    let current = Observation::new(current).unwrap();
    assert!(forecast.evolution().matches_scene(&basis, &current));

    let mut displaced = current.clone().into_parts();
    let BombKnowledge::Visible(bombs) = &mut displaced.bombs else {
        unreachable!()
    };
    bombs[0].cell = CellId::new(6, 1);
    assert!(
        !forecast
            .evolution()
            .matches_scene(&basis, &Observation::new(displaced).unwrap())
    );

    let mut new_flame = current.into_parts();
    new_flame.flames.push(ObservedFlame {
        cell: CellId::new(2, 2),
        remaining_ticks: 5,
    });
    assert!(
        !forecast
            .evolution()
            .matches_scene(&basis, &Observation::new(new_flame).unwrap())
    );
}

#[test]
fn blast_crossing_burning_oil_does_not_expect_a_finite_flame() {
    let mut basis = observation(
        0,
        BombKnowledge::Visible(vec![timed(9, CellId::new(1, 1), 2, BlastKind::Standard)]),
    )
    .into_parts();
    basis.oil.push(crate::cpu::ObservedOil {
        cell: CellId::new(3, 1),
        state: OilBarrelState::Burning,
    });
    let basis = Observation::new(basis).unwrap();
    let forecast = run(&basis, ForecastLimits::default()).unwrap();
    let mut current = basis.clone().into_parts();
    current.tick = Tick(2);
    current.rules.running_updates = 2;
    current.rules.remaining_ticks = 998;
    current.bombs = BombKnowledge::Visible(Vec::new());
    current.flames = forecast
        .cells()
        .iter()
        .enumerate()
        .filter(|(_, cell)| cell.lethal.iter().any(|window| window.start == Tick(2)))
        .map(|(index, _)| ObservedFlame {
            cell: CellId::new(
                u16::try_from(index % usize::from(ARENA_WIDTH)).unwrap(),
                u16::try_from(index / usize::from(ARENA_WIDTH)).unwrap(),
            ),
            remaining_ticks: 30,
        })
        .collect();
    let current = Observation::new(current).unwrap();

    assert!(forecast.evolution().matches_scene(&basis, &current));
}

#[test]
fn later_blast_crossing_predicted_ignited_oil_stays_persistent() {
    let mut basis = observation(
        0,
        BombKnowledge::Visible(vec![
            timed(1, CellId::new(1, 1), 2, BlastKind::Standard),
            timed(2, CellId::new(5, 1), 40, BlastKind::Standard),
        ]),
    )
    .into_parts();
    basis.oil.push(crate::cpu::ObservedOil {
        cell: CellId::new(3, 1),
        state: OilBarrelState::Unlit,
    });
    let basis = Observation::new(basis).unwrap();
    let forecast = run(&basis, ForecastLimits::default()).unwrap();
    let mut current = basis.clone().into_parts();
    current.tick = Tick(41);
    current.rules.running_updates = 41;
    current.rules.remaining_ticks = 959;
    current.bombs = BombKnowledge::Visible(Vec::new());
    current.oil[0].state = OilBarrelState::Burning;
    current.flames = forecast
        .cells()
        .iter()
        .enumerate()
        .filter(|(_, cell)| cell.lethal.iter().any(|window| window.start == Tick(40)))
        .map(|(index, _)| ObservedFlame {
            cell: CellId::new(
                u16::try_from(index % usize::from(ARENA_WIDTH)).unwrap(),
                u16::try_from(index / usize::from(ARENA_WIDTH)).unwrap(),
            ),
            remaining_ticks: 29,
        })
        .collect();
    let current = Observation::new(current).unwrap();

    assert!(forecast.evolution().matches_scene(&basis, &current));
}

use crate::cpu::{BombKnowledge, Observation};
use crate::game::{BlastKind, CellId, Tick};

use super::tests::{observation, run, timed};
use super::{ForecastFailure, ForecastLimits, HazardForecast, TickWindow};

fn with_second_deadline(remaining: u16) -> HazardForecast {
    let mut parts = observation(
        0,
        BombKnowledge::Visible(vec![
            timed(1, CellId::new(4, 3), 2, BlastKind::Standard),
            timed(2, CellId::new(6, 5), remaining, BlastKind::Standard),
        ]),
    )
    .into_parts();
    parts.arena.blocks.push(CellId::new(4, 5));
    run(&Observation::new(parts).unwrap(), ForecastLimits::default()).unwrap()
}

#[test]
fn later_blast_uses_prior_destruction_but_same_tick_keeps_immutable_blocking() {
    let target = CellId::new(3, 5);
    assert!(
        with_second_deadline(2)
            .cell(target)
            .unwrap()
            .lethal
            .is_empty()
    );
    assert_eq!(
        with_second_deadline(40).cell(target).unwrap().lethal,
        [TickWindow {
            start: Tick(40),
            end: Tick(70)
        }]
    );
}

#[test]
fn destroyed_soft_block_stays_uncertifiable_until_a_fresh_observation() {
    let block = CellId::new(4, 5);
    assert_eq!(
        with_second_deadline(40).cell(block).unwrap().blocking,
        [TickWindow {
            start: Tick(0),
            end: Tick(840)
        }]
    );
}

#[test]
fn interval_capacity_fails_instead_of_truncating_later_danger() {
    let observed = observation(
        0,
        BombKnowledge::Visible(vec![
            timed(1, CellId::new(7, 3), 2, BlastKind::Standard),
            timed(2, CellId::new(4, 6), 40, BlastKind::Standard),
        ]),
    );
    assert_eq!(
        run(
            &observed,
            ForecastLimits {
                horizon_ticks: 100,
                max_intervals_per_cell: 1,
            }
        ),
        Err(ForecastFailure::IntervalCapacity)
    );
}

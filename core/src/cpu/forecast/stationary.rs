use std::collections::{BTreeMap, BTreeSet};
use std::ops::ControlFlow;

use crate::cpu::{Observation, WorkBudget};
use crate::game::blast_geometry::{BlastSource, BlastVisit, visit_blast};
use crate::game::{CellId, EntityId, OilBarrelState, Tick};

use super::evolution::FlameChange;
use super::intervals::{add_window, take};
use super::model::{ForecastBomb, ForecastField, Trigger, index};
use super::{CellForecast, ForecastEvolution, ForecastFailure, ForecastLimits, TickWindow};

#[allow(clippy::too_many_arguments)]
pub(super) fn simulate_stationary(
    observation: &Observation,
    bombs: &[ForecastBomb],
    horizon_end: Tick,
    flame_ticks: u64,
    limits: ForecastLimits,
    work: &mut WorkBudget,
    cells: &mut [CellForecast],
    evolution: &mut ForecastEvolution,
) -> Result<(), ForecastFailure> {
    let parts = observation.parts();
    let mut blocks = parts.arena.blocks.iter().copied().collect::<BTreeSet<_>>();
    let mut oil = parts
        .oil
        .iter()
        .filter_map(|value| (value.state == OilBarrelState::Unlit).then_some(value.cell))
        .collect::<BTreeSet<_>>();
    let mut active = bombs.iter().map(|bomb| bomb.id).collect::<BTreeSet<_>>();
    let mut removals = BTreeMap::new();
    let mut oil_ignitions = BTreeMap::new();
    let mut events: BTreeMap<Tick, BTreeSet<EntityId>> = BTreeMap::new();
    seed_events(observation, bombs, horizon_end, &mut events);

    while let Some((tick, seeds)) = events.pop_first() {
        take(work, 1)?;
        let snapshot = active
            .iter()
            .filter_map(|id| bombs.iter().find(|bomb| bomb.id == *id).copied())
            .filter(|bomb| bomb.active_from <= tick)
            .map(|bomb| (bomb.cell, bomb))
            .collect::<BTreeMap<_, _>>();
        let field = ForecastField {
            observation,
            blocks: &blocks,
            oil: &oil,
            bombs: &snapshot,
        };
        let phase = plan_phase(&field, bombs, &active, seeds, work)?;
        let removal_end = Tick(
            tick.0
                .checked_add(1)
                .ok_or(ForecastFailure::ClockOverflow)?,
        );
        for id in phase.detonated.iter().chain(&phase.destroyed_mines) {
            active.remove(id);
            removals.entry(*id).or_insert(removal_end);
        }
        for cell in &phase.destroyed_blocks {
            blocks.remove(cell);
        }
        for cell in &phase.ignited_oil {
            oil_ignitions.entry(*cell).or_insert(removal_end);
            oil.remove(cell);
        }
        let physical_end = Tick(
            tick.0
                .checked_add(flame_ticks)
                .ok_or(ForecastFailure::ClockOverflow)?,
        );
        let finite_end = physical_end.min(horizon_end);
        evolution.record(
            tick,
            phase
                .detonated
                .iter()
                .chain(&phase.destroyed_mines)
                .copied(),
            phase.destroyed_blocks.iter().copied(),
            phase.ignited_oil.iter().copied(),
            phase.flames.iter().copied().map(|cell| FlameChange {
                at: tick,
                cell,
                expires_at: (!phase.ignited_oil.contains(&cell)
                    && !oil_ignitions.contains_key(&cell)
                    && !observation.parts().oil.iter().any(|barrel| {
                        barrel.cell == cell && barrel.state == OilBarrelState::Burning
                    }))
                .then_some(physical_end),
            }),
            work,
        )?;
        for cell in phase.flames {
            add_window(
                &mut cells[index(cell)].lethal,
                TickWindow {
                    start: tick,
                    end: if phase.ignited_oil.contains(&cell) {
                        horizon_end
                    } else {
                        finite_end
                    },
                },
                limits,
                work,
            )?;
        }
    }
    add_blocking_windows(
        observation,
        bombs,
        horizon_end,
        &removals,
        &oil_ignitions,
        evolution,
        limits,
        work,
        cells,
    )
}

fn seed_events(
    observation: &Observation,
    bombs: &[ForecastBomb],
    horizon_end: Tick,
    events: &mut BTreeMap<Tick, BTreeSet<EntityId>>,
) {
    for bomb in bombs {
        if let Trigger::Timed(at) = bomb.trigger
            && at.0 < horizon_end.0
        {
            events.entry(at).or_default().insert(bomb.id);
        }
        if !matches!(bomb.trigger, Trigger::Contact) {
            let ignition = if bomb.hypothetical {
                observation
                    .parts()
                    .flames
                    .iter()
                    .any(|flame| {
                        flame.cell == bomb.cell
                            && observation
                                .parts()
                                .tick
                                .0
                                .checked_add(u64::from(flame.remaining_ticks))
                                .is_some_and(|expiry| expiry > bomb.active_from.0)
                    })
                    .then_some(bomb.active_from)
            } else {
                observation
                    .parts()
                    .flames
                    .iter()
                    .any(|flame| flame.cell == bomb.cell && flame.remaining_ticks > 1)
                    .then_some(Tick(observation.parts().tick.0 + 1))
            };
            if let Some(at) = ignition {
                events.entry(at).or_default().insert(bomb.id);
            }
        }
    }
}

#[derive(Default)]
struct Phase {
    detonated: BTreeSet<EntityId>,
    destroyed_mines: BTreeSet<EntityId>,
    destroyed_blocks: BTreeSet<CellId>,
    ignited_oil: BTreeSet<CellId>,
    flames: BTreeSet<CellId>,
}

fn plan_phase(
    field: &ForecastField<'_>,
    bombs: &[ForecastBomb],
    active: &BTreeSet<EntityId>,
    mut pending: BTreeSet<EntityId>,
    work: &mut WorkBudget,
) -> Result<Phase, ForecastFailure> {
    let mut phase = Phase::default();
    while let Some(id) = pending.pop_first() {
        if !active.contains(&id) || !phase.detonated.insert(id) {
            continue;
        }
        let bomb = *bombs
            .iter()
            .find(|bomb| bomb.id == id)
            .expect("active forecast bomb exists");
        let result = visit_blast(
            field,
            BlastSource {
                cell: bomb.cell,
                range: bomb.range,
                kind: bomb.blast,
            },
            |visit| {
                if take(work, 1).is_err() {
                    return ControlFlow::Break(());
                }
                apply_visit(visit, &mut pending, &mut phase);
                ControlFlow::Continue(())
            },
        );
        if result.is_break() {
            return Err(ForecastFailure::WorkExhausted);
        }
    }
    Ok(phase)
}

fn apply_visit(visit: BlastVisit, pending: &mut BTreeSet<EntityId>, phase: &mut Phase) {
    phase.flames.insert(visit.cell);
    if visit.soft_block {
        phase.destroyed_blocks.insert(visit.cell);
    }
    if visit.unlit_oil {
        phase.ignited_oil.insert(visit.cell);
    }
    if let Some(bomb) = visit.bomb {
        if bomb.contact {
            phase.destroyed_mines.insert(bomb.id);
        } else {
            pending.insert(bomb.id);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn add_blocking_windows(
    observation: &Observation,
    bombs: &[ForecastBomb],
    horizon_end: Tick,
    removals: &BTreeMap<EntityId, Tick>,
    oil_ignitions: &BTreeMap<CellId, Tick>,
    evolution: &ForecastEvolution,
    limits: ForecastLimits,
    work: &mut WorkBudget,
    cells: &mut [CellForecast],
) -> Result<(), ForecastFailure> {
    let basis = observation.parts().tick;
    let pass_blocks = crate::cpu::capabilities::can_pass_known_soft_blocks(observation);
    let mut block_removals = BTreeMap::new();
    if pass_blocks {
        for (cell, tick) in evolution.block_removals() {
            take(work, 1)?;
            block_removals.entry(cell).or_insert(tick);
        }
    }
    for bomb in bombs {
        if !matches!(bomb.trigger, Trigger::Contact) {
            add_window(
                &mut cells[index(bomb.cell)].blocking,
                TickWindow {
                    start: if bomb.hypothetical {
                        Tick(
                            bomb.active_from
                                .0
                                .checked_add(1)
                                .ok_or(ForecastFailure::ClockOverflow)?,
                        )
                    } else {
                        basis
                    },
                    end: removals.get(&bomb.id).copied().unwrap_or(horizon_end),
                },
                limits,
                work,
            )?;
        }
    }
    for cell in &observation.parts().arena.blocks {
        let start = if pass_blocks {
            let Some(at) = block_removals.get(cell).copied() else {
                continue;
            };
            at
        } else {
            basis
        };
        add_window(
            &mut cells[index(*cell)].blocking,
            TickWindow {
                start,
                // A destroyed block can reveal concealed Oil. Recipient facts
                // cannot certify this cell traversable until another snapshot.
                end: horizon_end,
            },
            limits,
            work,
        )?;
    }
    for cell in observation
        .parts()
        .oil
        .iter()
        .filter_map(|oil| (oil.state == OilBarrelState::Unlit).then_some(oil.cell))
    {
        add_window(
            &mut cells[index(cell)].blocking,
            TickWindow {
                start: basis,
                end: oil_ignitions.get(&cell).copied().unwrap_or(horizon_end),
            },
            limits,
            work,
        )?;
    }
    Ok(())
}

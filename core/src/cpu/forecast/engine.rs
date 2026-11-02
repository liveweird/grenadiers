use std::collections::BTreeSet;

use crate::cpu::{BombKnowledge, MemoryKey, Observation, ObservedBombTrigger, WorkBudget};
use crate::game::arena::Terrain;
use crate::game::{OilBarrelState, RulesConfig, Tick};

use super::intervals::{add_window, take};
use super::model::{ForecastBomb, Trigger, index, terrain, unused_id};
use super::stationary::simulate_stationary;
use super::uncertainty::add_uncertainties;
use super::{
    CellForecast, ForecastEvolution, ForecastFailure, ForecastLimits, HazardForecast,
    HypotheticalPlacement, ObservedHazardMemory, TickWindow,
};

pub fn forecast(
    observation: &Observation,
    memory: &ObservedHazardMemory,
    hypothetical: Option<HypotheticalPlacement>,
    limits: ForecastLimits,
    work: &mut WorkBudget,
) -> Result<HazardForecast, ForecastFailure> {
    validate_limits(limits)?;
    let parts = observation.parts();
    let board = parts
        .arena
        .board()
        .ok_or(ForecastFailure::UnsupportedRules)?;
    let cell_count = board.cell_count();
    let key = MemoryKey {
        world: parts.world,
        cpu_seat: parts.recipient,
    };
    if memory.key() != key {
        return Err(ForecastFailure::WrongWorld);
    }
    let rules = crate::cpu::rules_config(observation).ok_or(ForecastFailure::UnsupportedRules)?;
    let horizon_end = Tick(
        parts
            .tick
            .0
            .checked_add(u64::from(limits.horizon_ticks))
            .ok_or(ForecastFailure::ClockOverflow)?,
    );
    let BombKnowledge::Visible(observed_bombs) = &parts.bombs else {
        return Err(ForecastFailure::BombKnowledgeUnavailable);
    };
    validate_observed(observation, rules, hypothetical, horizon_end)?;
    take(work, cell_count as u32)?;

    let mut cells = vec![CellForecast::default(); cell_count];
    let mut bomb_occupancy = super::BombOccupancyIndex::default();
    let mut evolution = ForecastEvolution::new(observation, horizon_end)?;
    let mut bombs = observed_bombs
        .iter()
        .map(|bomb| {
            let trigger = match bomb.trigger {
                ObservedBombTrigger::Timed {
                    remaining_ticks,
                    paused,
                } => {
                    let at = Tick(
                        parts
                            .tick
                            .0
                            .checked_add(u64::from(remaining_ticks))
                            .ok_or(ForecastFailure::ClockOverflow)?,
                    );
                    if paused {
                        Trigger::Paused(at)
                    } else {
                        Trigger::Timed(at)
                    }
                }
                ObservedBombTrigger::Remote => Trigger::Remote,
                ObservedBombTrigger::Contact => Trigger::Contact,
            };
            Ok(ForecastBomb {
                id: bomb.id,
                cell: bomb.cell,
                range: bomb.range,
                blast: bomb.blast,
                trigger,
                hypothetical: false,
                active_from: parts.tick,
            })
        })
        .collect::<Result<Vec<_>, ForecastFailure>>()?;
    let mut hypothetical_id = None;
    if let Some(placement) = hypothetical {
        let id = unused_id(&bombs);
        bombs.push(ForecastBomb {
            id,
            cell: placement.cell,
            range: placement.range,
            blast: placement.blast,
            trigger: Trigger::Timed(placement.detonates_at),
            hypothetical: true,
            active_from: placement.placed_at,
        });
        evolution.add_hypothetical(id, placement.owner)?;
        hypothetical_id = Some(id);
    }

    add_existing_hazards(observation, horizon_end, limits, work, &mut cells)?;
    let mut assumptions = observed_bombs
        .iter()
        .filter_map(|bomb| (!memory.was_displaced(bomb.id)).then_some(bomb.id))
        .collect::<Vec<_>>();
    assumptions.sort_unstable();
    simulate_stationary(
        observation,
        &bombs,
        horizon_end,
        rules.flame_ticks(),
        limits,
        work,
        &mut cells,
        &mut evolution,
        &mut bomb_occupancy,
    )?;
    add_uncertainties(observation, &bombs, horizon_end, limits, work, &mut cells)?;
    take(
        work,
        u32::try_from(bomb_occupancy.len()).map_err(|_| ForecastFailure::IntervalCapacity)?,
    )?;
    bomb_occupancy.finish();
    HazardForecast::from_parts(
        key,
        board,
        parts.tick,
        horizon_end,
        cells,
        bomb_occupancy,
        assumptions,
        evolution,
        hypothetical_id,
        limits,
    )
}

fn validate_limits(limits: ForecastLimits) -> Result<(), ForecastFailure> {
    if limits.horizon_ticks == 0
        || limits.horizon_ticks > super::MAX_FORECAST_HORIZON_TICKS
        || limits.max_intervals_per_cell == 0
        || limits.max_intervals_per_cell > super::MAX_INTERVALS_PER_CELL
    {
        return Err(ForecastFailure::InvalidLimits);
    }
    Ok(())
}

fn validate_observed(
    observation: &Observation,
    rules: RulesConfig,
    hypothetical: Option<HypotheticalPlacement>,
    horizon_end: Tick,
) -> Result<(), ForecastFailure> {
    let parts = observation.parts();
    let BombKnowledge::Visible(bombs) = &parts.bombs else {
        return Ok(());
    };
    if parts.flames.iter().any(|flame| flame.remaining_ticks == 0) {
        return Err(ForecastFailure::InvalidObservedHazards);
    }
    let mut occupied = BTreeSet::new();
    if bombs.iter().any(|bomb| {
        bomb.range == 0
            || bomb.range > rules.bomb_range_cap()
            || terrain(parts, bomb.cell) != Some(Terrain::Floor)
            || !occupied.insert(bomb.cell)
            || matches!(
                bomb.trigger,
                ObservedBombTrigger::Timed {
                    remaining_ticks: 0,
                    ..
                }
            )
    }) {
        return Err(ForecastFailure::InvalidObservedHazards);
    }
    if hypothetical.is_some_and(|placement| {
        placement.range == 0
            || placement.range > rules.bomb_range_cap()
            || placement.detonates_at.0 <= parts.tick.0
            || placement.detonates_at.0 >= horizon_end.0
            || placement.placed_at.0 <= parts.tick.0
            || placement.placed_at >= placement.detonates_at
            || terrain(parts, placement.cell) != Some(Terrain::Floor)
            || occupied.contains(&placement.cell)
            || parts.arena.blocks.contains(&placement.cell)
            || parts.oil.iter().any(|oil| oil.cell == placement.cell)
    }) {
        return Err(ForecastFailure::InvalidHypothetical);
    }
    Ok(())
}

fn add_existing_hazards(
    observation: &Observation,
    horizon_end: Tick,
    limits: ForecastLimits,
    work: &mut WorkBudget,
    cells: &mut [CellForecast],
) -> Result<(), ForecastFailure> {
    let basis = observation.parts().tick;
    let width = observation.parts().arena.width;
    for flame in &observation.parts().flames {
        let end = Tick(
            basis
                .0
                .checked_add(u64::from(flame.remaining_ticks))
                .ok_or(ForecastFailure::ClockOverflow)?,
        )
        .min(horizon_end);
        add_window(
            &mut cells[index(flame.cell, width)].lethal,
            TickWindow { start: basis, end },
            limits,
            work,
        )?;
    }
    for oil in &observation.parts().oil {
        if oil.state == OilBarrelState::Burning {
            add_window(
                &mut cells[index(oil.cell, width)].lethal,
                TickWindow {
                    start: basis,
                    end: horizon_end,
                },
                limits,
                work,
            )?;
        }
    }
    Ok(())
}

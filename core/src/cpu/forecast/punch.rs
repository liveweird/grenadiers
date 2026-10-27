use crate::cpu::{BombKnowledge, Observation, WorkBudget};
use crate::game::{BlastKind, CellId, EntityId, Tick};

use super::{
    ForecastFailure, ForecastLimits, HazardForecast, HypotheticalPlacement, ObservedHazardMemory,
    forecast,
};

pub(crate) struct PunchForecastBranch {
    forecast: HazardForecast,
    scenario_basis: Observation,
    hypothetical_id: EntityId,
}

impl PunchForecastBranch {
    #[must_use]
    pub(crate) const fn forecast(&self) -> &HazardForecast {
        &self.forecast
    }

    #[must_use]
    pub(crate) const fn scenario_basis(&self) -> &Observation {
        &self.scenario_basis
    }

    #[must_use]
    pub(crate) const fn hypothetical_id(&self) -> EntityId {
        self.hypothetical_id
    }
}

pub(crate) fn forecast_punch_branch(
    original: &Observation,
    history: &ObservedHazardMemory,
    source: EntityId,
    landing: CellId,
    execution_tick: Tick,
    limits: ForecastLimits,
    work: &mut WorkBudget,
) -> Result<PunchForecastBranch, ForecastFailure> {
    let parts = original.parts();
    if !parts.flames.is_empty() || !parts.oil.is_empty() || !parts.pickups.is_empty() {
        return Err(ForecastFailure::UnsupportedRules);
    }
    let BombKnowledge::Visible(bombs) = &parts.bombs else {
        return Err(ForecastFailure::BombKnowledgeUnavailable);
    };
    let [bomb] = bombs.as_slice() else {
        return Err(ForecastFailure::InvalidHypothetical);
    };
    let crate::cpu::ObservedBombTrigger::Timed {
        remaining_ticks,
        paused: false,
    } = bomb.trigger
    else {
        return Err(ForecastFailure::UnsupportedRules);
    };
    if bomb.id != source || bomb.blast != BlastKind::Standard || landing == bomb.cell {
        return Err(ForecastFailure::InvalidHypothetical);
    }
    let detonates_at = Tick(
        parts
            .tick
            .0
            .checked_add(u64::from(remaining_ticks))
            .ok_or(ForecastFailure::ClockOverflow)?,
    );
    if execution_tick <= parts.tick || execution_tick >= detonates_at {
        return Err(ForecastFailure::InvalidHypothetical);
    }

    let mut scenario_parts = original.clone().into_parts();
    scenario_parts.bombs = BombKnowledge::Visible(Vec::new());
    let scenario_basis =
        Observation::new(scenario_parts).map_err(|_| ForecastFailure::InvalidObservedHazards)?;
    let forecast = forecast(
        &scenario_basis,
        history,
        Some(HypotheticalPlacement {
            placed_at: execution_tick,
            owner: bomb.owner,
            cell: landing,
            range: bomb.range,
            blast: bomb.blast,
            detonates_at,
        }),
        limits,
        work,
    )?;
    let hypothetical_id = forecast
        .hypothetical_id()
        .ok_or(ForecastFailure::InvalidHypothetical)?;
    Ok(PunchForecastBranch {
        forecast,
        scenario_basis,
        hypothetical_id,
    })
}

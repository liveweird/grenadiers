use std::{collections::BTreeSet, fmt};

use grenadiers_core::{
    cpu::{
        Observation, ObservedBombTrigger, WorkBudget,
        forecast::{ForecastFailure, ForecastLimits, ObservedHazardMemory, forecast},
        policy::UtilityMemory,
        survival::{
            OriginRecoveryChoice, PlantEscapeOrigin, plan_continuation_lairs_avoiding_pits,
        },
    },
    game::{CellId, OilBarrelState, SeatId, Tick},
};

use crate::game::rooms::cpu::PlanningBasis;

const DIAGNOSTIC_ALLOWANCE: u32 = 65_536;
const PLACED_AT: Tick = Tick(862);
const DETONATES_AT: Tick = Tick(1_042);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SelectorResult {
    None,
    Continuation,
    Pulse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SelectorProbe {
    pub result: SelectorResult,
    pub work: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ForecastResult {
    Ok { basis_tick: Tick, horizon_end: Tick },
    Err(ForecastFailure),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ForecastProbe {
    pub result: ForecastResult,
    pub work: u32,
}

#[derive(Clone)]
pub(super) struct QueuedBasis {
    pub seat: SeatId,
    pub observation: Observation,
    pub hazards: ObservedHazardMemory,
    pub memory: UtilityMemory,
    pub settled_stop: Tick,
    pub origin: PlantEscapeOrigin,
    pub selector: SelectorProbe,
    pub forecast: ForecastProbe,
}

impl QueuedBasis {
    pub(super) const fn origin(&self) -> &PlantEscapeOrigin {
        &self.origin
    }
}

impl fmt::Debug for QueuedBasis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("QueuedBasis")
            .field("seat", &self.seat)
            .field("observation", &self.observation)
            .field("hazards", &self.hazards)
            .field("memory", &self.memory)
            .field("settled_stop", &self.settled_stop)
            .field("origin", &"<captured actual PlantEscapeOrigin>")
            .field("selector", &self.selector)
            .field("forecast", &self.forecast)
            .finish()
    }
}

pub(super) fn capture(basis: PlanningBasis<'_>, with_oil: bool) -> Option<QueuedBasis> {
    exact_trigger_basis(basis.observation, with_oil)?;
    let origin = basis.escape_origin?.clone();

    let mut selector_work = WorkBudget::new(DIAGNOSTIC_ALLOWANCE);
    let selected = plan_continuation_lairs_avoiding_pits(
        basis.observation,
        basis.hazard_memory,
        basis.settled_stop_tick,
        basis.memory,
        &origin,
        &mut selector_work,
    );
    let selector = SelectorProbe {
        result: match selected {
            Some(OriginRecoveryChoice::Continuation(_)) => SelectorResult::Continuation,
            Some(OriginRecoveryChoice::Pulse(_)) => SelectorResult::Pulse,
            None => SelectorResult::None,
        },
        work: selector_work.spent(),
    };

    let mut forecast_work = WorkBudget::new(DIAGNOSTIC_ALLOWANCE);
    let forecast_result = forecast(
        basis.observation,
        basis.hazard_memory,
        None,
        ForecastLimits::default(),
        &mut forecast_work,
    );
    let forecast = ForecastProbe {
        result: match forecast_result {
            Ok(forecast) => ForecastResult::Ok {
                basis_tick: forecast.basis_tick(),
                horizon_end: forecast.horizon_end(),
            },
            Err(error) => ForecastResult::Err(error),
        },
        work: forecast_work.spent(),
    };

    Some(QueuedBasis {
        seat: basis.observation.parts().recipient,
        observation: basis.observation.clone(),
        hazards: basis.hazard_memory.clone(),
        memory: *basis.memory,
        settled_stop: basis.settled_stop_tick,
        origin,
        selector,
        forecast,
    })
}

fn exact_trigger_basis(observation: &Observation, with_oil: bool) -> Option<()> {
    let grenadiers_core::cpu::BombKnowledge::Visible(bombs) = &observation.parts().bombs else {
        return None;
    };
    let cells = bombs.iter().map(|bomb| bomb.cell).collect::<BTreeSet<_>>();
    let expected_cells =
        BTreeSet::from([CellId::new(13, 3), CellId::new(3, 11), CellId::new(11, 11)]);
    let trigger_matches =
        bombs.len() == 3 && cells == expected_cells && bombs.iter().all(|bomb| {
            bomb.placed_at == PLACED_AT
                && bomb.range == 3
                && matches!(
                    bomb.trigger,
                    ObservedBombTrigger::Timed {
                        remaining_ticks,
                        paused: false,
                    } if observation.parts().tick.0 + u64::from(remaining_ticks) == DETONATES_AT.0
                )
        });
    let oil = observation.parts().oil.as_slice();
    let oil_matches = if with_oil {
        oil == [
            grenadiers_core::cpu::ObservedOil {
                cell: CellId::new(10, 1),
                state: OilBarrelState::Unlit,
            },
            grenadiers_core::cpu::ObservedOil {
                cell: CellId::new(13, 8),
                state: OilBarrelState::Unlit,
            },
        ]
    } else {
        oil.is_empty()
    };
    (trigger_matches && oil_matches).then_some(())
}

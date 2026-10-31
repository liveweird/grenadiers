use crate::cpu::forecast::TickWindow;
use crate::cpu::routing::{CERTIFIED_MOVE_UPDATES, MIN_FINAL_STOP_UPDATES};
use crate::cpu::{BombKnowledge, Observation, WorkBudget};
use crate::game::{OilBarrelState, Tick};

/// Keeps ordinary routes only as long as their currently visible finite danger.
/// Forecast construction remains full-horizon so every shorter proof uses the
/// same conservative public evolution model.
pub(super) fn ordinary_route_end(
    observation: &Observation,
    start_window: TickWindow,
    horizon_end: Tick,
    work: &mut WorkBudget,
) -> Option<Tick> {
    let parts = observation.parts();
    if parts.rules.version == 10 {
        return Some(horizon_end);
    }
    if !matches!(&parts.bombs, BombKnowledge::Visible(bombs) if bombs.is_empty()) {
        return Some(horizon_end);
    }
    for oil in &parts.oil {
        work.try_take(1).ok()?;
        if oil.state == OilBarrelState::Burning {
            return Some(horizon_end);
        }
    }
    let latest_start = start_window.end.0.checked_sub(1)?;
    let minimum_end = latest_start
        .checked_add(u64::from(CERTIFIED_MOVE_UPDATES))?
        .checked_add(MIN_FINAL_STOP_UPDATES)?;
    let mut flame_end = parts.tick.0;
    for flame in &parts.flames {
        work.try_take(1).ok()?;
        flame_end = flame_end.max(parts.tick.0.checked_add(u64::from(flame.remaining_ticks))?);
    }
    Some(Tick(minimum_end.max(flame_end).min(horizon_end.0)))
}

pub(super) fn ordinary_movement_is_clear(
    observation: &Observation,
    movement_is_nonstop: bool,
    work: &mut WorkBudget,
) -> bool {
    let parts = observation.parts();
    if parts.rules.version == 10
        || !movement_is_nonstop
        || !matches!(&parts.bombs, BombKnowledge::Visible(bombs) if bombs.is_empty())
        || !parts.flames.is_empty()
    {
        return false;
    }
    for oil in &parts.oil {
        if work.try_take(1).is_err() || oil.state == OilBarrelState::Burning {
            return false;
        }
    }
    true
}

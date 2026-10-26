use crate::cpu::WorkBudget;

use super::{ForecastFailure, ForecastLimits, TickWindow, UncertainWindow, UncertaintyReason};

pub(super) fn take(work: &mut WorkBudget, units: u32) -> Result<(), ForecastFailure> {
    work.try_take(units)
        .map_err(|_| ForecastFailure::WorkExhausted)
}

pub(super) fn add_window(
    windows: &mut Vec<TickWindow>,
    window: TickWindow,
    limits: ForecastLimits,
    work: &mut WorkBudget,
) -> Result<(), ForecastFailure> {
    if window.start.0 >= window.end.0 {
        return Ok(());
    }
    take(work, 1)?;
    windows.push(window);
    windows.sort_by_key(|value| (value.start.0, value.end.0));
    let mut merged: Vec<TickWindow> = Vec::with_capacity(windows.len());
    for current in windows.drain(..) {
        if let Some(previous) = merged.last_mut()
            && current.start.0 <= previous.end.0
        {
            previous.end = previous.end.max(current.end);
        } else {
            merged.push(current);
        }
    }
    if merged.len() > usize::from(limits.max_intervals_per_cell) {
        return Err(ForecastFailure::IntervalCapacity);
    }
    *windows = merged;
    Ok(())
}

pub(super) fn add_uncertain(
    windows: &mut Vec<UncertainWindow>,
    window: UncertainWindow,
    limits: ForecastLimits,
    work: &mut WorkBudget,
) -> Result<(), ForecastFailure> {
    if window.window.start.0 >= window.window.end.0 {
        return Ok(());
    }
    take(work, 1)?;
    windows.push(window);
    windows.sort_by_key(|value| {
        (
            reason_order(value.reason),
            kind_order(value.kind),
            value.window.start.0,
            value.window.end.0,
        )
    });
    let mut merged: Vec<UncertainWindow> = Vec::with_capacity(windows.len());
    for current in windows.drain(..) {
        if let Some(previous) = merged.last_mut()
            && previous.kind == current.kind
            && previous.reason == current.reason
            && current.window.start.0 <= previous.window.end.0
        {
            previous.window.end = previous.window.end.max(current.window.end);
        } else {
            merged.push(current);
        }
    }
    if merged.len() > usize::from(limits.max_intervals_per_cell) {
        return Err(ForecastFailure::IntervalCapacity);
    }
    merged.sort_by_key(|value| {
        (
            value.window.start.0,
            value.window.end.0,
            reason_order(value.reason),
        )
    });
    *windows = merged;
    Ok(())
}

const fn kind_order(kind: super::HazardKind) -> u8 {
    match kind {
        super::HazardKind::Lethal => 0,
        super::HazardKind::Blocking => 1,
    }
}

fn reason_order(reason: UncertaintyReason) -> (u8, u32) {
    match reason {
        UncertaintyReason::BombMotion(id) => (0, id.0),
        UncertaintyReason::Trigger(id) => (1, id.0),
        UncertaintyReason::Pause(id) => (2, id.0),
    }
}

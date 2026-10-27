use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::policy::{ControlPlan, UtilityMemory};
use crate::cpu::routing::{
    PulseLease, PulseOriginGrace, PulseRequest, RouteMode, certify_escape_pulse,
};
use crate::cpu::{Intent, MovementIntent, Observation, WorkBudget};
use crate::game::{Direction, Tick};

use super::{MAX_PLAN_AGE_TICKS, PlantEscapeOrigin, SurvivalPlan};

pub(super) fn plan(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    origin: &PlantEscapeOrigin,
    work: &mut WorkBudget,
) -> Option<SurvivalPlan> {
    origin.retain_stopped(observation)?;
    let forecast = forecast(observation, history, None, ForecastLimits::default(), work).ok()?;
    let basis = observation.parts().tick.0;
    let start_window = TickWindow {
        start: Tick(basis.checked_add(1)?),
        end: Tick(basis.checked_add(MAX_PLAN_AGE_TICKS + 2)?),
    };
    let mut best: Option<(u32, SurvivalPlan)> = None;
    for mode in [
        RouteMode::CertifiedOnly,
        RouteMode::AllowStationaryAssumptions,
    ] {
        let hold_ticks = if mode == RouteMode::CertifiedOnly {
            12
        } else {
            3
        };
        let until = start_window.end.0.checked_add(u64::from(hold_ticks) + 9)?;
        for direction in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            let movement = MovementIntent::Direction(direction);
            let request = PulseRequest {
                movement,
                hold_ticks,
                start_window,
                stopped_safe_until: Tick(until),
                mode,
                settled_stop_tick,
            };
            let Ok(lease) = certify_escape_pulse(
                observation,
                &forecast,
                request,
                PulseOriginGrace {
                    cell: origin.cell(),
                    bomb_id: origin.bomb_id(),
                },
                work,
            ) else {
                continue;
            };
            let distance = distance_from_origin(&lease, origin.cell());
            let candidate = SurvivalPlan {
                control: ControlPlan {
                    intent: Intent {
                        movement,
                        manual: None,
                    },
                    hold_ticks,
                    next_memory: UtilityMemory::new(memory.key()),
                },
                lease: Some(lease),
                execution: None,
            };
            if best
                .as_ref()
                .is_none_or(|(previous, _)| distance > *previous)
            {
                best = Some((distance, candidate));
            }
        }
        if best.is_some() {
            break;
        }
    }
    best.map(|(_, plan)| plan)
}

fn distance_from_origin(lease: &PulseLease, origin: crate::game::CellId) -> u32 {
    let bounds = lease.pose_after(lease.hold_ticks());
    let center = crate::game::Position::center(origin);
    let dx = if center.x < bounds.min().x {
        center.x.abs_diff(bounds.min().x)
    } else if center.x > bounds.max().x {
        center.x.abs_diff(bounds.max().x)
    } else {
        0
    };
    let dy = if center.y < bounds.min().y {
        center.y.abs_diff(bounds.min().y)
    } else if center.y > bounds.max().y {
        center.y.abs_diff(bounds.max().y)
    } else {
        0
    };
    dx.saturating_add(dy)
}

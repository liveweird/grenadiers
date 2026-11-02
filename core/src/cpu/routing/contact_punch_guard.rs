use crate::cpu::forecast::{HazardForecast, TickWindow, UncertaintyReason};
use crate::cpu::{Observation, WorkBudget};
use crate::game::geometry::{MovementSegment, distance_to_blocking_cell};
use crate::game::{CellId, Direction, PunchActivationPolicy, Tick};

use super::motion::Motion;
use super::safety::footprint_cells;
use super::{RouteFailure, route_time::add_tick};

pub(super) fn check_motion(
    observation: &Observation,
    forecasts: &[&HazardForecast],
    motion: &Motion,
    tick: Tick,
    origin_grace: Option<CellId>,
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    let Some(forward) = motion.forward_segment() else {
        return Ok(());
    };
    check_forward(observation, forecasts, forward, tick, origin_grace, work)
}

pub(super) fn check_forward(
    observation: &Observation,
    forecasts: &[&HazardForecast],
    forward: MovementSegment,
    tick: Tick,
    origin_grace: Option<CellId>,
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    if observation.parts().rules.punch_activation_policy != PunchActivationPolicy::ContactV2 {
        return Ok(());
    }
    work.try_take(
        u32::try_from(observation.parts().actors.len()).map_err(|_| RouteFailure::Capacity)?,
    )
    .map_err(|_| RouteFailure::WorkExhausted)?;
    let can_punch = observation
        .parts()
        .rules
        .settings
        .enabled_items()
        .contains_punch()
        && observation.parts().actors.iter().any(|actor| {
            matches!(actor, crate::cpu::ObservedActor::Visible(actor)
                if actor.seat == observation.parts().recipient
                    && actor.equipment == Some(crate::game::Equipment::Punch))
        });
    if !can_punch {
        return Ok(());
    }
    let Some(direction) = segment_direction(forward) else {
        return Ok(());
    };
    let extended = MovementSegment::new(forward.end(), translated_one(forward.end(), direction));
    let dimensions = (
        observation.parts().arena.width,
        observation.parts().arena.height,
    );
    let candidates = footprint_cells(&[extended], dimensions)
        .into_iter()
        .filter(|cell| distance_to_blocking_cell(forward.end(), direction, *cell) == Some(0))
        .collect::<Vec<_>>();
    let units = candidates
        .len()
        .checked_mul(forecasts.len())
        .and_then(|value| u32::try_from(value).ok())
        .ok_or(RouteFailure::Capacity)?;
    work.try_take(units.max(1))
        .map_err(|_| RouteFailure::WorkExhausted)?;
    let window = TickWindow {
        start: tick,
        end: add_tick(tick, 1)?,
    };
    for candidate in candidates {
        if origin_grace == Some(candidate)
            && crate::game::geometry::position_overlaps_cell(forward.start(), candidate)
        {
            continue;
        }
        for forecast in forecasts {
            let cell = forecast
                .cell(candidate)
                .ok_or(RouteFailure::InvalidRequest)?;
            if forecast.bomb_blocker(candidate, window).is_some()
                || cell.uncertainty.iter().any(|value| {
                    intersects(value.window, window)
                        && matches!(value.reason, UncertaintyReason::BombMotion(id)
                            if forecast.punchable_bomb(id, window))
                })
            {
                return Err(RouteFailure::NoRoute);
            }
        }
    }
    Ok(())
}

fn segment_direction(segment: MovementSegment) -> Option<Direction> {
    let start = segment.start();
    let end = segment.end();
    match (end.x.cmp(&start.x), end.y.cmp(&start.y)) {
        (std::cmp::Ordering::Greater, std::cmp::Ordering::Equal) => Some(Direction::East),
        (std::cmp::Ordering::Less, std::cmp::Ordering::Equal) => Some(Direction::West),
        (std::cmp::Ordering::Equal, std::cmp::Ordering::Greater) => Some(Direction::South),
        (std::cmp::Ordering::Equal, std::cmp::Ordering::Less) => Some(Direction::North),
        _ => None,
    }
}

fn translated_one(position: crate::game::Position, direction: Direction) -> crate::game::Position {
    let (dx, dy) = match direction {
        Direction::North => (0, -1),
        Direction::East => (1, 0),
        Direction::South => (0, 1),
        Direction::West => (-1, 0),
    };
    crate::game::Position {
        x: position.x + dx,
        y: position.y + dy,
    }
}

fn intersects(left: TickWindow, right: TickWindow) -> bool {
    left.start < right.end && right.start < left.end
}

#[cfg(test)]
mod tests {
    use crate::cpu::forecast::{ForecastFailure, ForecastLimits, ObservedHazardMemory, forecast};
    use crate::cpu::{
        BombKnowledge, ObservedBomb, ObservedBombTrigger, ObservedFlame, UnknownBombFacts,
        WorkBudget,
    };
    use crate::game::types::{HALF_FOOTPRINT, UNITS_PER_CELL};
    use crate::game::{BlastKind, EntityId, Position};

    use super::*;

    fn scene(policy: PunchActivationPolicy) -> (Observation, HazardForecast) {
        let mut parts =
            crate::cpu::routing::tests::observation(Vec::new(), Vec::new()).into_parts();
        parts.rules.punch_activation_policy = policy;
        parts.rules.settings = parts
            .rules
            .settings
            .with_enabled_items(crate::game::EnabledItems::PUNCH);
        let crate::cpu::ObservedActor::Visible(actor) = &mut parts.actors[1] else {
            unreachable!()
        };
        actor.equipment = Some(crate::game::Equipment::Punch);
        parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
            id: EntityId(90),
            placed_at: Tick(0),
            cell: CellId::new(3, 1),
            owner: EntityId(1),
            range: 2,
            trigger: ObservedBombTrigger::Timed {
                remaining_ticks: 180,
                paused: false,
            },
            blast: BlastKind::Standard,
            unknown: UnknownBombFacts::ALL,
        }]);
        let observation = Observation::new(parts).unwrap();
        let forecast = crate::cpu::routing::tests::forecast_for(&observation);
        (observation, forecast)
    }

    fn east_to(x: i32) -> MovementSegment {
        MovementSegment::new(
            Position {
                x: x - 64,
                y: 1_536,
            },
            Position { x, y: 1_536 },
        )
    }

    #[test]
    fn contact_policy_closes_the_exact_bomb_boundary_only() {
        let (observation, forecast) = scene(PunchActivationPolicy::ContactV2);
        let tangent = 3 * UNITS_PER_CELL - HALF_FOOTPRINT;
        let mut exact_work = WorkBudget::new(3);
        assert_eq!(
            check_forward(
                &observation,
                &[&forecast],
                east_to(tangent),
                Tick(1),
                None,
                &mut exact_work,
            ),
            Err(RouteFailure::NoRoute)
        );
        assert_eq!(exact_work.spent(), 3);
        let mut short_work = WorkBudget::new(3);
        assert_eq!(
            check_forward(
                &observation,
                &[&forecast],
                east_to(tangent - 1),
                Tick(1),
                None,
                &mut short_work,
            ),
            Ok(())
        );
        assert_eq!(short_work.spent(), 3);
        assert_eq!(
            check_forward(
                &observation,
                &[&forecast],
                east_to(tangent),
                Tick(1),
                None,
                &mut WorkBudget::new(2),
            ),
            Err(RouteFailure::WorkExhausted)
        );
    }

    #[test]
    fn archived_manual_policy_keeps_the_tangent_available() {
        let (observation, forecast) = scene(PunchActivationPolicy::ManualV1);
        let tangent = 3 * UNITS_PER_CELL - HALF_FOOTPRINT;
        assert_eq!(
            check_forward(
                &observation,
                &[&forecast],
                east_to(tangent),
                Tick(1),
                None,
                &mut WorkBudget::new(0),
            ),
            Ok(())
        );
    }

    #[test]
    fn contact_policy_does_not_close_a_wall_boundary_without_a_bomb() {
        let (observation, _) = scene(PunchActivationPolicy::ContactV2);
        let mut parts = observation.into_parts();
        parts.bombs = BombKnowledge::Visible(Vec::new());
        let wall = CellId::new(3, 1);
        let index = parts.arena.index(wall).unwrap();
        parts.arena.terrain[index] = crate::game::Terrain::Wall;
        let observation = Observation::new(parts).unwrap();
        let forecast = crate::cpu::routing::tests::forecast_for(&observation);
        let tangent = 3 * UNITS_PER_CELL - HALF_FOOTPRINT;
        assert_eq!(
            check_forward(
                &observation,
                &[&forecast],
                east_to(tangent),
                Tick(1),
                None,
                &mut WorkBudget::new(64),
            ),
            Ok(())
        );
    }

    #[test]
    fn contact_flame_expiry_index_has_an_exact_forecast_work_boundary() {
        let (observation, _) = scene(PunchActivationPolicy::ContactV2);
        let observation = observation
            .map_parts(|parts| {
                parts.flames.push(ObservedFlame {
                    cell: CellId::new(3, 1),
                    remaining_ticks: 1,
                });
            })
            .unwrap();
        let run = |observation: &Observation, allowance| {
            let mut work = WorkBudget::new(allowance);
            let result = forecast(
                observation,
                &ObservedHazardMemory::new(crate::cpu::routing::tests::key()),
                None,
                ForecastLimits::default(),
                &mut work,
            );
            (result, work.spent())
        };
        let (ample, exact) = run(&observation, 65_536);
        assert!(ample.is_ok());
        assert!(run(&observation, exact).0.is_ok());
        assert_eq!(
            run(&observation, exact - 1).0,
            Err(ForecastFailure::WorkExhausted)
        );

        let manual = observation
            .map_parts(|parts| {
                parts.rules.punch_activation_policy = PunchActivationPolicy::ManualV1;
            })
            .unwrap();
        let (manual_result, manual_work) = run(&manual, 65_536);
        assert!(manual_result.is_ok());
        assert_eq!(exact, manual_work + 2);
    }
}

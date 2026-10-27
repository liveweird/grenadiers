use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::policy::{ControlPlan, PolicyError, UtilityMemory};
use crate::cpu::routing::{
    PlannedBomb, PlantCertificate, PlantRequest, PulseLease, PulseRequest, RouteCertificate,
    RouteLimits, RouteMode, RouteRequest, RouteStep, certify_pulse, plan_ordinary_plant,
    plan_route,
};
use crate::cpu::{
    BombKnowledge, CpuRng, Intent, ManualIntent, MovementIntent, Observation, ObservedActor,
    WorkBudget,
};
use crate::game::{CellId, Direction, Terrain, Tick};

use super::MAX_PLAN_AGE_TICKS;
use super::compatibility::actor;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurvivalPlan {
    pub control: ControlPlan,
    pub lease: Option<PulseLease>,
    pub execution: Option<ExecutionCertificate>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionCertificate {
    Route(Box<RouteCertificate>),
    Plant(Box<PlantCertificate>),
}

impl ExecutionCertificate {
    #[must_use]
    pub fn route(&self) -> &RouteCertificate {
        match self {
            Self::Route(route) => route,
            Self::Plant(plant) => plant.route(),
        }
    }

    #[must_use]
    pub const fn plant(&self) -> Option<&PlantCertificate> {
        match self {
            Self::Route(_) => None,
            Self::Plant(plant) => Some(plant),
        }
    }
}

/// Produces one action; pulse leases never authorize manual input.
pub fn plan(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<SurvivalPlan, PolicyError> {
    let parts = observation.parts();
    if memory.key().world != parts.world
        || memory.key().cpu_seat != parts.recipient
        || history.key() != memory.key()
        || rng.key() != memory.key()
    {
        return Err(PolicyError::IdentityMismatch);
    }
    let stop = || SurvivalPlan {
        control: ControlPlan::stop(UtilityMemory::new(memory.key())),
        lease: None,
        execution: None,
    };
    if settled_stop_tick > parts.tick {
        return Ok(stop());
    }
    let Some(actor) = actor(observation) else {
        return Err(PolicyError::InvalidObservation);
    };
    if !actor.alive || actor.voluntary_actions_frozen {
        return Ok(stop());
    }
    let ordinary = crate::cpu::policy::plan(observation, memory, rng, work)?;
    let clear_movement = matches!(&parts.bombs, BombKnowledge::Visible(bombs) if bombs.is_empty())
        && parts.flames.is_empty()
        && parts.oil.is_empty()
        && ordinary.intent.movement != MovementIntent::Stop;
    if (ordinary.intent.manual.is_some() || clear_movement)
        && let Some(plan) = certified_plant(
            observation,
            history,
            settled_stop_tick,
            memory,
            &ordinary,
            work,
        )
    {
        return Ok(plan);
    }
    if ordinary.intent.manual.is_some() {
        return Ok(SurvivalPlan {
            control: ordinary,
            lease: None,
            execution: None,
        });
    }
    // Preserve the existing clear-board goals and pickup collection. Hazard
    // leases deliberately withhold item-changing paths until item tactics are
    // modeled; they must not replace the already legal clear-board behavior.
    if clear_movement {
        return Ok(SurvivalPlan {
            control: ordinary,
            lease: None,
            execution: None,
        });
    }
    let Ok(forecast) = forecast(observation, history, None, ForecastLimits::default(), work) else {
        return Ok(stop());
    };
    let Some(end) = parts.tick.0.checked_add(MAX_PLAN_AGE_TICKS + 2) else {
        return Ok(stop());
    };
    let start_window = TickWindow {
        start: Tick(parts.tick.0 + 1),
        end: Tick(end),
    };
    let Some(target) = super::steering::target(observation, &forecast, actor.position, work) else {
        return Ok(stop());
    };
    let mut choices = Vec::with_capacity(5);
    choices.push(ordinary.intent.movement);
    for direction in [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ] {
        let movement = MovementIntent::Direction(direction);
        if !choices.contains(&movement) {
            choices.push(movement);
        }
    }
    if !choices.contains(&MovementIntent::Stop) {
        choices.push(MovementIntent::Stop);
    }
    let mut fallback = None;
    let mut route_fallback = None;
    for mode in [
        RouteMode::CertifiedOnly,
        RouteMode::AllowStationaryAssumptions,
    ] {
        let mut best: Option<(u32, SurvivalPlan)> = None;
        for movement in &choices {
            let hold_ticks = match (movement, mode) {
                (MovementIntent::Stop, _) => 1,
                (_, RouteMode::AllowStationaryAssumptions) => 3,
                _ if *movement == ordinary.intent.movement => ordinary.hold_ticks.clamp(1, 12),
                _ => 12,
            };
            let Some(until) = end.checked_add(u64::from(hold_ticks) + 9) else {
                return Ok(stop());
            };
            let request = PulseRequest {
                movement: *movement,
                hold_ticks,
                start_window,
                stopped_safe_until: Tick(until),
                mode,
                settled_stop_tick,
            };
            if let Ok(lease) = certify_pulse(observation, &forecast, request, work) {
                let score = super::steering::score(&lease, target);
                let candidate = SurvivalPlan {
                    control: ControlPlan {
                        intent: Intent {
                            movement: *movement,
                            manual: None,
                        },
                        hold_ticks,
                        next_memory: if *movement == ordinary.intent.movement {
                            ordinary.next_memory
                        } else {
                            UtilityMemory::new(memory.key())
                        },
                    },
                    lease: Some(lease),
                    execution: None,
                };
                if best.as_ref().is_none_or(|(previous, _)| score < *previous) {
                    best = Some((score, candidate));
                }
            }
        }
        if let Some((_, candidate)) = best {
            fallback = Some(candidate);
            break;
        }
    }
    for mode in [
        RouteMode::CertifiedOnly,
        RouteMode::AllowStationaryAssumptions,
    ] {
        let request = RouteRequest {
            start_window,
            settled_stop_tick,
            survive_until: forecast.horizon_end(),
            mode,
            limits: RouteLimits::default(),
        };
        let Ok(route) = plan_route(observation, &forecast, request, work) else {
            continue;
        };
        let (movement, hold_ticks) = match route.steps().first() {
            Some(RouteStep::Move { direction, updates }) => {
                (MovementIntent::Direction(*direction), *updates)
            }
            Some(RouteStep::Wait { .. }) => (MovementIntent::Stop, 1),
            None => continue,
        };
        route_fallback = Some(SurvivalPlan {
            control: ControlPlan {
                intent: Intent {
                    movement,
                    manual: None,
                },
                hold_ticks,
                next_memory: UtilityMemory::new(memory.key()),
            },
            lease: None,
            execution: Some(ExecutionCertificate::Route(Box::new(route))),
        });
        break;
    }
    if let Some(plan) = certified_plant(
        observation,
        history,
        settled_stop_tick,
        memory,
        &ordinary,
        work,
    ) {
        return Ok(plan);
    }
    Ok(route_fallback.or(fallback).unwrap_or_else(stop))
}

fn certified_plant(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    ordinary: &ControlPlan,
    work: &mut WorkBudget,
) -> Option<SurvivalPlan> {
    if ordinary.intent.manual == Some(ManualIntent::Equipment) {
        return None;
    }
    if !could_be_useful(observation, work) {
        return None;
    }
    let execution_tick = observation.parts().tick.0.checked_add(7).map(Tick)?;
    let certificate = plan_ordinary_plant(
        observation,
        history,
        PlantRequest {
            execution_tick,
            settled_stop_tick,
            first_movement: ordinary.intent.movement,
            limits: RouteLimits::default(),
        },
        work,
    )
    .ok()?;
    useful_plant(observation, certificate.placement()).then(|| SurvivalPlan {
        control: ControlPlan {
            intent: Intent {
                movement: ordinary.intent.movement,
                manual: Some(ManualIntent::Bomb),
            },
            hold_ticks: 1,
            next_memory: UtilityMemory::new(memory.key()),
        },
        lease: None,
        execution: Some(ExecutionCertificate::Plant(Box::new(certificate))),
    })
}

fn could_be_useful(observation: &Observation, work: &mut WorkBudget) -> bool {
    let Some(actor) = actor(observation) else {
        return false;
    };
    let Some(origin) = actor.position.cell() else {
        return false;
    };
    let parts = observation.parts();
    std::iter::once(Some(origin))
        .chain(adjacent_cells(origin))
        .flatten()
        .any(|candidate| {
            if work.try_take(1).is_err() {
                return false;
            }
            adjacent_cells(candidate)
                .into_iter()
                .flatten()
                .any(|cell| parts.arena.blocks.contains(&cell))
                || parts.actors.iter().any(|observed| {
                    let ObservedActor::Visible(rival) = observed else {
                        return false;
                    };
                    rival.id != actor.id
                        && rival.alive
                        && rival.position.cell().is_some_and(|target| {
                            (candidate.column == target.column || candidate.row == target.row)
                                && candidate.column.abs_diff(target.column)
                                    + candidate.row.abs_diff(target.row)
                                    <= 8
                        })
                })
        })
}

fn useful_plant(observation: &Observation, bomb: &PlannedBomb) -> bool {
    let parts = observation.parts();
    if adjacent_cells(bomb.cell)
        .into_iter()
        .flatten()
        .any(|cell| parts.arena.blocks.contains(&cell))
    {
        return true;
    }
    parts.actors.iter().any(|actor| {
        let ObservedActor::Visible(actor) = actor else {
            return false;
        };
        actor.id != bomb.owner
            && actor.alive
            && actor.position.cell().is_some_and(|target| {
                blast_reaches(
                    bomb.cell,
                    target,
                    bomb.range,
                    parts.arena.width,
                    &parts.arena.terrain,
                    &parts.arena.blocks,
                )
            })
    })
}

fn adjacent_cells(cell: CellId) -> [Option<CellId>; 4] {
    [
        cell.row
            .checked_sub(1)
            .map(|row| CellId::new(cell.column, row)),
        cell.column
            .checked_add(1)
            .map(|column| CellId::new(column, cell.row)),
        cell.row
            .checked_add(1)
            .map(|row| CellId::new(cell.column, row)),
        cell.column
            .checked_sub(1)
            .map(|column| CellId::new(column, cell.row)),
    ]
}

fn blast_reaches(
    origin: CellId,
    target: CellId,
    range: u16,
    width: u16,
    terrain: &[Terrain],
    blocks: &[CellId],
) -> bool {
    let (dc, dr, distance) = if origin.column == target.column {
        (
            0_i16,
            if target.row > origin.row { 1 } else { -1 },
            origin.row.abs_diff(target.row),
        )
    } else if origin.row == target.row {
        (
            if target.column > origin.column { 1 } else { -1 },
            0_i16,
            origin.column.abs_diff(target.column),
        )
    } else {
        return false;
    };
    if distance == 0 || distance > range {
        return false;
    }
    (1..=distance).all(|step| {
        let column = i32::from(origin.column) + i32::from(dc) * i32::from(step);
        let row = i32::from(origin.row) + i32::from(dr) * i32::from(step);
        let (Ok(column), Ok(row)) = (u16::try_from(column), u16::try_from(row)) else {
            return false;
        };
        let cell = CellId::new(column, row);
        let index = usize::from(row) * usize::from(width) + usize::from(column);
        terrain.get(index) == Some(&Terrain::Floor) && !blocks.contains(&cell)
    })
}

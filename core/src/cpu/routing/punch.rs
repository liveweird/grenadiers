use crate::cpu::forecast::{
    ForecastLimits, ObservedHazardMemory, TickWindow, forecast, forecast_punch_branch,
};
use crate::cpu::{BombKnowledge, Observation, ObservedActor, ObservedBomb, WorkBudget};
use crate::game::types::{HALF_FOOTPRINT, TICKS_PER_SECOND, UNITS_PER_CELL};
use crate::game::{BlastKind, CellId, Equipment, Position, SpeedLevel, Terrain, Tick};

use super::punch_types::BombSignature;
use super::route::plan_route_seeded;
use super::safety::RecipientBarrier;
use super::{
    PunchCertificate, PunchFailure, PunchOutcome, PunchRequest, RouteMode, RouteRequest, RouteStep,
};

pub fn plan_punch(
    observation: &Observation,
    history: &ObservedHazardMemory,
    request: PunchRequest,
    work: &mut WorkBudget,
) -> Result<PunchCertificate, PunchFailure> {
    let parts = observation.parts();
    if request.execution_tick
        != Tick(
            parts
                .tick
                .0
                .checked_add(7)
                .ok_or(PunchFailure::InvalidRequest)?,
        )
        || request.settled_stop_tick > parts.tick
    {
        return Err(PunchFailure::InvalidRequest);
    }
    let (actor, _rival, actor_cell, bomb, signature) = planning_facts(observation)?;
    let landing = select_landing(observation, bomb, actor.position, work)?;
    if !all_rivals_support_landing(observation, landing, 7) {
        return Err(PunchFailure::Unsupported);
    }
    let unchanged = forecast(observation, history, None, ForecastLimits::default(), work)?;
    let relocated = forecast_punch_branch(
        observation,
        history,
        bomb.id,
        landing,
        request.execution_tick,
        ForecastLimits::default(),
        work,
    )?;
    let rules = crate::cpu::rules_config(observation).ok_or(PunchFailure::Unsupported)?;
    let survive_until = Tick(
        signature
            .detonates_at
            .0
            .checked_add(rules.flame_ticks())
            .ok_or(PunchFailure::InvalidRequest)?,
    );
    let route = plan_route_seeded(
        observation,
        &[&unchanged, relocated.forecast()],
        RouteRequest {
            start_window: TickWindow {
                start: request.execution_tick,
                end: Tick(
                    request
                        .execution_tick
                        .0
                        .checked_add(1)
                        .ok_or(PunchFailure::InvalidRequest)?,
                ),
            },
            settled_stop_tick: request.settled_stop_tick,
            survive_until,
            mode: RouteMode::CertifiedOnly,
            limits: request.limits,
        },
        Some(RouteStep::Wait { updates: 4 }),
        None,
        Some(RecipientBarrier {
            cell: actor_cell,
            bomb_id: bomb.id,
        }),
        work,
    )?;
    Ok(PunchCertificate::new(
        route,
        request.execution_tick,
        signature,
        landing,
        relocated.hypothetical_id(),
        unchanged.evolution().clone(),
        relocated.forecast().evolution().clone(),
        relocated.scenario_basis().clone(),
    ))
}

pub(super) fn can_admit(
    certificate: &PunchCertificate,
    basis: &Observation,
    current: &Observation,
    admission_tick: Tick,
) -> bool {
    let Some(expected_admission) = certificate.execution_tick().0.checked_sub(1).map(Tick) else {
        return false;
    };
    if admission_tick != expected_admission
        || current.parts().tick != admission_tick
        || certificate.confirm(basis, current) != Ok(PunchOutcome::Awaiting)
    {
        return false;
    }
    let (Some(before), Some(after), Some(before_rival), Some(rival)) = (
        recipient(basis),
        recipient(current),
        visible_rival(basis),
        visible_rival(current),
    ) else {
        return false;
    };
    if before.id != after.id
        || before.position != after.position
        || before.heading != after.heading
        || before.speed != after.speed
        || after.equipment != Some(Equipment::Punch)
        || !after.alive
        || after.voluntary_actions_frozen
        || before_rival.id != rival.id
        || before_rival.seat != rival.seat
        || (current.parts().rules.version == 6 && !rivals_unchanged(basis, current))
        || !rival_supported(rival)
        || !current.parts().pickups.is_empty()
        || !current.parts().oil.is_empty()
        || !current.parts().flames.is_empty()
    {
        return false;
    }
    let BombKnowledge::Visible(bombs) = &current.parts().bombs else {
        return false;
    };
    let [bomb] = bombs.as_slice() else {
        return false;
    };
    let Ok(landing) = select_landing_unmetered(current, bomb, after.position) else {
        return false;
    };
    landing == certificate.landing()
        && certificate.bomb().id == bomb.id
        && all_rivals_support_landing(current, landing, 1)
}

fn planning_facts(
    observation: &Observation,
) -> Result<
    (
        &crate::cpu::VisibleActor,
        &crate::cpu::VisibleActor,
        CellId,
        &ObservedBomb,
        BombSignature,
    ),
    PunchFailure,
> {
    let parts = observation.parts();
    let actor = recipient(observation).ok_or(PunchFailure::InvalidRequest)?;
    let rival = visible_rival(observation).ok_or(PunchFailure::Unsupported)?;
    let actor_cell = centered_cell(actor.position).ok_or(PunchFailure::Unsupported)?;
    if !matches!(parts.rules.version, 5 | 6)
        || !parts.rules.settings.enabled_items().contains_punch()
        || actor.equipment != Some(Equipment::Punch)
        || !actor.alive
        || actor.voluntary_actions_frozen
        || !rival_supported(rival)
        || !parts.pickups.is_empty()
        || !parts.oil.is_empty()
        || !parts.flames.is_empty()
        || !barrier_cell_open(observation, actor_cell)
    {
        return Err(PunchFailure::Unsupported);
    }
    let BombKnowledge::Visible(bombs) = &parts.bombs else {
        return Err(PunchFailure::Unsupported);
    };
    let [bomb] = bombs.as_slice() else {
        return Err(PunchFailure::Unsupported);
    };
    if crate::game::punch_adjacent_cell(actor_cell, actor.heading) != Some(bomb.cell)
        || bomb.blast != BlastKind::Standard
    {
        return Err(PunchFailure::Unsupported);
    }
    let crate::cpu::ObservedBombTrigger::Timed {
        remaining_ticks,
        paused: false,
    } = bomb.trigger
    else {
        return Err(PunchFailure::Unsupported);
    };
    let detonates_at = Tick(
        parts
            .tick
            .0
            .checked_add(u64::from(remaining_ticks))
            .ok_or(PunchFailure::InvalidRequest)?,
    );
    if detonates_at <= Tick(parts.tick.0 + 7) {
        return Err(PunchFailure::Unsupported);
    }
    Ok((
        actor,
        rival,
        actor_cell,
        bomb,
        BombSignature {
            id: bomb.id,
            placed_at: bomb.placed_at,
            source: bomb.cell,
            owner: bomb.owner,
            range: bomb.range,
            detonates_at,
        },
    ))
}

pub(super) fn select_landing(
    observation: &Observation,
    bomb: &ObservedBomb,
    actor: Position,
    work: &mut WorkBudget,
) -> Result<CellId, PunchFailure> {
    work.try_take(
        u32::try_from(observation.parts().arena.cell_count())
            .map_err(|_| PunchFailure::Route(super::RouteFailure::Capacity))?,
    )
    .map_err(|_| PunchFailure::Route(super::RouteFailure::WorkExhausted))?;
    select_landing_unmetered(observation, bomb, actor)
}

pub(super) fn select_landing_unmetered(
    observation: &Observation,
    bomb: &ObservedBomb,
    actor: Position,
) -> Result<CellId, PunchFailure> {
    let direction = recipient(observation)
        .map(|actor| actor.heading)
        .ok_or(PunchFailure::InvalidRequest)?;
    let board = observation
        .parts()
        .arena
        .board()
        .ok_or(PunchFailure::InvalidRequest)?;
    crate::game::punch_landing_candidates_for_board(bomb.cell, direction, board)
        .find(|candidate| landing_open(observation, bomb.id, actor, *candidate))
        .ok_or(PunchFailure::Unsupported)
}

fn landing_open(
    observation: &Observation,
    source: crate::game::EntityId,
    actor: Position,
    cell: CellId,
) -> bool {
    let parts = observation.parts();
    let Some(index) = parts.arena.index(cell) else {
        return false;
    };
    crate::game::punch_landing_is_open(
        parts.arena.terrain.get(index).copied(),
        parts.arena.blocks.contains(&cell),
        parts
            .oil
            .iter()
            .any(|oil| oil.cell == cell && oil.state == crate::game::OilBarrelState::Unlit),
        matches!(&parts.bombs, BombKnowledge::Visible(bombs)
            if bombs.iter().any(|bomb| bomb.id != source && bomb.cell == cell)),
        crate::game::geometry::position_overlaps_cell(actor, cell),
    )
}

fn barrier_cell_open(observation: &Observation, cell: CellId) -> bool {
    let parts = observation.parts();
    let Some(index) = parts.arena.index(cell) else {
        return false;
    };
    parts.arena.terrain.get(index) == Some(&Terrain::Floor)
        && !parts.arena.blocks.contains(&cell)
        && !parts.oil.iter().any(|oil| oil.cell == cell)
        && matches!(&parts.bombs, BombKnowledge::Visible(bombs)
            if bombs.iter().all(|bomb| bomb.cell != cell))
}

fn centered_cell(position: Position) -> Option<CellId> {
    let cell = position.cell()?;
    (position == Position::center(cell)).then_some(cell)
}

fn recipient(observation: &Observation) -> Option<&crate::cpu::VisibleActor> {
    observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == observation.parts().recipient => {
                Some(actor)
            }
            _ => None,
        })
}

fn visible_rival(observation: &Observation) -> Option<&crate::cpu::VisibleActor> {
    let parts = observation.parts();
    if parts.actors.iter().any(|actor| {
        matches!(actor,
        ObservedActor::Hidden { seat, .. } if *seat != parts.recipient)
    }) {
        return None;
    }
    parts.actors.iter().find_map(|actor| match actor {
        ObservedActor::Visible(actor) if actor.seat != parts.recipient => Some(actor),
        _ => None,
    })
}

fn all_rivals_support_landing(observation: &Observation, landing: CellId, updates: u64) -> bool {
    let parts = observation.parts();
    parts.actors.iter().all(|actor| match actor {
        ObservedActor::Visible(actor) if actor.seat != parts.recipient => {
            rival_supported(actor)
                && !reachable_footprint_intersects(actor.position, landing, updates)
        }
        ObservedActor::Hidden { seat, .. } if *seat != parts.recipient => false,
        _ => true,
    })
}

fn rivals_unchanged(basis: &Observation, current: &Observation) -> bool {
    let old = basis.parts();
    let new = current.parts();
    old.actors.iter().all(|before| {
        if before.seat() == old.recipient {
            return true;
        }
        new.actors.iter().any(|after| match (before, after) {
            (ObservedActor::Visible(before), ObservedActor::Visible(after)) => {
                before.id == after.id
                    && before.seat == after.seat
                    && before.position == after.position
                    && before.alive == after.alive
                    && before.equipment == after.equipment
            }
            _ => false,
        })
    })
}

fn rival_supported(rival: &crate::cpu::VisibleActor) -> bool {
    rival.alive
        && !matches!(
            rival.equipment,
            Some(Equipment::Kick | Equipment::Punch | Equipment::Bracelet)
        )
}

fn reachable_footprint_intersects(position: Position, cell: CellId, updates: u64) -> bool {
    let travel = u64::from(SpeedLevel::Four.units_per_second())
        .saturating_mul(updates)
        .div_ceil(u64::from(TICKS_PER_SECOND));
    let expansion = i64::from(HALF_FOOTPRINT) + i64::try_from(travel).unwrap_or(i64::MAX);
    let left = i64::from(cell.column) * i64::from(UNITS_PER_CELL);
    let top = i64::from(cell.row) * i64::from(UNITS_PER_CELL);
    let right = left + i64::from(UNITS_PER_CELL);
    let bottom = top + i64::from(UNITS_PER_CELL);
    i64::from(position.x) > left - expansion
        && i64::from(position.x) < right + expansion
        && i64::from(position.y) > top - expansion
        && i64::from(position.y) < bottom + expansion
}

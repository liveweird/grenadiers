use crate::cpu::forecast::{HazardForecast, TickWindow};
use crate::cpu::{BombKnowledge, MovementIntent, Observation, ObservedActor, WorkBudget};
use crate::game::{Equipment, PickupKind, SpecialWeapon, Tick};

use super::motion::{Motion, advance};
use super::teleporter_step_proof::{ProofInput, prove};
use super::teleporter_step_types::{TeleporterStep, TeleporterStepPhase, TeleporterStepRequest};
use super::{RouteFailure, validation};

const START_DELAY_END: u64 = 8;
const EXCLUSIVE_HORIZON: u64 = 18;

pub fn certify_teleporter_step(
    observation: &Observation,
    forecast: &HazardForecast,
    request: TeleporterStepRequest,
    work: &mut WorkBudget,
) -> Result<TeleporterStep, RouteFailure> {
    certify(observation, forecast, request, false, work)
}

pub fn certify_teleporter_step_v50(
    observation: &Observation,
    forecast: &HazardForecast,
    request: TeleporterStepRequest,
    work: &mut WorkBudget,
) -> Result<TeleporterStep, RouteFailure> {
    certify(observation, forecast, request, true, work)
}

fn certify(
    observation: &Observation,
    forecast: &HazardForecast,
    request: TeleporterStepRequest,
    allow_distant_teleporters: bool,
    work: &mut WorkBudget,
) -> Result<TeleporterStep, RouteFailure> {
    let basis = observation.parts().tick;
    let start_window = TickWindow {
        start: checked_tick(basis, 1)?,
        end: checked_tick(basis, START_DELAY_END)?,
    };
    let safe_until = checked_tick(basis, EXCLUSIVE_HORIZON)?;
    charge_validation_scan(observation, work)?;
    let context = validation::validate(observation, forecast, start_window, safe_until)?;
    validate_scene(
        observation,
        request,
        safe_until,
        allow_distant_teleporters,
        work,
    )?;
    // Nominal phase is based on the unobstructed root update. Arbitrary rival
    // motion and newly planted blocking bombs are represented by every partial
    // endpoint of these segments in the proof below.
    charge_observation_clone(observation, work)?;
    let motion_observation = observation
        .clone()
        .map_parts(|parts| {
            for actor in &mut parts.actors {
                if actor.seat() != parts.recipient
                    && let ObservedActor::Visible(actor) = actor
                {
                    actor.alive = false;
                }
            }
        })
        .map_err(|_| RouteFailure::InvalidRequest)?;
    charge_validation_scan(&motion_observation, work)?;
    let motion_context =
        validation::validate(&motion_observation, forecast, start_window, safe_until)?;
    take(work, 7)?;
    let mut nominal = Vec::with_capacity(7);
    for start in start_window.start.0..start_window.end.0 {
        take(work, 1)?;
        let mut motion = Motion::stopped(&motion_context, Tick(start));
        for tick in basis.0 + 1..start {
            super::storm::charge_tick(context.storm, work)?;
            charge_stop_advance(work)?;
            advance(
                &motion_context,
                &mut motion,
                MovementIntent::Stop,
                Tick(tick),
            )?;
        }
        super::storm::charge_tick(context.storm, work)?;
        charge_directional_advance(&motion_observation, work)?;
        let segments = advance(&motion_context, &mut motion, request.movement, Tick(start))?;
        super::contact_punch_guard::check_motion(
            observation,
            &[forecast],
            &motion,
            Tick(start),
            None,
            work,
        )?;
        nominal.push((Tick(start), motion, segments));
    }
    take(work, usize_to_work(nominal.len())?)?;
    let all_enter = nominal
        .iter()
        .all(|(_, motion, _)| motion.position.cell() == Some(request.target));
    let none_enter = nominal
        .iter()
        .all(|(_, motion, _)| motion.position.cell() != Some(request.target));
    let phase = if all_enter {
        if context.actor.position.cell() == Some(request.target) {
            return Err(RouteFailure::InvalidRequest);
        }
        TeleporterStepPhase::Collect
    } else if none_enter {
        TeleporterStepPhase::Approach
    } else {
        return Err(RouteFailure::NoRoute);
    };
    let outcome = prove(ProofInput {
        context: &context,
        forecast,
        request,
        phase,
        safe_until,
        nominal: &nominal,
        allow_distant_teleporters,
        work,
    })?;
    charge_observation_clone(observation, work)?;
    Ok(TeleporterStep {
        key: context.key,
        basis_tick: basis,
        start_window,
        safe_until,
        settled_stop_tick: request.settled_stop_tick,
        target: request.target,
        movement: request.movement,
        phase,
        initial_position: context.actor.position,
        nominal_positions: outcome.nominal_positions,
        collecting_positions: outcome.collecting_positions,
        clipped_positions: outcome.clipped_positions,
        landing_regions: outcome.landing_regions,
        basis: observation.clone(),
    })
}

fn validate_scene(
    observation: &Observation,
    request: TeleporterStepRequest,
    safe_until: Tick,
    allow_distant_teleporters: bool,
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    let parts = observation.parts();
    take(work, 1)?;
    if parts.rules.version != 10
        || request.settled_stop_tick > parts.tick
        || !matches!(request.movement, MovementIntent::Direction(_))
        || !parts.rules.settings.enabled_items().contains_teleporter()
        || !matches!(&parts.bombs, BombKnowledge::Visible(bombs) if bombs.is_empty())
        || !parts.flames.is_empty()
        || !parts.oil.is_empty()
        || crate::cpu::rules_config(observation)
            .is_none_or(|rules| rules.bomb_fuse_ticks() <= safe_until.0 - parts.tick.0)
    {
        return Err(RouteFailure::Unsupported);
    }
    take(work, usize_to_work(parts.pickups.len())?)?;
    let teleporters = parts
        .pickups
        .iter()
        .filter(|pickup| pickup.kind == PickupKind::Teleporter)
        .map(|pickup| pickup.cell);
    if if allow_distant_teleporters {
        teleporters.filter(|cell| *cell == request.target).count() != 1
    } else {
        teleporters.collect::<Vec<_>>() != [request.target]
    } {
        return Err(RouteFailure::InvalidRequest);
    }
    take(work, usize_to_work(parts.actors.len())?)?;
    if parts.actors.iter().any(|actor| match actor {
        ObservedActor::Hidden { alive: true, .. } => true,
        ObservedActor::Hidden { alive: false, .. } => false,
        ObservedActor::Visible(actor) => {
            actor.alive
                && (actor.voluntary_actions_frozen
                    || !actor.effects.is_empty()
                    || actor.equipment == Some(Equipment::Bracelet)
                    || matches!(
                        actor.weapon,
                        Some(SpecialWeapon::Remote | SpecialWeapon::Landmine)
                    ))
        }
    }) {
        return Err(RouteFailure::Unsupported);
    }
    Ok(())
}

pub(super) fn take(work: &mut WorkBudget, units: u32) -> Result<(), RouteFailure> {
    work.try_take(units)
        .map_err(|_| RouteFailure::WorkExhausted)
}

pub(super) fn usize_to_work(value: usize) -> Result<u32, RouteFailure> {
    u32::try_from(value).map_err(|_| RouteFailure::Capacity)
}

fn checked_tick(basis: Tick, offset: u64) -> Result<Tick, RouteFailure> {
    basis
        .0
        .checked_add(offset)
        .map(Tick)
        .ok_or(RouteFailure::InvalidRequest)
}

fn charge_validation_scan(
    observation: &Observation,
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    let parts = observation.parts();
    let scans = parts
        .arena
        .cell_count()
        .checked_add(parts.actors.len())
        .and_then(|value| value.checked_add(parts.oil.len()))
        .ok_or(RouteFailure::Capacity)?;
    take(work, usize_to_work(scans)?)
}

fn charge_observation_clone(
    observation: &Observation,
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    let parts = observation.parts();
    let mut scans = parts
        .arena
        .cell_count()
        .checked_add(parts.arena.blocks.len())
        .and_then(|value| value.checked_add(parts.actors.len()))
        .and_then(|value| value.checked_add(parts.pickups.len()))
        .and_then(|value| value.checked_add(parts.flames.len()))
        .and_then(|value| value.checked_add(parts.oil.len()))
        .ok_or(RouteFailure::Capacity)?;
    if let Some(lairs) = observation.respawn_lairs() {
        scans = scans
            .checked_add(lairs.lairs().len())
            .and_then(|value| value.checked_add(lairs.mobs().len()))
            .ok_or(RouteFailure::Capacity)?;
    }
    if let Some(pits) = observation.spike_pits() {
        scans = scans
            .checked_add(pits.pits().len())
            .ok_or(RouteFailure::Capacity)?;
    }
    take(work, usize_to_work(scans)?)
}

fn charge_stop_advance(work: &mut WorkBudget) -> Result<(), RouteFailure> {
    // `Stop` allocates one stationary segment and clears two scalar fields.
    take(work, 1)
}

pub(super) fn charge_directional_advance(
    observation: &Observation,
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    // SpeedLevel::Four plus an unknown residual moves at most 86 units/update,
    // less than one cell. Forgiving turn can create at most two axis legs. Each
    // leg enumerates at most four footprint cells, collects the full actor
    // roster once, and checks each resulting obstacle's footprint/reserved cell.
    // Sixteen covers both segment/static allocations and cell visits; eight per
    // actor covers both roster passes and both collision scans with slack.
    let units = observation
        .parts()
        .actors
        .len()
        .checked_mul(8)
        .and_then(|value| value.checked_add(16))
        .ok_or(RouteFailure::Capacity)?;
    take(work, usize_to_work(units)?)
}

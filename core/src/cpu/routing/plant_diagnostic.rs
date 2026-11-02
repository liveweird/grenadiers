use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, forecast};
use crate::cpu::{
    BombKnowledge, MovementIntent, Observation, ObservedActor, ObservedBombTrigger, WorkBudget,
};
use crate::game::{BlastKind, EffectKind, SpecialWeapon, Tick};

use super::motion::{Motion, advance};
use super::plant::{add_tick, derive_bomb_modifiers, effect_active, origin_is_free};
use super::{PlannedBomb, PlantFailure, PlantRequest};

/// Derives the authoritative bomb tuple before hypothetical forecasting and
/// route proof. This grants no planting authority.
pub(crate) fn derive_lair_plant_placement(
    observation: &Observation,
    memory: &ObservedHazardMemory,
    request: PlantRequest,
    work: &mut WorkBudget,
) -> Result<PlannedBomb, PlantFailure> {
    let parts = observation.parts();
    if !matches!(parts.rules.version, 8..=10) {
        return Err(PlantFailure::Unsupported);
    }
    let expected = parts
        .tick
        .0
        .checked_add(7)
        .map(Tick)
        .ok_or(PlantFailure::InvalidRequest)?;
    if request.execution_tick != expected || request.settled_stop_tick > parts.tick {
        return Err(PlantFailure::InvalidRequest);
    }
    let ObservedActor::Visible(actor) = parts
        .actors
        .iter()
        .find(|actor| actor.seat() == parts.recipient)
        .ok_or(PlantFailure::InvalidRequest)?
    else {
        return Err(PlantFailure::InvalidRequest);
    };
    let weapon = weapon_blast(observation, actor.weapon)?;
    if !actor.alive
        || actor.voluntary_actions_frozen
        || matches!(
            &parts.bombs,
            BombKnowledge::Visible(bombs)
                if bombs.iter().any(|bomb| {
                    bomb.owner == actor.id && bomb.trigger == ObservedBombTrigger::Remote
                })
        )
        || effect_active(
            parts.tick,
            request.execution_tick,
            actor,
            EffectKind::Diarrhea,
        )?
        || effect_active(
            parts.tick,
            request.execution_tick,
            actor,
            EffectKind::Stopwatch,
        )?
    {
        return Err(PlantFailure::Unsupported);
    }
    let absent = forecast(observation, memory, None, ForecastLimits::default(), work)?;
    if absent.evolution().expected_live_bombs(actor.id, parts.tick) != Some(actor.live_bomb_count)
        || absent
            .evolution()
            .expected_live_bombs(actor.id, request.execution_tick)
            .is_none_or(|count| count >= actor.bomb_capacity)
    {
        return Err(PlantFailure::Unsupported);
    }
    let context = super::validation::validate(
        observation,
        &absent,
        crate::cpu::forecast::TickWindow {
            start: request.execution_tick,
            end: add_tick(request.execution_tick, 1)?,
        },
        request.execution_tick,
    )?;
    let position = match request.first_movement {
        MovementIntent::Direction(_) => {
            let mut motion = Motion::stopped(&context, request.execution_tick);
            let _ = advance(
                &context,
                &mut motion,
                request.first_movement,
                request.execution_tick,
            )?;
            super::contact_punch_guard::check_motion(
                observation,
                &[&absent],
                &motion,
                request.execution_tick,
                None,
                work,
            )?;
            motion.position
        }
        MovementIntent::Stop => actor.position,
    };
    let cell = position.cell().ok_or(PlantFailure::Unsupported)?;
    if !origin_is_free(observation, cell) {
        return Err(PlantFailure::Unsupported);
    }
    let rules = crate::cpu::rules_config(observation).ok_or(PlantFailure::Unsupported)?;
    let (range, fuse) = derive_bomb_modifiers(actor, parts.tick, request.execution_tick, rules)?;
    Ok(PlannedBomb {
        placed_at: request.execution_tick,
        owner: actor.id,
        cell,
        range,
        blast: weapon.map_or(BlastKind::Standard, |(_, blast)| blast),
        detonates_at: add_tick(request.execution_tick, fuse)?,
    })
}

fn weapon_blast(
    observation: &Observation,
    weapon: Option<SpecialWeapon>,
) -> Result<Option<(SpecialWeapon, BlastKind)>, PlantFailure> {
    let Some(weapon) = weapon else {
        return Ok(None);
    };
    let enabled = observation.parts().rules.settings.enabled_items();
    match weapon {
        SpecialWeapon::Piercing if enabled.contains_piercing_bomb() => {
            Ok(Some((weapon, BlastKind::Piercing)))
        }
        SpecialWeapon::Power if enabled.contains_power_bomb() => {
            Ok(Some((weapon, BlastKind::Power)))
        }
        SpecialWeapon::Remote
        | SpecialWeapon::Landmine
        | SpecialWeapon::Piercing
        | SpecialWeapon::Power => Err(PlantFailure::Unsupported),
    }
}

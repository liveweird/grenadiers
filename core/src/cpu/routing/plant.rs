use crate::cpu::forecast::{
    ForecastLimits, HypotheticalPlacement, ObservedHazardMemory, TickWindow, forecast,
};
use crate::cpu::{
    BombKnowledge, MovementIntent, Observation, ObservedActor, ObservedBombTrigger, WorkBudget,
};
use crate::game::effects::{effective_bomb_fuse_ticks, effective_bomb_range};
use crate::game::{
    BlastKind, EffectKind, OilBarrelState, RulesConfig, SpecialWeapon, Terrain, Tick,
};

use super::motion::{Motion, advance};
use super::route::plan_route_seeded;
use super::route_hints::plan_route_to_refuge;
use super::validation;
use super::{
    PlannedBomb, PlantCertificate, PlantFailure, PlantRequest, RouteMode, RouteRequest, RouteStep,
};

pub fn plan_ordinary_plant(
    observation: &Observation,
    memory: &ObservedHazardMemory,
    request: PlantRequest,
    work: &mut WorkBudget,
) -> Result<PlantCertificate, PlantFailure> {
    if observation.parts().rules.version >= 8 {
        return Err(PlantFailure::Unsupported);
    }
    plan_timed_plant(observation, memory, request, work, None, None)
}

pub(crate) fn plan_lair_ordinary_plant(
    observation: &Observation,
    memory: &ObservedHazardMemory,
    request: PlantRequest,
    work: &mut WorkBudget,
) -> Result<PlantCertificate, PlantFailure> {
    validate_lair_observation(observation)?;
    plan_timed_plant(observation, memory, request, work, None, None)
}

pub(crate) fn plan_lair_ordinary_plant_to_refuge(
    observation: &Observation,
    memory: &ObservedHazardMemory,
    request: PlantRequest,
    refuge: crate::game::CellId,
    work: &mut WorkBudget,
) -> Result<PlantCertificate, PlantFailure> {
    validate_lair_observation(observation)?;
    plan_timed_plant(observation, memory, request, work, None, Some(refuge))
}

pub fn plan_timed_weapon_plant(
    observation: &Observation,
    memory: &ObservedHazardMemory,
    request: PlantRequest,
    work: &mut WorkBudget,
) -> Result<PlantCertificate, PlantFailure> {
    if observation.parts().rules.version >= 8 {
        return Err(PlantFailure::Unsupported);
    }
    plan_lair_or_legacy_weapon_plant(observation, memory, request, work)
}

pub(crate) fn plan_lair_timed_weapon_plant(
    observation: &Observation,
    memory: &ObservedHazardMemory,
    request: PlantRequest,
    work: &mut WorkBudget,
) -> Result<PlantCertificate, PlantFailure> {
    validate_lair_observation(observation)?;
    plan_lair_or_legacy_weapon_plant(observation, memory, request, work)
}

fn plan_lair_or_legacy_weapon_plant(
    observation: &Observation,
    memory: &ObservedHazardMemory,
    request: PlantRequest,
    work: &mut WorkBudget,
) -> Result<PlantCertificate, PlantFailure> {
    let weapon = actor_weapon(observation).ok_or(PlantFailure::Unsupported)?;
    let enabled = observation.parts().rules.settings.enabled_items();
    let blast = match weapon {
        SpecialWeapon::Piercing if enabled.contains_piercing_bomb() => BlastKind::Piercing,
        SpecialWeapon::Power if enabled.contains_power_bomb() => BlastKind::Power,
        SpecialWeapon::Remote
        | SpecialWeapon::Landmine
        | SpecialWeapon::Piercing
        | SpecialWeapon::Power => return Err(PlantFailure::Unsupported),
    };
    plan_timed_plant(
        observation,
        memory,
        request,
        work,
        Some((weapon, blast)),
        None,
    )
}

fn validate_lair_observation(observation: &Observation) -> Result<(), PlantFailure> {
    ((observation.parts().rules.version == 8 && observation.lairs().is_some())
        || matches!(observation.parts().rules.version, 9 | 10))
    .then_some(())
    .ok_or(PlantFailure::Unsupported)
}

fn plan_timed_plant(
    observation: &Observation,
    memory: &ObservedHazardMemory,
    request: PlantRequest,
    work: &mut WorkBudget,
    weapon: Option<(SpecialWeapon, BlastKind)>,
    refuge: Option<crate::game::CellId>,
) -> Result<PlantCertificate, PlantFailure> {
    let parts = observation.parts();
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
    if !actor.alive
        || actor.voluntary_actions_frozen
        || actor.weapon != weapon.map(|(weapon, _)| weapon)
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
    let start_window = TickWindow {
        start: request.execution_tick,
        end: add_tick(request.execution_tick, 1)?,
    };
    let context = validation::validate(observation, &absent, start_window, request.execution_tick)?;
    let (position, first_step) = match request.first_movement {
        MovementIntent::Direction(direction) => {
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
            (
                motion.position,
                RouteStep::Move {
                    direction,
                    updates: 1,
                },
            )
        }
        MovementIntent::Stop => (actor.position, RouteStep::Wait { updates: 4 }),
    };
    let cell = position.cell().ok_or(PlantFailure::Unsupported)?;
    if !origin_is_free(observation, cell) {
        return Err(PlantFailure::Unsupported);
    }

    let rules = crate::cpu::rules_config(observation).ok_or(PlantFailure::Unsupported)?;
    let (range, fuse) = derive_bomb_modifiers(actor, parts.tick, request.execution_tick, rules)?;
    let detonates_at = add_tick(request.execution_tick, fuse)?;
    let survive_until = add_tick(detonates_at, rules.flame_ticks())?;
    let placement = PlannedBomb {
        placed_at: request.execution_tick,
        owner: actor.id,
        cell,
        range,
        blast: weapon.map_or(BlastKind::Standard, |(_, blast)| blast),
        detonates_at,
    };
    let present = forecast(
        observation,
        memory,
        Some(HypotheticalPlacement {
            placed_at: placement.placed_at,
            owner: placement.owner,
            cell: placement.cell,
            range: placement.range,
            blast: placement.blast,
            detonates_at: placement.detonates_at,
        }),
        ForecastLimits::default(),
        work,
    )?;
    let hypothetical_id = present
        .hypothetical_id()
        .ok_or(PlantFailure::InvalidRequest)?;
    let first_publication = add_tick(request.execution_tick, 2)?;
    if present
        .evolution()
        .hypothetical_removed(hypothetical_id, first_publication)
    {
        return Err(PlantFailure::Unsupported);
    }
    let route_request = RouteRequest {
        start_window,
        settled_stop_tick: request.settled_stop_tick,
        survive_until,
        mode: RouteMode::CertifiedOnly,
        limits: request.limits,
    };
    let route = if let Some(refuge) = refuge {
        plan_route_to_refuge(
            observation,
            &[&absent, &present],
            route_request,
            first_step,
            cell,
            refuge,
            work,
        )
    } else {
        plan_route_seeded(
            observation,
            &[&absent, &present],
            route_request,
            Some(first_step),
            Some(cell),
            None,
            work,
        )
    }?;
    Ok(PlantCertificate::new(
        route,
        placement,
        hypothetical_id,
        absent.evolution().clone(),
        present.evolution().clone(),
        actor.weapon,
    ))
}

fn actor_weapon(observation: &Observation) -> Option<SpecialWeapon> {
    observation
        .parts()
        .actors
        .iter()
        .find(|actor| actor.seat() == observation.parts().recipient)
        .and_then(|actor| match actor {
            ObservedActor::Visible(actor) => actor.weapon,
            ObservedActor::Hidden { .. } => None,
        })
}

fn effect_active(
    basis: Tick,
    at: Tick,
    actor: &crate::cpu::VisibleActor,
    kind: EffectKind,
) -> Result<bool, PlantFailure> {
    actor
        .effects
        .iter()
        .filter(|effect| effect.kind == kind)
        .try_fold(false, |active, effect| {
            let expiry = basis
                .0
                .checked_add(u64::from(effect.remaining_ticks))
                .ok_or(PlantFailure::InvalidRequest)?;
            Ok(active || expiry > at.0)
        })
}

pub(crate) fn derive_bomb_modifiers(
    actor: &crate::cpu::VisibleActor,
    basis: Tick,
    at: Tick,
    rules: RulesConfig,
) -> Result<(u16, u64), PlantFailure> {
    let short_flame = effect_active(basis, at, actor, EffectKind::ShortFlame)?;
    let golden_flame = effect_active(basis, at, actor, EffectKind::GoldenFlame)?;
    let short_fuse = effect_active(basis, at, actor, EffectKind::ShortFuse)?;
    let long_fuse = effect_active(basis, at, actor, EffectKind::LongFuse)?;
    let active = |kind| match kind {
        EffectKind::ShortFlame => short_flame,
        EffectKind::GoldenFlame => golden_flame,
        EffectKind::ShortFuse => short_fuse,
        EffectKind::LongFuse => long_fuse,
        _ => false,
    };
    let range = effective_bomb_range(actor.bomb_range, active);
    let fuse = effective_bomb_fuse_ticks(rules.bomb_fuse_ticks(), active)
        .ok_or(PlantFailure::InvalidRequest)?;
    Ok((range, fuse))
}

fn origin_is_free(observation: &Observation, cell: crate::game::CellId) -> bool {
    let parts = observation.parts();
    parts
        .arena
        .index(cell)
        .and_then(|index| parts.arena.terrain.get(index))
        == Some(&Terrain::Floor)
        && !parts.arena.blocks.contains(&cell)
        && !parts
            .oil
            .iter()
            .any(|oil| oil.cell == cell && oil.state == OilBarrelState::Unlit)
        && matches!(&parts.bombs, BombKnowledge::Visible(bombs) if !bombs.iter().any(|bomb| bomb.cell == cell))
}

fn add_tick(tick: Tick, amount: u64) -> Result<Tick, PlantFailure> {
    tick.0
        .checked_add(amount)
        .map(Tick)
        .ok_or(PlantFailure::InvalidRequest)
}

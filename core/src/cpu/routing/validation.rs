use crate::cpu::forecast::{HazardForecast, TickWindow};
use crate::cpu::{MemoryKey, Observation, ObservedActor, VisibleActor};
use crate::game::geometry::position_overlaps_cell;
use crate::game::{EffectKind, Position, Terrain, Tick};

use super::RouteFailure;

pub(super) const MAX_EXECUTION_DELAY_TICKS: u64 = 10;

pub(super) struct Context<'a> {
    pub(super) observation: &'a Observation,
    pub(super) actor: &'a VisibleActor,
    pub(super) key: MemoryKey,
    pub(super) reverse_expiry: Option<Tick>,
    pub(super) slow_expiries: Vec<Tick>,
    pub(super) storm: Option<super::storm::StormContext>,
}

pub(super) fn validate<'a>(
    observation: &'a Observation,
    forecast: &HazardForecast,
    start_window: TickWindow,
    safe_until: Tick,
) -> Result<Context<'a>, RouteFailure> {
    let parts = observation.parts();
    let key = MemoryKey {
        world: parts.world,
        cpu_seat: parts.recipient,
    };
    if forecast.key() != key
        || forecast.basis_tick() != parts.tick
        || forecast.dimensions() != (parts.arena.width, parts.arena.height)
    {
        return Err(RouteFailure::IdentityMismatch);
    }
    validate_request(parts.tick, forecast.horizon_end(), start_window, safe_until)?;
    if !matches!(parts.rules.version, 5..=10) {
        return Err(RouteFailure::Unsupported);
    }
    let actor = parts
        .actors
        .iter()
        .find_map(|candidate| match candidate {
            ObservedActor::Visible(actor) if actor.seat == parts.recipient => Some(actor),
            _ => None,
        })
        .ok_or(RouteFailure::InvalidRequest)?;
    let active_at_start = |kind| {
        actor.effects.iter().any(|effect| {
            effect.kind == kind
                && parts
                    .tick
                    .0
                    .checked_add(u64::from(effect.remaining_ticks))
                    .is_some_and(|expiry| expiry > start_window.start.0)
        })
    };
    if !actor.alive
        || actor.voluntary_actions_frozen
        || active_at_start(EffectKind::Diarrhea)
        || !position_in_arena(actor.position, parts.arena.width, parts.arena.height)
        || overlaps_static_blocker(observation, actor.position)
    {
        return Err(RouteFailure::Unsupported);
    }
    let expiry = |remaining: u16| {
        parts
            .tick
            .0
            .checked_add(u64::from(remaining))
            .map(Tick)
            .ok_or(RouteFailure::InvalidRequest)
    };
    let reverse_expiry = actor
        .effects
        .iter()
        .find(|effect| effect.kind == EffectKind::ReverseControls)
        .map(|effect| expiry(effect.remaining_ticks))
        .transpose()?;
    let slow_foot_applies =
        parts.rules.version < 7 || parts.rules.settings.enabled_items().contains_slow_foot();
    let slow_expiries = actor
        .effects
        .iter()
        .filter(|effect| slow_foot_applies && effect.kind == EffectKind::SlowFoot)
        .map(|effect| expiry(effect.remaining_ticks))
        .collect::<Result<Vec<_>, _>>()?;
    let storm = super::storm::context(observation)?;
    Ok(Context {
        observation,
        actor,
        key,
        reverse_expiry,
        slow_expiries,
        storm,
    })
}

fn validate_request(
    basis: Tick,
    horizon_end: Tick,
    window: TickWindow,
    safe_until: Tick,
) -> Result<(), RouteFailure> {
    let TickWindow { start, end } = window;
    let latest_admitted = basis
        .0
        .checked_add(MAX_EXECUTION_DELAY_TICKS)
        .and_then(|tick| tick.checked_add(1))
        .ok_or(RouteFailure::InvalidRequest)?;
    if start.0 <= basis.0
        || start.0 >= end.0
        || end.0 > latest_admitted
        || end.0 > horizon_end.0
        || safe_until.0 < end.0.saturating_sub(1)
        || safe_until > horizon_end
    {
        return Err(RouteFailure::InvalidRequest);
    }
    Ok(())
}

fn position_in_arena(position: Position, width: u16, height: u16) -> bool {
    position.x >= 0
        && position.y >= 0
        && position.x < i32::from(width) * crate::game::types::UNITS_PER_CELL
        && position.y < i32::from(height) * crate::game::types::UNITS_PER_CELL
}

fn overlaps_static_blocker(observation: &Observation, position: Position) -> bool {
    let parts = observation.parts();
    let pass_blocks = crate::cpu::capabilities::can_pass_known_soft_blocks(observation);
    (0..parts.arena.height).any(|row| {
        (0..parts.arena.width).any(|column| {
            let cell = crate::game::CellId::new(column, row);
            let index = usize::from(row) * usize::from(parts.arena.width) + usize::from(column);
            (parts.arena.terrain[index] == Terrain::Wall
                || (!pass_blocks && parts.arena.blocks.contains(&cell)))
                && position_overlaps_cell(position, cell)
        })
    }) || parts.oil.iter().any(|oil| {
        oil.state == crate::game::OilBarrelState::Unlit
            && position_overlaps_cell(position, oil.cell)
    })
}

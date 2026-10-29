use crate::cpu::forecast::{HazardForecast, TickWindow};
use crate::cpu::policy::{ControlPlan, UtilityMemory};
use crate::cpu::routing::{PulseRequest, RouteMode, certify_pulse};
use crate::cpu::{BombKnowledge, Intent, MovementIntent, Observation, ObservedActor, WorkBudget};
use crate::game::geometry::MovementSegment;
use crate::game::types::{HALF_FOOTPRINT, UNITS_PER_CELL};
use crate::game::{Direction, EffectKind, Equipment, OilBarrelState, Position, Terrain, Tick};

use super::{MAX_PLAN_AGE_TICKS, SurvivalPlan};

pub(super) fn plan(
    observation: &Observation,
    forecast: &HazardForecast,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    ordinary: &ControlPlan,
    work: &mut WorkBudget,
) -> Option<SurvivalPlan> {
    let parts = observation.parts();
    let actor = super::compatibility::actor(observation)?;
    let rivals = visible_rivals(observation)?;
    if !matches!(parts.rules.version, 5..=7)
        || !parts.rules.settings.enabled_items().contains_bracelet()
        || actor.equipment != Some(Equipment::Bracelet)
        || !actor.alive
        || actor.voluntary_actions_frozen
        || !matches!(&parts.bombs, BombKnowledge::Visible(bombs) if bombs.is_empty())
        || !parts.pickups.is_empty()
        || actor.effects.iter().any(|effect| {
            matches!(
                effect.kind,
                EffectKind::SlowFoot | EffectKind::ReverseControls
            )
        })
        || !has_existing_fire(observation, work)
    {
        return None;
    }
    let start_window = TickWindow {
        start: Tick(parts.tick.0.checked_add(1)?),
        end: Tick(parts.tick.0.checked_add(MAX_PLAN_AGE_TICKS + 2)?),
    };
    let hold_ticks = 12;
    let stopped_safe_until = Tick(start_window.end.0.checked_add(u64::from(hold_ticks) + 9)?);
    for rival in &rivals {
        if rival.equipment == Some(Equipment::Bracelet)
            || !rival.alive
            || footprints_overlap(actor.position, rival.position)
            || (parts.rules.version >= 6
                && rivals.iter().any(|third| {
                    third.id != rival.id
                        && third.alive
                        && third.equipment == Some(Equipment::Bracelet)
                }))
        {
            continue;
        }
        for direction in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            work.try_take(1).ok()?;
            let movement = MovementIntent::Direction(direction);
            let Ok(lease) = certify_pulse(
                observation,
                forecast,
                PulseRequest {
                    movement,
                    hold_ticks,
                    start_window,
                    stopped_safe_until,
                    mode: RouteMode::CertifiedOnly,
                    settled_stop_tick,
                },
                work,
            ) else {
                continue;
            };
            let Some(contact_update) =
                straight_contact_update(&lease, rival.position, direction, work)
            else {
                continue;
            };
            let latest_contact = Tick(
                start_window
                    .end
                    .0
                    .checked_sub(1)?
                    .checked_add(u64::from(contact_update - 1))?,
            );
            let victim_sweep = victim_sweep(rival.position, direction)?;
            if !sweep_clear(observation, victim_sweep, work)
            || parts.actors.iter().any(|candidate| matches!(candidate,
                ObservedActor::Visible(candidate)
                    if candidate.seat != parts.recipient && candidate.id != rival.id
                        && candidate.alive && sweep_overlaps_actor(victim_sweep, candidate.position)))
            || !sweep_touches_live_fire(observation, victim_sweep, latest_contact, work)
        {
            continue;
        }
            return Some(SurvivalPlan {
                control: ControlPlan {
                    intent: Intent {
                        movement,
                        manual: None,
                    },
                    hold_ticks,
                    next_memory: if movement == ordinary.intent.movement {
                        ordinary.next_memory
                    } else {
                        UtilityMemory::new(memory.key())
                    },
                },
                lease: Some(lease.with_bracelet_guard(rival.id, direction)),
                execution: None,
            });
        }
    }
    None
}

fn visible_rivals(observation: &Observation) -> Option<Vec<&crate::cpu::VisibleActor>> {
    let parts = observation.parts();
    let mut rivals = Vec::new();
    for actor in &parts.actors {
        match actor {
            ObservedActor::Visible(actor) if actor.seat != parts.recipient => rivals.push(actor),
            ObservedActor::Hidden { seat, .. } if *seat != parts.recipient => return None,
            _ => {}
        }
    }
    (!rivals.is_empty()).then_some(rivals)
}

fn sweep_overlaps_actor(sweep: MovementSegment, position: Position) -> bool {
    let left = i64::from(sweep.start().x.min(sweep.end().x)) - i64::from(HALF_FOOTPRINT);
    let right = i64::from(sweep.start().x.max(sweep.end().x)) + i64::from(HALF_FOOTPRINT);
    let top = i64::from(sweep.start().y.min(sweep.end().y)) - i64::from(HALF_FOOTPRINT);
    let bottom = i64::from(sweep.start().y.max(sweep.end().y)) + i64::from(HALF_FOOTPRINT);
    i64::from(position.x) > left
        && i64::from(position.x) < right
        && i64::from(position.y) > top
        && i64::from(position.y) < bottom
}

fn straight_contact_update(
    lease: &crate::cpu::routing::PulseLease,
    rival: Position,
    direction: Direction,
    work: &mut WorkBudget,
) -> Option<u8> {
    work.try_take(u32::from(lease.hold_ticks())).ok()?;
    for completed in 1..=lease.hold_ticks() {
        let before = lease.pose_after(completed - 1);
        let after = lease.pose_after(completed);
        if before.min() != before.max() || after.min() != after.max() {
            return None;
        }
        let segment = MovementSegment::new(before.min(), after.min());
        let delta = (
            segment.end().x - segment.start().x,
            segment.end().y - segment.start().y,
        );
        let straight = match direction {
            Direction::North => delta.0 == 0 && delta.1 < 0,
            Direction::East => delta.0 > 0 && delta.1 == 0,
            Direction::South => delta.0 == 0 && delta.1 > 0,
            Direction::West => delta.0 < 0 && delta.1 == 0,
        };
        if !straight {
            return None;
        }
        if crate::game::stationary_contact_entry(segment, rival) {
            return Some(completed);
        }
    }
    None
}

fn victim_sweep(position: Position, direction: Direction) -> Option<MovementSegment> {
    let end = match direction {
        Direction::North => Position {
            y: position.y.checked_sub(UNITS_PER_CELL)?,
            ..position
        },
        Direction::East => Position {
            x: position.x.checked_add(UNITS_PER_CELL)?,
            ..position
        },
        Direction::South => Position {
            y: position.y.checked_add(UNITS_PER_CELL)?,
            ..position
        },
        Direction::West => Position {
            x: position.x.checked_sub(UNITS_PER_CELL)?,
            ..position
        },
    };
    Some(MovementSegment::new(position, end))
}

fn sweep_clear(observation: &Observation, sweep: MovementSegment, work: &mut WorkBudget) -> bool {
    let parts = observation.parts();
    if work
        .try_take(u32::from(parts.arena.width) * u32::from(parts.arena.height))
        .is_err()
    {
        return false;
    }
    for row in 0..parts.arena.height {
        for column in 0..parts.arena.width {
            let cell = crate::game::CellId::new(column, row);
            let index = usize::from(row) * usize::from(parts.arena.width) + usize::from(column);
            if sweep.overlaps_cell(cell) && parts.arena.terrain[index] == Terrain::Wall {
                return false;
            }
        }
    }
    let Ok(block_units) = u32::try_from(parts.arena.blocks.len()) else {
        return false;
    };
    let Ok(oil_units) = u32::try_from(parts.oil.len()) else {
        return false;
    };
    work.try_take(block_units.saturating_add(oil_units)).is_ok()
        && !parts
            .arena
            .blocks
            .iter()
            .any(|cell| sweep.overlaps_cell(*cell))
        && !parts
            .oil
            .iter()
            .any(|oil| oil.state == OilBarrelState::Unlit && sweep.overlaps_cell(oil.cell))
}

fn has_existing_fire(observation: &Observation, work: &mut WorkBudget) -> bool {
    let parts = observation.parts();
    let Ok(units) = u32::try_from(parts.flames.len() + parts.oil.len()) else {
        return false;
    };
    work.try_take(units).is_ok()
        && (!parts.flames.is_empty()
            || parts
                .oil
                .iter()
                .any(|oil| oil.state == OilBarrelState::Burning))
}

fn sweep_touches_live_fire(
    observation: &Observation,
    sweep: MovementSegment,
    latest_contact: Tick,
    work: &mut WorkBudget,
) -> bool {
    let parts = observation.parts();
    let Ok(units) = u32::try_from(parts.flames.len() + parts.oil.len()) else {
        return false;
    };
    work.try_take(units).is_ok()
        && (parts.flames.iter().any(|flame| {
            parts
                .tick
                .0
                .checked_add(u64::from(flame.remaining_ticks))
                .is_some_and(|expiry| expiry > latest_contact.0)
                && sweep.overlaps_cell(flame.cell)
        }) || parts
            .oil
            .iter()
            .any(|oil| oil.state == OilBarrelState::Burning && sweep.overlaps_cell(oil.cell)))
}

fn footprints_overlap(left: Position, right: Position) -> bool {
    let diameter = i64::from(2 * HALF_FOOTPRINT);
    (i64::from(left.x) - i64::from(right.x)).abs() < diameter
        && (i64::from(left.y) - i64::from(right.y)).abs() < diameter
}

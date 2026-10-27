use crate::game;
use crate::protocol::{EntityId, Tick};
use std::collections::BTreeMap;

use super::projection_capture_events::capture_report;
use super::projection_capture_wire::*;
use super::projection_facts::{
    ExpandedCapture, OilFacts, OilPhase, ProjectionError, TimedBombFacts, WorldFacts,
};
use super::projection_validation::{validate_reports, validate_world_for_items};
use super::{
    ArenaView, BlastKind, BombTrigger, BombView, DamageGraceTicks, EffectView, EffectViews,
    FlameView, InventoryView, ItemSet, PickupView, RulesVersion, Terrain, VisibleActorView,
};

pub fn capture_expanded(
    state: &game::State,
    reports: &[game::StepReport],
) -> Result<ExpandedCapture, ProjectionError> {
    capture_expanded_for_items(state, reports, super::projection_binding::expanded_items())
}

pub(crate) fn capture_expanded_for_items(
    state: &game::State,
    reports: &[game::StepReport],
    supported_items: ItemSet,
) -> Result<ExpandedCapture, ProjectionError> {
    if state.actors().len() != game::SeatId::DUEL.len() {
        return Err(ProjectionError::InvalidActors);
    }
    validate_legacy_reports(reports)?;
    let public = state.public_view();
    let physical_tick = wire_tick(public.tick)?;
    let rules_version =
        RulesVersion::try_from(public.rules_version).map_err(|_| ProjectionError::InvalidRules)?;
    let actors = [
        capture_actor(state, game::SeatId::One, physical_tick)?,
        capture_actor(state, game::SeatId::Two, physical_tick)?,
    ];
    let enabled_items = if rules_version == RulesVersion::ExpandedV5 {
        item_set(state.rules().enabled_items())?
    } else {
        ItemSet::empty()
    };
    if !enabled_items.is_subset_of(supported_items) {
        return Err(ProjectionError::InvalidItems);
    }
    let captured_bombs = state
        .world()
        .bombs()
        .values()
        .map(|bomb| capture_bomb(*bomb, public.tick, state.timed_bombs_paused()))
        .collect::<Result<Vec<_>, _>>()?;
    let world = validate_world_for_items(
        WorldFacts {
            rules_version,
            physical_tick,
            last_teleport_tick: state.last_teleport_tick().map(wire_tick).transpose()?,
            running_updates: u16::try_from(public.running_updates)
                .map_err(|_| ProjectionError::InvalidClock)?,
            remaining_ticks: u16::try_from(public.remaining_ticks)
                .map_err(|_| ProjectionError::InvalidClock)?,
            result: wire_result(public.result)?,
            arena: capture_arena(&public.arena)?,
            actors,
            last_successful_punch_ticks: state
                .actors()
                .iter()
                .map(|actor| {
                    actor
                        .last_successful_punch_tick()
                        .map(wire_tick)
                        .transpose()
                })
                .collect::<Result<Vec<_>, _>>()?
                .try_into()
                .map_err(|_| ProjectionError::InvalidActors)?,
            last_successful_shove_ticks: state
                .actors()
                .iter()
                .map(|actor| {
                    actor
                        .last_successful_shove_tick()
                        .map(wire_tick)
                        .transpose()
                })
                .collect::<Result<Vec<_>, _>>()?
                .try_into()
                .map_err(|_| ProjectionError::InvalidActors)?,
            soft_wall_grace: state
                .actors()
                .iter()
                .map(|actor| {
                    actor
                        .soft_wall_grace_observations()
                        .map(wire_cell)
                        .collect::<Result<Vec<_>, _>>()
                })
                .collect::<Result<Vec<_>, _>>()?
                .try_into()
                .map_err(|_| ProjectionError::InvalidActors)?,
            unlit_oil_grace: state
                .actors()
                .iter()
                .map(|actor| {
                    actor
                        .unlit_oil_grace
                        .iter()
                        .copied()
                        .map(wire_cell)
                        .collect::<Result<Vec<_>, _>>()
                })
                .collect::<Result<Vec<_>, _>>()?
                .try_into()
                .map_err(|_| ProjectionError::InvalidActors)?,
            bombs: captured_bombs.iter().map(|(bomb, _, _, _)| *bomb).collect(),
            timed_bombs: captured_bombs
                .iter()
                .filter_map(|(bomb, timing, _, _)| timing.map(|facts| (bomb.id, facts)))
                .collect::<BTreeMap<_, _>>(),
            kick_bombs: captured_bombs
                .iter()
                .map(|(bomb, _, kick, _)| (bomb.id, *kick))
                .collect(),
            punch_bombs: captured_bombs
                .iter()
                .map(|(bomb, _, _, punch)| (bomb.id, *punch))
                .collect(),
            flames: public
                .flames
                .iter()
                .map(|flame| {
                    Ok(FlameView {
                        cell: wire_cell(flame.cell)?,
                        expires_at: wire_tick(flame.expires_at)?,
                    })
                })
                .collect::<Result<_, ProjectionError>>()?,
            pickups: public
                .pickups
                .iter()
                .map(|pickup| {
                    Ok(PickupView {
                        cell: wire_cell(pickup.cell)?,
                        kind: wire_pickup(pickup.kind),
                    })
                })
                .collect::<Result<_, ProjectionError>>()?,
            private_pickup_cells: {
                let mut cells = state
                    .world()
                    .pickups
                    .iter()
                    .map(|(cell, _)| wire_cell(cell))
                    .collect::<Result<Vec<_>, _>>()?;
                cells.sort_by_key(|cell| (cell.row(), cell.column()));
                cells
            },
            oil: state
                .world()
                .oil_barrels()
                .map(|(cell, phase)| {
                    Ok(OilFacts {
                        cell: wire_cell(cell)?,
                        phase: match phase {
                            game::OilState::Concealed => OilPhase::Concealed,
                            game::OilState::Pending => OilPhase::Pending,
                            game::OilState::Unlit => OilPhase::Unlit,
                            game::OilState::Burning => OilPhase::Burning,
                        },
                    })
                })
                .collect::<Result<_, ProjectionError>>()?,
            available_items: if rules_version == RulesVersion::ExpandedV5 {
                supported_items
            } else {
                ItemSet::empty()
            },
            enabled_items: if rules_version == RulesVersion::ExpandedV5 {
                enabled_items
            } else {
                ItemSet::empty()
            },
        },
        supported_items,
    )?;
    let facts = reports
        .iter()
        .map(capture_report)
        .collect::<Result<Vec<_>, _>>()?;
    let validated_reports = validate_reports(&world, facts)?;
    Ok(ExpandedCapture {
        rules: *state.rules(),
        world,
        reports: validated_reports,
    })
}

fn capture_actor(
    state: &game::State,
    seat: game::SeatId,
    physical_tick: Tick,
) -> Result<VisibleActorView, ProjectionError> {
    let actor = state.actor(seat).ok_or(ProjectionError::InvalidActors)?;
    let effects = actor
        .effect_observations()
        .map(|(kind, expires_at)| {
            let remaining = expires_at
                .0
                .checked_sub(physical_tick.get())
                .ok_or(ProjectionError::InvalidEffects)?;
            EffectView::new(
                wire_effect(kind),
                u16::try_from(remaining).map_err(|_| ProjectionError::InvalidEffects)?,
            )
            .map_err(|_| ProjectionError::InvalidEffects)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let grace = match actor.damage_grace_observation() {
        Some(grace) => {
            physical_tick
                .get()
                .checked_sub(grace.received_at().0)
                .ok_or(ProjectionError::InvalidEffects)?;
            u8::try_from(grace.remaining_ticks(game::Tick(physical_tick.get())))
                .map_err(|_| ProjectionError::InvalidEffects)?
        }
        None => 0,
    };
    let inventory = *actor.inventory();
    Ok(VisibleActorView {
        id: EntityId::new(actor.id().0),
        seat: wire_seat(actor.seat())?,
        position: wire_position(actor.position())?,
        heading: wire_direction(actor.heading()),
        speed_level: wire_speed(actor.speed()),
        alive: actor.alive(),
        bomb_capacity: u8::try_from(actor.bomb_capacity())
            .map_err(|_| ProjectionError::InvalidActors)?,
        bomb_range: u8::try_from(actor.bomb_range()).map_err(|_| ProjectionError::InvalidActors)?,
        live_bomb_count: u8::try_from(state.live_bomb_count(seat))
            .map_err(|_| ProjectionError::InvalidActors)?,
        inventory: InventoryView {
            equipment: inventory.equipment().map(wire_equipment),
            weapon: inventory.weapon().map(wire_weapon),
        },
        effects: EffectViews::new(effects).map_err(|_| ProjectionError::InvalidEffects)?,
        damage_grace_remaining_ticks: DamageGraceTicks::new(grace)
            .map_err(|_| ProjectionError::InvalidEffects)?,
    })
}

fn validate_legacy_reports(reports: &[game::StepReport]) -> Result<(), ProjectionError> {
    if reports.iter().any(|report| {
        report.traces.len() != game::SeatId::DUEL.len()
            || report
                .events
                .iter()
                .any(|event| !event.audience.is_subset_of(game::EventAudience::Both))
            || report
                .forced_movement
                .is_some_and(|forced| !game::SeatId::DUEL.contains(&forced.seat))
    }) {
        Err(ProjectionError::InvalidReports)
    } else {
        Ok(())
    }
}

fn capture_arena(arena: &game::ArenaView) -> Result<ArenaView, ProjectionError> {
    Ok(ArenaView {
        width: u8::try_from(arena.width).map_err(|_| ProjectionError::InvalidArena)?,
        height: u8::try_from(arena.height).map_err(|_| ProjectionError::InvalidArena)?,
        terrain: arena
            .terrain
            .iter()
            .map(|terrain| match terrain {
                game::Terrain::Floor => Terrain::Floor,
                game::Terrain::Wall => Terrain::Wall,
            })
            .collect(),
        blocks: arena
            .blocks
            .iter()
            .copied()
            .map(wire_cell)
            .collect::<Result<_, _>>()?,
    })
}

fn capture_bomb(
    bomb: game::Bomb,
    at: game::Tick,
    paused: bool,
) -> Result<
    (
        BombView,
        Option<TimedBombFacts>,
        super::projection_facts::KickBombFacts,
        super::projection_facts::PunchBombFacts,
    ),
    ProjectionError,
> {
    let view = game::BombView::from(bomb);
    let (trigger, timed_bomb) = match view.trigger {
        game::BombTrigger::Timed { detonates_at } => {
            // Preserve the absolute deadline check before a zero observation can alias
            // another allowed captured duration in a frozen result.
            let remaining = detonates_at
                .0
                .checked_sub(at.0)
                .ok_or(ProjectionError::InvalidBombs)?;
            let trigger = BombTrigger::timed(
                u16::try_from(remaining).map_err(|_| ProjectionError::InvalidBombs)?,
                paused,
            )
            .map_err(|_| ProjectionError::InvalidBombs)?;
            let history = bomb.timed_history().ok_or(ProjectionError::InvalidBombs)?;
            (
                trigger,
                Some(TimedBombFacts {
                    captured_fuse_ticks: history.captured_fuse_ticks,
                    paused_ticks: history.paused_ticks,
                    detonates_at: wire_tick(detonates_at)?,
                    paused,
                }),
            )
        }
        game::BombTrigger::Remote => (BombTrigger::remote(), None),
        game::BombTrigger::Contact => (BombTrigger::contact(), None),
    };
    let bomb_view = BombView {
        id: EntityId::new(view.id.0),
        cell: wire_cell(view.cell)?,
        owner: EntityId::new(view.owner.0),
        range: u8::try_from(view.range).map_err(|_| ProjectionError::InvalidBombs)?,
        placed_at: wire_tick(view.placed_at)?,
        trigger,
        blast: match view.blast {
            game::BlastKind::Standard => BlastKind::Standard,
            game::BlastKind::Piercing => BlastKind::Piercing,
            game::BlastKind::Power => BlastKind::Power,
        },
    };
    Ok((
        bomb_view,
        timed_bomb,
        super::projection_kick::capture_bomb(bomb, bomb_view),
        super::projection_punch::capture_bomb(bomb, bomb_view),
    ))
}

#[cfg(test)]
#[path = "projection_capture_tests.rs"]
mod tests;

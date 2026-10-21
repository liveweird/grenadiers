use crate::game;
use crate::protocol::{EntityId, Tick};
use std::collections::BTreeMap;

use super::projection_capture_wire::*;
use super::projection_facts::{
    EventFacts, ExpandedCapture, ProjectionError, ReportFacts, TimedBombFacts, WorldFacts,
};
use super::projection_validation::{validate_reports, validate_world};
use super::{
    ArenaView, BlastKind, BombTrigger, BombView, DamageGraceTicks, EffectView, EffectViews,
    EventKind, FlameView, InventoryView, ItemSet, PickupView, RulesVersion, Terrain,
    VisibleActorView,
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
    let world = validate_world(WorldFacts {
        rules_version,
        physical_tick,
        running_updates: u16::try_from(public.running_updates)
            .map_err(|_| ProjectionError::InvalidClock)?,
        remaining_ticks: u16::try_from(public.remaining_ticks)
            .map_err(|_| ProjectionError::InvalidClock)?,
        result: wire_result(public.result),
        arena: capture_arena(&public.arena)?,
        actors,
        bombs: captured_bombs.iter().map(|(bomb, _)| *bomb).collect(),
        timed_bombs: captured_bombs
            .iter()
            .filter_map(|(bomb, timing)| timing.map(|facts| (bomb.id, facts)))
            .collect::<BTreeMap<_, _>>(),
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
    })?;
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
    let actor = &state.actors()[seat.index()];
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
        seat: wire_seat(actor.seat()),
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
) -> Result<(BombView, Option<TimedBombFacts>), ProjectionError> {
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
    Ok((
        BombView {
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
        },
        timed_bomb,
    ))
}

fn capture_event_bomb(
    bomb: game::BombView,
    event_tick: game::Tick,
    timing: Option<game::events::TimedBombEvent>,
) -> Result<(BombView, Option<TimedBombFacts>), ProjectionError> {
    if matches!(
        bomb.trigger,
        game::BombTrigger::Timed { detonates_at } if detonates_at.0 < event_tick.0
    ) {
        return Err(ProjectionError::InvalidEvent);
    }
    let (trigger, timed_bomb) = match bomb.trigger {
        game::BombTrigger::Timed { detonates_at } => {
            let remaining = detonates_at
                .0
                .checked_sub(event_tick.0)
                .ok_or(ProjectionError::InvalidEvent)?;
            let facts = timing.ok_or(ProjectionError::InvalidEvent)?;
            (
                BombTrigger::timed(
                    u16::try_from(remaining).map_err(|_| ProjectionError::InvalidEvent)?,
                    facts.paused,
                )
                .map_err(|_| ProjectionError::InvalidEvent)?,
                Some(TimedBombFacts {
                    captured_fuse_ticks: facts.history.captured_fuse_ticks,
                    paused_ticks: facts.history.paused_ticks,
                    detonates_at: wire_tick(facts.detonates_at)
                        .map_err(|_| ProjectionError::InvalidEvent)?,
                    paused: facts.paused,
                }),
            )
        }
        game::BombTrigger::Remote => (BombTrigger::remote(), None),
        game::BombTrigger::Contact => (BombTrigger::contact(), None),
    };
    Ok((
        BombView {
            id: EntityId::new(bomb.id.0),
            cell: wire_cell(bomb.cell)?,
            owner: EntityId::new(bomb.owner.0),
            range: u8::try_from(bomb.range).map_err(|_| ProjectionError::InvalidEvent)?,
            placed_at: wire_tick(bomb.placed_at)?,
            trigger,
            blast: match bomb.blast {
                game::BlastKind::Standard => BlastKind::Standard,
                game::BlastKind::Piercing => BlastKind::Piercing,
                game::BlastKind::Power => BlastKind::Power,
            },
        },
        timed_bomb,
    ))
}

fn capture_report(report: &game::StepReport) -> Result<ReportFacts, ProjectionError> {
    let tick = wire_tick(report.tick)?;
    Ok(ReportFacts {
        tick,
        events: report
            .events
            .iter()
            .map(|event| {
                let (kind, timed_bomb) = match event.kind {
                    game::StepEventKind::BombPlaced(bomb) => {
                        let (bomb, timing) =
                            capture_event_bomb(bomb, event.tick, event.timed_bomb)?;
                        (EventKind::BombPlaced { bomb }, timing)
                    }
                    game::StepEventKind::BombDetonated(bomb) => {
                        let (bomb, timing) =
                            capture_event_bomb(bomb, event.tick, event.timed_bomb)?;
                        (EventKind::BombDetonated { bomb }, timing)
                    }
                    game::StepEventKind::ActorEliminated { id, seat } => (
                        EventKind::ActorEliminated {
                            id: EntityId::new(id.0),
                            seat: wire_seat(seat),
                        },
                        None,
                    ),
                    game::StepEventKind::PickupCollected {
                        id,
                        seat,
                        cell,
                        kind,
                    } => (
                        EventKind::PickupCollected {
                            id: EntityId::new(id.0),
                            seat: wire_seat(seat),
                            cell: wire_cell(cell)?,
                            kind: wire_pickup(kind),
                        },
                        None,
                    ),
                    game::StepEventKind::RoundEnded(result) => (
                        EventKind::RoundEnded {
                            result: wire_result(Some(result))
                                .ok_or(ProjectionError::InvalidEvent)?,
                        },
                        None,
                    ),
                    game::StepEventKind::DamagePrevented { id, seat, source } => (
                        EventKind::DamagePrevented {
                            id: EntityId::new(id.0),
                            seat: wire_seat(seat),
                            source: match source {
                                game::DamagePreventionSource::Heart => {
                                    super::DamagePreventionSource::Heart
                                }
                                game::DamagePreventionSource::Shield => {
                                    super::DamagePreventionSource::Shield
                                }
                            },
                        },
                        None,
                    ),
                };
                Ok(EventFacts {
                    tick: wire_tick(event.tick)?,
                    audience: event.audience,
                    kind,
                    timed_bomb,
                })
            })
            .collect::<Result<_, ProjectionError>>()?,
    })
}
#[cfg(test)]
#[path = "projection_capture_tests.rs"]
mod tests;

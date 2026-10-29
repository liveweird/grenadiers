use std::collections::BTreeSet;

use crate::game;
use crate::protocol::EntityId;

use super::projection_capture_wire as wire;
use super::projection_facts::{
    BombHistory, EventFacts, ExpandedCapture, ForcedMovementFacts, OilFacts, OilPhase,
    PrivatePickupFacts, PrivatePickupPhase, ProjectionProfile, ReportFacts, StormCloudFacts,
    TriggerHistory, WorldFacts,
};
use super::{
    ArenaView, FlameView, ItemMask, ItemSet, PickupView, ProjectionError, RulesVersion, Terrain,
};

const MAX_REPORTS: usize = 3;
const MAX_EVENTS_PER_REPORT: usize = 49;

pub fn capture_expanded(
    state: &game::State,
    reports: &[game::StepReport],
) -> Result<ExpandedCapture, ProjectionError> {
    let version =
        u8::try_from(state.rules().version()).map_err(|_| ProjectionError::InvalidRules)?;
    let rules_version = RulesVersion::new(version).map_err(|_| ProjectionError::InvalidRules)?;
    capture_with_profile(
        state,
        reports,
        ProjectionProfile {
            rules: *state.rules(),
            available_items: super::projection_supported_items::available_items_for(rules_version),
            storm_cloud: crate::game::StormCloudConfig::Disabled,
        },
    )
}

pub(crate) fn capture_with_profile(
    state: &game::State,
    reports: &[game::StepReport],
    profile: ProjectionProfile,
) -> Result<ExpandedCapture, ProjectionError> {
    let public = state.public_view();
    let count = public.actors.len();
    if !(2..=4).contains(&count) || (public.rules_version < 6 && count != 2) {
        return Err(ProjectionError::InvalidActors);
    }
    let physical_tick = wire::tick(public.tick)?;
    let rules_version =
        u8::try_from(public.rules_version).map_err(|_| ProjectionError::InvalidRules)?;
    if profile.rules != *state.rules()
        || profile.rules.version() != public.rules_version
        || profile.storm_cloud != profile.rules.storm_cloud_config()
    {
        return Err(ProjectionError::InvalidRules);
    }
    let available_items = profile.available_items;
    let enabled_items = item_set(state.rules().enabled_items())?;
    if !enabled_items.is_subset_of(available_items) {
        return Err(ProjectionError::InvalidItems);
    }
    let actors = state
        .actors()
        .iter()
        .map(|actor| super::projection_actor_validation::capture(state, actor, physical_tick))
        .collect::<Result<Vec<_>, _>>()?;
    let captured_bombs = state
        .world()
        .bombs()
        .values()
        .map(|bomb| {
            Ok((
                wire::bomb_view((*bomb).into(), public.tick, state.timed_bombs_paused())?,
                bomb_history(*bomb, state.timed_bombs_paused())?,
            ))
        })
        .collect::<Result<Vec<_>, ProjectionError>>()?;
    let world = WorldFacts {
        rules_version,
        physical_tick,
        last_teleport_tick: state.last_teleport_tick().map(wire::tick).transpose()?,
        running_updates: u16::try_from(public.running_updates)
            .map_err(|_| ProjectionError::InvalidClock)?,
        remaining_ticks: u16::try_from(public.remaining_ticks)
            .map_err(|_| ProjectionError::InvalidClock)?,
        result: public.result.map(wire::result),
        arena: capture_arena(&public.arena)?,
        actors,
        bombs: captured_bombs.iter().map(|(view, _)| *view).collect(),
        bomb_history: captured_bombs.iter().map(|(_, history)| *history).collect(),
        flames: public
            .flames
            .iter()
            .map(|flame| {
                Ok(FlameView {
                    cell: wire::cell(flame.cell)?,
                    expires_at: wire::tick(flame.expires_at)?,
                })
            })
            .collect::<Result<_, ProjectionError>>()?,
        pickups: public
            .pickups
            .iter()
            .map(|pickup| {
                Ok(PickupView {
                    cell: wire::cell(pickup.cell)?,
                    kind: wire::pickup(pickup.kind),
                })
            })
            .collect::<Result<_, ProjectionError>>()?,
        private_pickup_cells: {
            let mut cells = state
                .world()
                .pickups
                .iter()
                .map(|(cell, _)| wire::cell(cell))
                .collect::<Result<Vec<_>, _>>()?;
            cells.sort_by_key(|cell| (cell.row(), cell.column()));
            cells
        },
        private_pickups: {
            let mut pickups = state
                .world()
                .pickup_observations()
                .map(|(cell, kind, phase)| {
                    Ok(PrivatePickupFacts {
                        cell: wire::cell(cell)?,
                        kind: wire::pickup(kind),
                        phase: match phase {
                            game::PickupLayer::Concealed => PrivatePickupPhase::Concealed,
                            game::PickupLayer::Pending => PrivatePickupPhase::Pending,
                            game::PickupLayer::Exposed => PrivatePickupPhase::Exposed,
                        },
                    })
                })
                .collect::<Result<Vec<_>, ProjectionError>>()?;
            pickups.sort_by_key(|pickup| (pickup.cell.row(), pickup.cell.column()));
            pickups
        },
        oil: state
            .world()
            .oil_barrels()
            .map(|(cell, phase)| {
                Ok(OilFacts {
                    cell: wire::cell(cell)?,
                    phase: match phase {
                        game::OilState::Concealed => OilPhase::Concealed,
                        game::OilState::Pending => OilPhase::Pending,
                        game::OilState::Unlit => OilPhase::Unlit,
                        game::OilState::Burning => OilPhase::Burning,
                    },
                })
            })
            .collect::<Result<_, ProjectionError>>()?,
        available_items,
        enabled_items,
        storm_cloud: state
            .storm_cloud()
            .map(|cloud| {
                Ok(StormCloudFacts {
                    position: wire::position(cloud.position)?,
                    direction: wire::direction(cloud.direction),
                    leg_remaining_ticks: cloud.leg_remaining_ticks,
                })
            })
            .transpose()?,
    };
    let captured_reports = capture_reports(state, reports)?;
    validate_capture(&world, &captured_reports)?;
    super::projection_validation::validate(&world, &captured_reports, state.rules())?;
    Ok(ExpandedCapture {
        rules: profile.rules,
        world,
        reports: captured_reports,
    })
}

fn capture_reports(
    state: &game::State,
    reports: &[game::StepReport],
) -> Result<Vec<ReportFacts>, ProjectionError> {
    if reports.len() > MAX_REPORTS {
        return Err(ProjectionError::InvalidReports);
    }
    reports
        .iter()
        .map(|report| {
            if report.traces.len() != state.player_count().get()
                || report.events.len() > MAX_EVENTS_PER_REPORT
            {
                return Err(ProjectionError::InvalidReports);
            }
            Ok(ReportFacts {
                tick: wire::tick(report.tick)?,
                trace_count: report.traces.len(),
                events: report
                    .events
                    .iter()
                    .map(|event| {
                        Ok(EventFacts {
                            tick: wire::tick(event.tick)?,
                            audience: event.audience,
                            kind: wire::event_kind(
                                &event.kind,
                                event.tick,
                                event.timed_bomb.is_some_and(|facts| facts.paused),
                            )?,
                            bomb_history: event_bomb_history(event)?,
                        })
                    })
                    .collect::<Result<_, ProjectionError>>()?,
                forced_movements: report
                    .forced_movements
                    .iter()
                    .map(|forced| {
                        Ok(ForcedMovementFacts {
                            pusher: wire::seat(forced.pusher),
                            victim: wire::seat(forced.seat),
                            start: wire::position(forced.segment.start())?,
                            end: wire::position(forced.segment.end())?,
                        })
                    })
                    .collect::<Result<_, ProjectionError>>()?,
            })
        })
        .collect()
}

fn bomb_history(bomb: game::Bomb, paused: bool) -> Result<BombHistory, ProjectionError> {
    let trigger = match bomb.trigger() {
        game::BombTrigger::Timed { detonates_at } => {
            let history = bomb.timed_history().ok_or(ProjectionError::InvalidBombs)?;
            TriggerHistory::Timed {
                captured_fuse_ticks: history.captured_fuse_ticks,
                paused_ticks: history.paused_ticks,
                detonates_at: wire::tick(detonates_at)?,
                paused,
            }
        }
        game::BombTrigger::Remote => TriggerHistory::Remote,
        game::BombTrigger::Contact => TriggerHistory::Contact,
    };
    Ok(BombHistory {
        id: EntityId::new(bomb.id().0),
        trigger,
        motion: bomb
            .motion()
            .map(|motion| (wire::direction(motion.direction()), motion.countdown())),
        has_been_kicked: bomb.has_been_kicked(),
        has_been_punched: bomb.has_been_punched(),
    })
}

fn event_bomb_history(event: &game::StepEvent) -> Result<Option<BombHistory>, ProjectionError> {
    let bomb = match event.kind {
        game::StepEventKind::BombPlaced(bomb) | game::StepEventKind::BombDetonated(bomb) => bomb,
        _ => return Ok(None),
    };
    let trigger = match bomb.trigger {
        game::BombTrigger::Timed { .. } => {
            let facts = event.timed_bomb.ok_or(ProjectionError::InvalidEvent)?;
            TriggerHistory::Timed {
                captured_fuse_ticks: facts.history.captured_fuse_ticks,
                paused_ticks: facts.history.paused_ticks,
                detonates_at: wire::tick(facts.detonates_at)?,
                paused: facts.paused,
            }
        }
        game::BombTrigger::Remote => TriggerHistory::Remote,
        game::BombTrigger::Contact => TriggerHistory::Contact,
    };
    Ok(Some(BombHistory {
        id: EntityId::new(bomb.id.0),
        trigger,
        motion: event
            .bomb_motion
            .map(|motion| (wire::direction(motion.direction()), motion.countdown())),
        has_been_kicked: event.bomb_has_been_kicked,
        has_been_punched: event.bomb_has_been_punched,
    }))
}

fn capture_arena(arena: &game::ArenaView) -> Result<ArenaView, ProjectionError> {
    Ok(ArenaView {
        width: u8::try_from(arena.width).map_err(|_| ProjectionError::InvalidArena)?,
        height: u8::try_from(arena.height).map_err(|_| ProjectionError::InvalidArena)?,
        terrain: arena
            .terrain
            .iter()
            .map(|value| match value {
                game::Terrain::Floor => Terrain::Floor,
                game::Terrain::Wall => Terrain::Wall,
            })
            .collect(),
        blocks: arena
            .blocks
            .iter()
            .copied()
            .map(wire::cell)
            .collect::<Result<_, _>>()?,
    })
}

fn validate_capture(world: &WorldFacts, reports: &[ReportFacts]) -> Result<(), ProjectionError> {
    let count = world.actors.len();
    let player_count =
        game::PlayerCount::try_from(count).map_err(|()| ProjectionError::InvalidActors)?;
    let audience = game::EventAudience::all(player_count);
    super::projection_geometry::validate_dimensions(world)?;
    if world.bombs.len() > super::MAX_BOMBS {
        return Err(ProjectionError::InvalidArena);
    }
    if world
        .actors
        .iter()
        .enumerate()
        .any(|(index, actor)| actor.view.seat.index() != index)
    {
        return Err(ProjectionError::InvalidActors);
    }
    let ids: BTreeSet<_> = world.actors.iter().map(|actor| actor.view.id).collect();
    if ids.len() != count || world.bombs.iter().any(|bomb| !ids.contains(&bomb.owner)) {
        return Err(ProjectionError::InvalidActors);
    }
    let invalid_audience = reports
        .iter()
        .flat_map(|report| &report.events)
        .any(|event| !event.audience.is_subset_of(audience));
    let invalid_movement = reports
        .iter()
        .flat_map(|report| &report.forced_movements)
        .any(|movement| {
            movement.pusher == movement.victim
                || movement.pusher.index() >= count
                || movement.victim.index() >= count
        });
    if invalid_audience || invalid_movement {
        return Err(ProjectionError::InvalidReports);
    }
    Ok(())
}

fn item_set(value: game::EnabledItems) -> Result<ItemSet, ProjectionError> {
    Ok(ItemSet::from_mask(
        ItemMask::new(value.bits()).map_err(|_| ProjectionError::InvalidItems)?,
    ))
}

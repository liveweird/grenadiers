use crate::game::RulesConfig;

use super::projection_facts::{BombHistory, ReportFacts, TriggerHistory, WorldFacts};
use super::{BombView, ItemId, ProjectionError};
use crate::protocol::v7::BombTriggerObservation;

pub(crate) fn validate(
    world: &WorldFacts,
    reports: &[ReportFacts],
    rules: &RulesConfig,
) -> Result<(), ProjectionError> {
    validate_actor_history(world)?;
    validate_bomb_history(world, rules)?;
    validate_reports(world, reports, rules)
}

fn validate_actor_history(world: &WorldFacts) -> Result<(), ProjectionError> {
    let round_start = round_start(world)?;
    for actor in &world.actors {
        for (marker, item) in [
            (actor.last_successful_punch_tick, ItemId::Punch),
            (actor.last_successful_shove_tick, ItemId::Bracelet),
        ] {
            if marker.is_some_and(|tick| {
                !world.enabled_items.contains(item)
                    || tick.get() <= round_start
                    || tick > world.physical_tick
            }) {
                return Err(ProjectionError::InvalidActors);
            }
        }
        let teleport = actor.last_teleport_tick;
        if actor.view.last_teleport_tick != teleport
            || teleport.is_some_and(|tick| {
                !world.enabled_items.contains(ItemId::Teleporter)
                    || tick.get() <= round_start
                    || tick > world.physical_tick
            })
        {
            return Err(ProjectionError::InvalidActors);
        }
    }
    let latest = world
        .actors
        .iter()
        .filter_map(|actor| actor.last_teleport_tick)
        .max();
    if latest != world.last_teleport_tick
        || latest.is_some_and(|tick| {
            world
                .actors
                .iter()
                .filter(|actor| actor.last_teleport_tick == Some(tick))
                .count()
                != 2
        })
    {
        return Err(ProjectionError::InvalidClock);
    }
    Ok(())
}

fn validate_bomb_history(world: &WorldFacts, rules: &RulesConfig) -> Result<(), ProjectionError> {
    if world.bomb_history.len() != world.bombs.len() {
        return Err(ProjectionError::InvalidBombs);
    }
    let paused_now = world.actors.iter().any(|actor| {
        actor.view.alive
            && actor
                .view
                .effects
                .as_slice()
                .iter()
                .any(|effect| effect.kind() == super::EffectKind::Stopwatch)
    });
    for (bomb, history) in world.bombs.iter().zip(&world.bomb_history) {
        if matches!(
            history.trigger,
            TriggerHistory::Timed { paused, .. } if paused != paused_now
        ) {
            return Err(ProjectionError::InvalidBombs);
        }
        validate_bomb(world, *bomb, *history, world.physical_tick, rules, false)
            .map_err(|()| ProjectionError::InvalidBombs)?;
    }
    Ok(())
}

pub(super) fn validate_bomb(
    world: &WorldFacts,
    bomb: BombView,
    history: BombHistory,
    at: crate::protocol::Tick,
    rules: &RulesConfig,
    placement: bool,
) -> Result<(), ()> {
    if history.id != bomb.id
        || history.motion.is_some_and(|(_, countdown)| countdown > 9)
        || (history.motion.is_some() && !history.has_been_kicked)
        || placement
            && (history.motion.is_some() || history.has_been_kicked || history.has_been_punched)
        || matches!(history.trigger, TriggerHistory::Contact)
            && (history.motion.is_some() || history.has_been_kicked || history.has_been_punched)
        || history.has_been_punched
            && !world.actors.iter().any(|actor| {
                actor
                    .last_successful_punch_tick
                    .is_some_and(|tick| tick >= bomb.placed_at && tick <= at)
            })
    {
        return Err(());
    }
    match (bomb.trigger.observation(), history.trigger) {
        (
            BombTriggerObservation::Timed {
                remaining_ticks,
                paused: observed_paused,
            },
            TriggerHistory::Timed {
                captured_fuse_ticks,
                paused_ticks,
                detonates_at,
                paused,
            },
        ) => {
            let elapsed = at.get().checked_sub(bomb.placed_at.get()).ok_or(())?;
            let base = rules.bomb_fuse_ticks();
            let valid_fuse = captured_fuse_ticks == base
                || world.enabled_items.contains(ItemId::ShortFuse)
                    && captured_fuse_ticks == base.div_ceil(2).max(1)
                || world.enabled_items.contains(ItemId::LongFuse)
                    && base.checked_mul(2) == Some(captured_fuse_ticks);
            let valid = valid_fuse
                && bomb
                    .placed_at
                    .get()
                    .checked_add(captured_fuse_ticks)
                    .and_then(|deadline| deadline.checked_add(paused_ticks))
                    == Some(detonates_at.get())
                && detonates_at.get().checked_sub(at.get()) == Some(u64::from(remaining_ticks))
                && paused_ticks <= elapsed
                && paused == observed_paused
                && (!placement
                    || (elapsed == 0
                        && paused_ticks == 0
                        && u64::from(remaining_ticks) == captured_fuse_ticks))
                && (!(paused || paused_ticks > 0)
                    || world.enabled_items.contains(ItemId::Stopwatch));
            valid.then_some(()).ok_or(())
        }
        (BombTriggerObservation::Remote, TriggerHistory::Remote) => {
            (world.enabled_items.contains(ItemId::RemoteBomb)
                && bomb.blast == super::BlastKind::Standard)
                .then_some(())
                .ok_or(())
        }
        (BombTriggerObservation::Contact, TriggerHistory::Contact) => {
            (world.enabled_items.contains(ItemId::Landmine)
                && bomb.blast == super::BlastKind::Standard)
                .then_some(())
                .ok_or(())
        }
        _ => Err(()),
    }
}

fn validate_reports(
    world: &WorldFacts,
    reports: &[ReportFacts],
    rules: &RulesConfig,
) -> Result<(), ProjectionError> {
    if reports.len() > 3
        || reports
            .windows(2)
            .any(|pair| pair[0].tick.get().checked_add(1) != Some(pair[1].tick.get()))
        || reports
            .iter()
            .any(|report| report.tick > world.physical_tick)
        || reports.iter().any(|report| report.events.len() > 49)
        || reports
            .iter()
            .map(|report| report.events.len())
            .sum::<usize>()
            > super::MAX_SNAPSHOT_EVENTS
    {
        return Err(ProjectionError::InvalidReports);
    }
    for report in reports {
        if report.trace_count != world.actors.len() {
            return Err(ProjectionError::InvalidReports);
        }
        validate_forced_movements(world, report)?;
        for event in &report.events {
            super::projection_event_validation::validate(world, report.tick, event, rules)?;
        }
    }
    Ok(())
}

fn validate_forced_movements(
    world: &WorldFacts,
    report: &ReportFacts,
) -> Result<(), ProjectionError> {
    let start = round_start(world)?;
    if report.forced_movements.len() > world.actors.len() / 2 {
        return Err(ProjectionError::InvalidReports);
    }
    let mut participants = std::collections::BTreeSet::new();
    for forced in &report.forced_movements {
        let Some(pusher) = world.actors.get(forced.pusher.index()) else {
            return Err(ProjectionError::InvalidReports);
        };
        let Some(victim) = world.actors.get(forced.victim.index()) else {
            return Err(ProjectionError::InvalidReports);
        };
        let dx = i32::from(forced.end.x()) - i32::from(forced.start.x());
        let dy = i32::from(forced.end.y()) - i32::from(forced.start.y());
        let distance = dx.unsigned_abs().max(dy.unsigned_abs());
        if !super::projection_geometry::position_in_bounds(world, forced.start)
            || !super::projection_geometry::position_in_bounds(world, forced.end)
            || forced.pusher == forced.victim
            || !participants.insert(forced.pusher.index())
            || !participants.insert(forced.victim.index())
            || !world.enabled_items.contains(ItemId::Bracelet)
            || report.tick.get() <= start
            || distance == 0
            || distance > 1_024
            || (dx != 0) == (dy != 0)
            || pusher
                .last_successful_shove_tick
                .is_none_or(|tick| tick < report.tick)
            || report.tick == world.physical_tick
                && victim.last_teleport_tick != Some(report.tick)
                && victim.view.position != forced.end
        {
            return Err(ProjectionError::InvalidReports);
        }
    }
    Ok(())
}

fn round_start(world: &WorldFacts) -> Result<u64, ProjectionError> {
    world
        .physical_tick
        .get()
        .checked_sub(u64::from(world.running_updates))
        .ok_or(ProjectionError::InvalidClock)
}

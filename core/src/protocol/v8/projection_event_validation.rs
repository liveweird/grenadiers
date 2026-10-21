use crate::protocol::Seat;

use super::BombTriggerObservation;
use super::projection_facts::{EventFacts, ProjectionError, TimedBombFacts, WorldFacts};
use super::projection_items::pickup_item;
use super::projection_validation::terrain_at;
use super::{BlastKind, BombView, EventKind, ItemId, Terrain};

pub(super) fn validate_event(
    world: &WorldFacts,
    report_tick: crate::protocol::Tick,
    event: &EventFacts,
) -> Result<(), ProjectionError> {
    if event.tick != report_tick {
        return Err(ProjectionError::InvalidEvent);
    }
    match event.kind {
        EventKind::BombPlaced { bomb } => {
            if bomb.placed_at != report_tick {
                return Err(ProjectionError::InvalidEvent);
            }
            validate_event_bomb(
                world,
                report_tick,
                bomb,
                event.timed_bomb,
                event.kick_bomb,
                event.punch_bomb,
                true,
            )
        }
        EventKind::BombDetonated { bomb } => validate_event_bomb(
            world,
            report_tick,
            bomb,
            event.timed_bomb,
            event.kick_bomb,
            event.punch_bomb,
            false,
        ),
        EventKind::ActorEliminated { id, seat } => {
            validate_actor_identity(world, id, seat)?;
            reject_non_bomb_timing(event)?;
            world
                .actors
                .iter()
                .find(|actor| actor.id == id)
                .is_some_and(|actor| !actor.alive)
                .then_some(())
                .ok_or(ProjectionError::InvalidEvent)
        }
        EventKind::DamagePrevented { id, seat, source } => {
            reject_non_bomb_timing(event)?;
            validate_actor_identity(world, id, seat)?;
            let required = match source {
                super::DamagePreventionSource::Heart => ItemId::Heart,
                super::DamagePreventionSource::Shield => ItemId::Shield,
            };
            world
                .enabled_items
                .contains(required)
                .then_some(())
                .ok_or(ProjectionError::InvalidItems)
        }
        EventKind::PickupCollected {
            id,
            seat,
            cell,
            kind,
        } => {
            reject_non_bomb_timing(event)?;
            validate_actor_identity(world, id, seat)?;
            if terrain_at(world, cell) != Some(Terrain::Floor) || world.arena.blocks.contains(&cell)
            {
                return Err(ProjectionError::InvalidEvent);
            }
            if let Some(item) = pickup_item(kind)
                && !world.enabled_items.contains(item)
            {
                return Err(ProjectionError::InvalidItems);
            }
            Ok(())
        }
        EventKind::RoundEnded { result } => {
            reject_non_bomb_timing(event)?;
            (world.result == Some(result))
                .then_some(())
                .ok_or(ProjectionError::InvalidEvent)
        }
    }
}

fn validate_event_bomb(
    world: &WorldFacts,
    tick: crate::protocol::Tick,
    bomb: BombView,
    timing: Option<TimedBombFacts>,
    kick: Option<super::projection_facts::KickBombFacts>,
    punch: Option<super::projection_facts::PunchBombFacts>,
    placement: bool,
) -> Result<(), ProjectionError> {
    if !(1..=8).contains(&bomb.range)
        || bomb.placed_at > tick
        || terrain_at(world, bomb.cell) != Some(Terrain::Floor)
        || world.arena.blocks.contains(&bomb.cell)
        || world.actors.iter().any(|actor| actor.id == bomb.id)
        || !world.actors.iter().any(|actor| actor.id == bomb.owner)
    {
        return Err(ProjectionError::InvalidEvent);
    }
    let trigger_item = match bomb.trigger.observation() {
        BombTriggerObservation::Timed {
            remaining_ticks, ..
        } => {
            let timing = timing.ok_or(ProjectionError::InvalidEvent)?;
            validate_timed_bomb(
                world,
                bomb,
                timing,
                tick,
                u64::from(remaining_ticks),
                placement,
            )?;
            None
        }
        BombTriggerObservation::Remote => Some(ItemId::RemoteBomb),
        BombTriggerObservation::Contact => Some(ItemId::Landmine),
    };
    if !matches!(
        bomb.trigger.observation(),
        BombTriggerObservation::Timed { .. }
    ) && timing.is_some()
    {
        return Err(ProjectionError::InvalidEvent);
    }
    super::projection_kick::validate_event(world, bomb, kick, placement)?;
    super::projection_punch::validate_event(world, bomb, punch, placement)?;
    let blast_item = match bomb.blast {
        BlastKind::Standard => None,
        BlastKind::Piercing => Some(ItemId::PiercingBomb),
        BlastKind::Power => Some(ItemId::PowerBomb),
    };
    if trigger_item
        .into_iter()
        .chain(blast_item)
        .any(|item| !world.enabled_items.contains(item))
    {
        return Err(ProjectionError::InvalidItems);
    }
    Ok(())
}

fn reject_non_bomb_timing(event: &EventFacts) -> Result<(), ProjectionError> {
    (event.timed_bomb.is_none() && event.kick_bomb.is_none() && event.punch_bomb.is_none())
        .then_some(())
        .ok_or(ProjectionError::InvalidEvent)
}

pub(super) fn validate_timed_bomb(
    world: &WorldFacts,
    bomb: BombView,
    timing: TimedBombFacts,
    at: crate::protocol::Tick,
    remaining: u64,
    placement: bool,
) -> Result<(), ProjectionError> {
    let supports_stopwatch = world.available_items.contains(ItemId::Stopwatch);
    let enabled_stopwatch = world.enabled_items.contains(ItemId::Stopwatch);
    let observed_paused = matches!(
        bomb.trigger.observation(),
        BombTriggerObservation::Timed { paused: true, .. }
    );
    let valid_history = timing.captured_fuse_ticks > 0
        && bomb
            .placed_at
            .get()
            .checked_add(timing.captured_fuse_ticks)
            .and_then(|deadline| deadline.checked_add(timing.paused_ticks))
            == Some(timing.detonates_at.get())
        && timing.detonates_at.get().checked_sub(at.get()) == Some(remaining)
        && at
            .get()
            .checked_sub(bomb.placed_at.get())
            .is_some_and(|elapsed| timing.paused_ticks <= elapsed)
        && timing.paused == observed_paused
        && (!placement || (timing.paused_ticks == 0 && remaining == timing.captured_fuse_ticks))
        && (!(timing.paused || timing.paused_ticks > 0)
            || (supports_stopwatch && enabled_stopwatch));
    valid_history
        .then_some(())
        .ok_or(ProjectionError::InvalidEvent)
}

fn validate_actor_identity(
    world: &WorldFacts,
    id: crate::protocol::EntityId,
    seat: Seat,
) -> Result<(), ProjectionError> {
    world
        .actors
        .iter()
        .any(|actor| actor.id == id && actor.seat == seat)
        .then_some(())
        .ok_or(ProjectionError::InvalidEvent)
}

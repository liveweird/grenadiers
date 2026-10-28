use crate::game::{
    EXPANDED_GENERATOR_VERSION, GENERATOR_VERSION, GenerationRecord, MULTIPLAYER_GENERATOR_VERSION,
};
use crate::protocol::Counter;
use crate::room::RoomSettings;
use std::collections::BTreeSet;

use super::projection_facts::{BoundPublication, ExpandedCapture};
use super::{MapId, ProjectionError, RoomPhase, RoomView, validate_room_view};

pub fn bind_expanded(
    room: &RoomView,
    state_epoch: Counter,
    capture: &ExpandedCapture,
    generation: Option<GenerationRecord>,
) -> Result<BoundPublication, ProjectionError> {
    validate_room_view(room)?;
    let world = &capture.world;
    let (width, height) = room.configuration.domain().board().dimensions();
    if world.rules_version != room.rules_version
        || capture.rules != expected_rules(room)?
        || world.actors.len() != room.configuration.domain().controllers().len()
        || u16::from(world.arena.width) != width
        || u16::from(world.arena.height) != height
        || world.enabled_items != room.settings.enabled_items()
        || world.available_items != room.available_items
        || !generation_matches(room.map_id, room.rules_version.get(), generation)
        || !state_binding(room, state_epoch, capture)
        || !validate_rebound(room, capture)
    {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    Ok(BoundPublication {
        room: room.clone(),
        state_epoch,
        world: world.clone(),
        reports: capture.reports.clone(),
        generation,
    })
}

fn expected_rules(room: &RoomView) -> Result<crate::game::RulesConfig, ProjectionError> {
    let settings = RoomSettings::new(
        match room.settings.starting_speed_level() {
            0 => crate::game::SpeedLevel::Base,
            1 => crate::game::SpeedLevel::One,
            2 => crate::game::SpeedLevel::Two,
            3 => crate::game::SpeedLevel::Three,
            4 => crate::game::SpeedLevel::Four,
            _ => return Err(ProjectionError::InvalidExpandedBinding),
        },
        room.settings.turn_assist_units(),
        room.settings.wins_to_finish(),
        u64::from(room.settings.bomb_fuse_ticks()),
    )
    .map_err(|_| ProjectionError::InvalidExpandedBinding)?
    .with_enabled_items(
        crate::game::EnabledItems::from_bits(room.settings.enabled_items().mask().bits())
            .ok_or(ProjectionError::InvalidItems)?,
    );
    Ok(match room.rules_version.get() {
        1 => crate::game::RulesConfig::classic_v1(),
        2 => crate::game::RulesConfig::classic_v2(),
        3 => crate::game::RulesConfig::classic_v3(),
        4 => crate::game::RulesConfig::configurable_v4(settings),
        5 => crate::game::RulesConfig::expanded_v5(settings),
        6 => crate::game::RulesConfig::multiplayer_v6(settings),
        _ => return Err(ProjectionError::InvalidRules),
    })
}

fn validate_rebound(room: &RoomView, capture: &ExpandedCapture) -> bool {
    let world = &capture.world;
    let width = u16::from(world.arena.width);
    let height = u16::from(world.arena.height);
    let valid_cell = |cell: crate::protocol::Cell| {
        u16::from(cell.column()) < width && u16::from(cell.row()) < height
    };
    let actor_ids: BTreeSet<_> = world
        .actors
        .iter()
        .map(|actor| actor.view.id.get())
        .collect();
    let bomb_ids: BTreeSet<_> = world.bombs.iter().map(|bomb| bomb.id.get()).collect();
    let bomb_cells: BTreeSet<_> = world
        .bombs
        .iter()
        .map(|bomb| (bomb.cell.column(), bomb.cell.row()))
        .collect();
    let valid_history = world.bomb_history.len() == world.bombs.len()
        && world
            .bombs
            .iter()
            .zip(&world.bomb_history)
            .all(|(bomb, history)| valid_bomb_history(*bomb, *history, room));
    actor_ids.len() == world.actors.len()
        && world
            .actors
            .iter()
            .enumerate()
            .all(|(index, actor)| actor.view.seat.index() == index)
        && bomb_ids.len() == world.bombs.len()
        && bomb_cells.len() == world.bombs.len()
        && valid_history
        && world.arena.blocks.iter().all(|&cell| valid_cell(cell))
        && world.flames.iter().all(|flame| valid_cell(flame.cell))
        && world.pickups.iter().all(|pickup| valid_cell(pickup.cell))
        && world
            .private_pickup_cells
            .iter()
            .all(|&cell| valid_cell(cell))
        && world.oil.iter().all(|oil| valid_cell(oil.cell))
        && world
            .bombs
            .iter()
            .all(|bomb| actor_ids.contains(&bomb.owner.get()) && valid_cell(bomb.cell))
        && world.actors.iter().all(|actor| {
            actor.view.position.x() < width * 1_024
                && actor.view.position.y() < height * 1_024
                && actor
                    .last_successful_punch_tick
                    .is_none_or(|tick| tick <= world.physical_tick)
                && actor
                    .last_successful_shove_tick
                    .is_none_or(|tick| tick <= world.physical_tick)
                && actor.soft_wall_grace.iter().all(|&cell| valid_cell(cell))
                && actor.unlit_oil_grace.iter().all(|&cell| valid_cell(cell))
                && super::validate_effects(&actor.view.effects).is_ok()
                && actor
                    .view
                    .effects
                    .as_slice()
                    .iter()
                    .all(|effect| world.enabled_items.contains(effect_item(effect.kind())))
        })
        && capture.reports.len() <= 3
        && capture
            .reports
            .iter()
            .all(|report| report.tick <= world.physical_tick && report.events.len() <= 49)
        && capture
            .reports
            .iter()
            .flat_map(|report| &report.events)
            .all(|event| match (&event.kind, event.bomb_history) {
                (
                    super::EventKind::BombPlaced { bomb }
                    | super::EventKind::BombDetonated { bomb },
                    Some(history),
                ) => valid_bomb_history(*bomb, history, room),
                (
                    super::EventKind::BombPlaced { .. } | super::EventKind::BombDetonated { .. },
                    None,
                ) => false,
                (_, None) => true,
                (_, Some(_)) => false,
            })
        && room
            .settings
            .enabled_items()
            .is_subset_of(room.available_items)
}

fn valid_bomb_history(
    bomb: super::BombView,
    history: super::projection_facts::BombHistory,
    room: &RoomView,
) -> bool {
    use super::projection_facts::TriggerHistory;
    if history.id != bomb.id
        || (history.motion.is_some() || history.has_been_kicked)
            && !room.settings.enabled_items().contains(super::ItemId::Kick)
        || history.has_been_punched && !room.settings.enabled_items().contains(super::ItemId::Punch)
    {
        return false;
    }
    match history.trigger {
        TriggerHistory::Timed {
            captured_fuse_ticks,
            paused_ticks,
            detonates_at,
            paused,
        } => {
            let base = u64::from(room.settings.bomb_fuse_ticks());
            let enabled = room.settings.enabled_items();
            let valid_fuse = captured_fuse_ticks == base
                || enabled.contains(super::ItemId::ShortFuse)
                    && captured_fuse_ticks == base.div_ceil(2).max(1)
                || enabled.contains(super::ItemId::LongFuse)
                    && base.checked_mul(2) == Some(captured_fuse_ticks);
            detonates_at >= bomb.placed_at
                && valid_fuse
                && (paused_ticks == 0 && !paused || enabled.contains(super::ItemId::Stopwatch))
        }
        TriggerHistory::Remote => {
            room.settings
                .enabled_items()
                .contains(super::ItemId::RemoteBomb)
                && bomb.blast == super::BlastKind::Standard
        }
        TriggerHistory::Contact => {
            room.settings
                .enabled_items()
                .contains(super::ItemId::Landmine)
                && bomb.blast == super::BlastKind::Standard
        }
    }
}

const fn effect_item(kind: super::EffectKind) -> super::ItemId {
    match kind {
        super::EffectKind::Shield => super::ItemId::Shield,
        super::EffectKind::Stopwatch => super::ItemId::Stopwatch,
        super::EffectKind::InvisibilityCloak => super::ItemId::InvisibilityCloak,
        super::EffectKind::GoldenFlame => super::ItemId::GoldenFlame,
        super::EffectKind::ShortFuse => super::ItemId::ShortFuse,
        super::EffectKind::Diarrhea => super::ItemId::Diarrhea,
        super::EffectKind::SlowFoot => super::ItemId::SlowFoot,
        super::EffectKind::ShortFlame => super::ItemId::ShortFlame,
        super::EffectKind::LongFuse => super::ItemId::LongFuse,
        super::EffectKind::ReverseControls => super::ItemId::ReverseControls,
        super::EffectKind::InvisibleBombs => super::ItemId::InvisibleBombs,
    }
}

fn generation_matches(map: MapId, rules: u8, generation: Option<GenerationRecord>) -> bool {
    match (map, rules, generation) {
        (MapId::GeneratedV2, 6, Some(record)) => record.version() == MULTIPLAYER_GENERATOR_VERSION,
        (MapId::FixedV1, _, None) => true,
        (MapId::GeneratedV1, _, Some(record)) => record.version() == GENERATOR_VERSION,
        (MapId::GeneratedV2, _, Some(record)) => record.version() == EXPANDED_GENERATOR_VERSION,
        _ => false,
    }
}

fn state_binding(room: &RoomView, epoch: Counter, capture: &ExpandedCapture) -> bool {
    let world = &capture.world;
    match room.phase {
        RoomPhase::Running => {
            epoch == room.round_id && world.physical_tick == room.tick && world.result.is_none()
        }
        RoomPhase::Result { .. } => completed_binding(room, epoch, capture, None),
        RoomPhase::MatchResult { ended_at, .. } if world.result.is_some() => {
            completed_binding(room, epoch, capture, Some(ended_at))
        }
        RoomPhase::MatchResult { ended_at, .. } => {
            epoch == room.round_id && world.physical_tick == ended_at && world.result.is_none()
        }
        RoomPhase::Waiting | RoomPhase::Countdown { .. } | RoomPhase::Interrupted { .. } => false,
    }
}

fn completed_binding(
    room: &RoomView,
    epoch: Counter,
    capture: &ExpandedCapture,
    outcome_ended_at: Option<crate::protocol::Tick>,
) -> bool {
    room.last_round.is_some_and(|last| {
        epoch == last.round_id
            && capture.world.physical_tick == last.ended_at
            && capture.world.result == Some(last.result)
            && outcome_ended_at.is_none_or(|ended_at| last.ended_at <= ended_at)
    })
}

pub(crate) fn phase_allows_null_body(phase: &RoomPhase) -> bool {
    matches!(
        phase,
        RoomPhase::Waiting | RoomPhase::Countdown { .. } | RoomPhase::Interrupted { .. }
    )
}

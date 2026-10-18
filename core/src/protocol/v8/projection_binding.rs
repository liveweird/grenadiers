use crate::game::{EXPANDED_GENERATOR_VERSION, GenerationRecord, RulesConfig};
use crate::protocol::Counter;

use super::projection_facts::{BoundPublication, ExpandedCapture, ProjectionError};
use super::{
    BlastKind, BombTriggerObservation, EventKind, ItemId, ItemSet, MapId, MatchInterruptReason,
    MatchResult, PickupKind, RoomPhase, RoomView, RulesVersion,
};

pub fn bind_expanded(
    room: &RoomView,
    state_epoch: Counter,
    capture: &ExpandedCapture,
    generation: Option<GenerationRecord>,
) -> Result<BoundPublication, ProjectionError> {
    validate_expanded_room(room)?;
    let settings = super::expanded_settings(room.settings)?;
    let expected_rules = match room.rules_version {
        RulesVersion::ClassicV1 => RulesConfig::classic_v1(),
        RulesVersion::ClassicV2 => RulesConfig::classic_v2(),
        RulesVersion::ClassicV3 => RulesConfig::classic_v3(),
        RulesVersion::ConfigurableV4 => RulesConfig::configurable_v4(settings),
        RulesVersion::ExpandedV5 => RulesConfig::expanded_v5(settings),
    };
    if capture.rules != expected_rules
        || capture.world.0.rules_version != room.rules_version
        || capture.world.0.available_items != available_items(room.rules_version)
        || capture.world.0.enabled_items != room.settings.enabled_items()
        || !generation_matches(room.rules_version, generation)
        || !canonical_gameplay(
            room.rules_version,
            capture,
            u64::from(room.settings.bomb_fuse_ticks()),
        )
        || !state_binding(room, state_epoch, capture)
    {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    Ok(BoundPublication {
        room: room.clone(),
        world: capture.world.clone(),
        reports: capture.reports.clone(),
    })
}

pub(super) fn validate_expanded_room(room: &RoomView) -> Result<(), ProjectionError> {
    if room.map_id != expected_map(room.rules_version)
        || room.available_items != available_items(room.rules_version)
        || !room
            .settings
            .enabled_items()
            .is_subset_of(room.available_items)
        || super::expanded_settings(room.settings).is_err()
    {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    super::projection_room_validation::validate_room_view(room)?;
    Ok(())
}

fn canonical_gameplay(version: RulesVersion, capture: &ExpandedCapture, fuse_ticks: u64) -> bool {
    let world = &capture.world.0;
    let classic = version != RulesVersion::ExpandedV5;
    world.bombs.iter().all(|bomb| {
        bomb.blast == BlastKind::Standard && timed_fuse_at(world.physical_tick, *bomb, fuse_ticks)
    }) && (!classic
        || (world.actors.iter().all(|actor| {
            actor.inventory.equipment.is_none()
                && actor.inventory.weapon.is_none()
                && actor.effects.as_slice().is_empty()
                && actor.damage_grace_remaining_ticks.get() == 0
        }) && world
            .pickups
            .iter()
            .all(|pickup| classic_pickup(pickup.kind))))
        && capture
            .reports
            .0
            .iter()
            .flat_map(|report| &report.events)
            .all(|event| {
                (!classic || event.audience == crate::game::EventAudience::Both)
                    && match event.kind {
                        EventKind::BombPlaced { bomb } => {
                            bomb.blast == BlastKind::Standard
                                && timed_fuse_at(event.tick, bomb, fuse_ticks)
                        }
                        EventKind::BombDetonated { bomb } => {
                            bomb.blast == BlastKind::Standard
                                && timed_fuse_at(event.tick, bomb, fuse_ticks)
                        }
                        EventKind::PickupCollected { kind, .. } => !classic || classic_pickup(kind),
                        EventKind::DamagePrevented { .. } => !classic,
                        EventKind::ActorEliminated { .. } | EventKind::RoundEnded { .. } => true,
                    }
            })
}

fn timed_fuse_at(at: crate::protocol::Tick, bomb: super::BombView, fuse_ticks: u64) -> bool {
    let Some(expected) = bomb
        .placed_at
        .get()
        .checked_add(fuse_ticks)
        .and_then(|deadline| deadline.checked_sub(at.get()))
    else {
        return false;
    };
    matches!(
        bomb.trigger.observation(),
        BombTriggerObservation::Timed { remaining_ticks, paused: false }
            if u64::from(remaining_ticks) == expected
    )
}

fn classic_pickup(kind: PickupKind) -> bool {
    matches!(
        kind,
        PickupKind::BombCapacity | PickupKind::BlastRange | PickupKind::MovementSpeed
    )
}

fn state_binding(room: &RoomView, epoch: Counter, capture: &ExpandedCapture) -> bool {
    let world = &capture.world.0;
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
            && capture.world.0.physical_tick == last.ended_at
            && capture.world.0.result == Some(last.result)
            && outcome_ended_at.is_none_or(|ended_at| last.ended_at <= ended_at)
    })
}

pub(super) fn phase_allows_null_body(phase: &RoomPhase) -> bool {
    matches!(
        phase,
        RoomPhase::Waiting | RoomPhase::Countdown { .. } | RoomPhase::Interrupted { .. }
    ) || matches!(
        phase,
        RoomPhase::MatchResult {
            result: MatchResult::Interrupted {
                reason: MatchInterruptReason::InternalError
            },
            ..
        }
    )
}

pub(super) fn expanded_items() -> ItemSet {
    ItemSet::new([ItemId::Heart, ItemId::Shield, ItemId::GoldenFlame])
        .expect("expanded items are canonical")
}

pub(super) fn available_items(version: RulesVersion) -> ItemSet {
    if version == RulesVersion::ExpandedV5 {
        expanded_items()
    } else {
        ItemSet::empty()
    }
}

fn expected_map(version: RulesVersion) -> MapId {
    match version {
        RulesVersion::ClassicV1 => MapId::FixedV1,
        RulesVersion::ClassicV2 | RulesVersion::ClassicV3 | RulesVersion::ConfigurableV4 => {
            MapId::GeneratedV1
        }
        RulesVersion::ExpandedV5 => MapId::GeneratedV2,
    }
}

pub(super) fn generation_matches(
    version: RulesVersion,
    generation: Option<GenerationRecord>,
) -> bool {
    match (version, generation) {
        (RulesVersion::ClassicV1, None) => true,
        (
            RulesVersion::ClassicV2 | RulesVersion::ClassicV3 | RulesVersion::ConfigurableV4,
            Some(record),
        ) => record.version() == crate::game::GENERATOR_VERSION,
        (RulesVersion::ExpandedV5, Some(record)) => record.version() == EXPANDED_GENERATOR_VERSION,
        _ => false,
    }
}

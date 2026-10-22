use crate::game::{EXPANDED_GENERATOR_VERSION, GenerationRecord, RulesConfig};
use crate::protocol::Counter;

use super::projection_facts::{BoundPublication, ExpandedCapture, ProjectionError, TimedBombFacts};
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
    bind_expanded_for_items(room, state_epoch, capture, generation, expanded_items())
}

pub(crate) fn bind_expanded_for_items(
    room: &RoomView,
    state_epoch: Counter,
    capture: &ExpandedCapture,
    generation: Option<GenerationRecord>,
    supported_items: ItemSet,
) -> Result<BoundPublication, ProjectionError> {
    validate_expanded_room_for_items(room, supported_items)?;
    let settings =
        super::projection_room::expanded_settings_for_items(room.settings, supported_items)?;
    let expected_rules = match room.rules_version {
        RulesVersion::ClassicV1 => RulesConfig::classic_v1(),
        RulesVersion::ClassicV2 => RulesConfig::classic_v2(),
        RulesVersion::ClassicV3 => RulesConfig::classic_v3(),
        RulesVersion::ConfigurableV4 => RulesConfig::configurable_v4(settings),
        RulesVersion::ExpandedV5 => RulesConfig::expanded_v5(settings),
    };
    if capture.rules != expected_rules
        || capture.world.0.rules_version != room.rules_version
        || capture.world.0.available_items
            != available_items_for(room.rules_version, supported_items)
        || capture.world.0.enabled_items != room.settings.enabled_items()
        || !generation_matches(room.rules_version, generation)
        || !canonical_gameplay(
            room.rules_version,
            capture,
            u64::from(room.settings.bomb_fuse_ticks()),
            supported_items,
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

pub(crate) fn validate_expanded_room(room: &RoomView) -> Result<(), ProjectionError> {
    validate_expanded_room_for_items(room, expanded_items())
}

pub(crate) fn validate_expanded_room_for_items(
    room: &RoomView,
    supported_items: ItemSet,
) -> Result<(), ProjectionError> {
    if room.map_id != expected_map(room.rules_version)
        || room.available_items != available_items_for(room.rules_version, supported_items)
        || !room
            .settings
            .enabled_items()
            .is_subset_of(room.available_items)
        || super::projection_room::expanded_settings_for_items(room.settings, supported_items)
            .is_err()
    {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    super::projection_room_validation::validate_room_view(room, supported_items)?;
    Ok(())
}

fn canonical_gameplay(
    version: RulesVersion,
    capture: &ExpandedCapture,
    fuse_ticks: u64,
    supported_items: ItemSet,
) -> bool {
    let world = &capture.world.0;
    let classic = version != RulesVersion::ExpandedV5;
    world.bombs.iter().all(|bomb| {
        canonical_bomb(
            *bomb,
            world.timed_bombs.get(&bomb.id).copied(),
            fuse_ticks,
            world.enabled_items,
        )
    }) && unique_remote_owners(&world.bombs)
        && (!classic
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
                canonical_audience(classic, world, event, supported_items)
                    && match event.kind {
                        EventKind::BombPlaced { bomb } => {
                            canonical_bomb(bomb, event.timed_bomb, fuse_ticks, world.enabled_items)
                        }
                        EventKind::BombDetonated { bomb } => {
                            canonical_bomb(bomb, event.timed_bomb, fuse_ticks, world.enabled_items)
                        }
                        EventKind::PickupCollected { kind, .. } => !classic || classic_pickup(kind),
                        EventKind::DamagePrevented { .. } => !classic,
                        EventKind::ActorEliminated { .. } | EventKind::RoundEnded { .. } => true,
                    }
            })
}

fn canonical_audience(
    classic: bool,
    world: &super::projection_facts::WorldFacts,
    event: &super::projection_facts::EventFacts,
    supported_items: ItemSet,
) -> bool {
    use crate::game::EventAudience;
    if classic {
        return event.audience == EventAudience::Both;
    }
    if matches!(
        event.kind,
        EventKind::BombPlaced { .. } | EventKind::BombDetonated { .. }
    ) && supported_items.contains(ItemId::InvisibleBombs)
        && world.enabled_items.contains(ItemId::InvisibleBombs)
    {
        return true;
    }
    if !world.enabled_items.contains(ItemId::InvisibilityCloak) {
        return event.audience == EventAudience::Both;
    }
    let seat = match event.kind {
        EventKind::BombPlaced { bomb } | EventKind::BombDetonated { bomb } => world
            .actors
            .iter()
            .find(|actor| actor.id == bomb.owner)
            .map(|actor| actor.seat),
        EventKind::PickupCollected { seat, .. } | EventKind::DamagePrevented { seat, .. } => {
            Some(seat)
        }
        EventKind::ActorEliminated { .. } | EventKind::RoundEnded { .. } => {
            return event.audience == EventAudience::Both;
        }
    };
    seat.is_some_and(|seat| {
        event.audience == EventAudience::Both
            || event.audience
                == match seat {
                    crate::protocol::Seat::One => EventAudience::One,
                    crate::protocol::Seat::Two => EventAudience::Two,
                }
    })
}

fn canonical_bomb(
    bomb: super::BombView,
    timing: Option<TimedBombFacts>,
    base_fuse: u64,
    enabled: ItemSet,
) -> bool {
    (bomb.blast == BlastKind::Standard
        || (enabled.contains(ItemId::PiercingBomb) && bomb.blast == BlastKind::Piercing)
        || (enabled.contains(ItemId::PowerBomb) && bomb.blast == BlastKind::Power))
        && match bomb.trigger.observation() {
            BombTriggerObservation::Remote => {
                timing.is_none()
                    && enabled.contains(ItemId::RemoteBomb)
                    && bomb.blast == BlastKind::Standard
            }
            BombTriggerObservation::Timed { .. } => timing.is_some_and(|timing| {
                let observed_paused = matches!(
                    bomb.trigger.observation(),
                    BombTriggerObservation::Timed { paused: true, .. }
                );
                timing.paused == observed_paused
                    && (timing.paused_ticks == 0 && !timing.paused
                        || enabled.contains(ItemId::Stopwatch))
                    && (timing.captured_fuse_ticks == base_fuse
                        || (enabled.contains(ItemId::ShortFuse)
                            && timing.captured_fuse_ticks == base_fuse.div_ceil(2).max(1))
                        || (enabled.contains(ItemId::LongFuse)
                            && base_fuse
                                .checked_mul(2)
                                .is_some_and(|fuse| timing.captured_fuse_ticks == fuse)))
            }),
            BombTriggerObservation::Contact => {
                timing.is_none()
                    && enabled.contains(ItemId::Landmine)
                    && bomb.blast == BlastKind::Standard
            }
        }
}

fn unique_remote_owners(bombs: &[super::BombView]) -> bool {
    let mut owners = std::collections::BTreeSet::new();
    bombs.iter().all(|bomb| {
        !matches!(bomb.trigger.observation(), BombTriggerObservation::Remote)
            || owners.insert(bomb.owner)
    })
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

pub(crate) fn phase_allows_null_body(phase: &RoomPhase) -> bool {
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

pub(crate) fn expanded_items() -> ItemSet {
    ItemSet::new([
        ItemId::Heart,
        ItemId::RemoteBomb,
        ItemId::PiercingBomb,
        ItemId::PowerBomb,
        ItemId::Landmine,
        ItemId::Shield,
        ItemId::InvisibilityCloak,
        ItemId::GoldenFlame,
        ItemId::ShortFuse,
        ItemId::Diarrhea,
        ItemId::SlowFoot,
        ItemId::ShortFlame,
        ItemId::LongFuse,
        ItemId::ReverseControls,
    ])
    .expect("expanded items are canonical")
}

pub(crate) fn available_items_for(version: RulesVersion, supported_items: ItemSet) -> ItemSet {
    if version == RulesVersion::ExpandedV5 {
        supported_items
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

#[cfg(test)]
mod fuse_policy_tests {
    use super::canonical_bomb;
    use crate::protocol::v8::projection_facts::TimedBombFacts;
    use crate::protocol::v8::{BlastKind, BombTrigger, BombView, ItemId, ItemSet};
    use crate::protocol::{Cell, EntityId, Tick};

    fn bomb(duration: u16) -> BombView {
        BombView {
            id: EntityId::new(1),
            cell: Cell::new(1, 1).unwrap(),
            owner: EntityId::new(2),
            range: 2,
            placed_at: Tick::new(10).unwrap(),
            trigger: BombTrigger::timed(duration, false).unwrap(),
            blast: BlastKind::Standard,
        }
    }

    #[test]
    fn odd_policy_accepts_base_half_and_double_historical_captures() {
        let enabled = ItemSet::new([ItemId::ShortFuse, ItemId::LongFuse]).unwrap();
        let facts = |duration| TimedBombFacts {
            captured_fuse_ticks: duration,
            paused_ticks: 0,
            detonates_at: Tick::new(10 + duration).unwrap(),
            paused: false,
        };
        assert!(canonical_bomb(bomb(181), Some(facts(181)), 181, enabled));
        assert!(canonical_bomb(bomb(91), Some(facts(91)), 181, enabled));
        assert!(canonical_bomb(bomb(362), Some(facts(362)), 181, enabled));
        assert!(!canonical_bomb(bomb(180), Some(facts(180)), 181, enabled));
    }
}

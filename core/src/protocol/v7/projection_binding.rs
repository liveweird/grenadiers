use crate::game::SpeedLevel;
use crate::protocol::{Counter, MapId};
use crate::room::RoomSettings;

use super::projection_facts::{BoundPublication, ClassicCapture, ProjectionError, rules_settings};
use super::state::BombTriggerObservation;
use super::{BlastKind, EventKind, PickupKind, RoomView};

pub fn bind_classic(
    room: &RoomView,
    state_epoch: Counter,
    capture: &ClassicCapture,
) -> Result<BoundPublication, ProjectionError> {
    let fuse_ticks = capture.rules.bomb_fuse_ticks();
    if room.rules_version != capture.world.0.rules_version
        || capture.world.0.rules_version != capture.legacy.rules_version
        || capture.world.0.physical_tick != capture.legacy.tick
        || capture.world.0.result != capture.legacy.result
        || room.settings != rules_settings(capture.rules)?
        || room.available_items != super::ItemSet::empty()
        || room.settings.enabled_items() != super::ItemSet::empty()
        || room.map_id != expected_map(capture.world.0.rules_version)
        || !classic_world(capture, fuse_ticks)
        || !classic_reports(&capture.reports.0, fuse_ticks)
    {
        return Err(ProjectionError::InvalidClassicBinding);
    }
    let legacy_room = validate_classic_room(room)?;
    crate::protocol::v6::validate_state_binding(&legacy_room, state_epoch, &capture.legacy)
        .map_err(|_| ProjectionError::InvalidClassicBinding)?;
    Ok(BoundPublication {
        room: room.clone(),
        world: capture.world.clone(),
        reports: capture.reports.clone(),
    })
}

pub(super) fn validate_classic_room(
    room: &RoomView,
) -> Result<crate::protocol::v6::RoomView, ProjectionError> {
    if room.available_items != super::ItemSet::empty()
        || room.settings.enabled_items() != super::ItemSet::empty()
        || room.map_id != expected_map(room.rules_version)
    {
        return Err(ProjectionError::InvalidClassicBinding);
    }
    let legacy = legacy_room(room)?;
    crate::protocol::v6::validate_room_view(&legacy)
        .map_err(|_| ProjectionError::InvalidClassicBinding)?;
    Ok(legacy)
}

fn classic_world(capture: &ClassicCapture, fuse_ticks: u64) -> bool {
    let world = &capture.world.0;
    world.available_items == super::ItemSet::empty()
        && world.enabled_items == super::ItemSet::empty()
        && world.pickups.iter().all(|pickup| {
            matches!(
                pickup.kind,
                PickupKind::BombCapacity | PickupKind::BlastRange | PickupKind::MovementSpeed
            )
        })
        && world.actors.iter().all(|actor| {
            actor.inventory.equipment.is_none()
                && actor.inventory.weapon.is_none()
                && actor.effects.as_slice().is_empty()
                && actor.damage_grace_remaining_ticks.get() == 0
        })
        && world.bombs.iter().all(|bomb| {
            bomb.blast == BlastKind::Standard
                && matches!(
                    bomb.trigger.observation(),
                    BombTriggerObservation::Timed { paused: false, .. }
                )
        })
        && world.bombs.len() == capture.legacy.state.bombs.len()
        && world
            .bombs
            .iter()
            .zip(&capture.legacy.state.bombs)
            .all(|(bomb, legacy)| classic_world_bomb(world.physical_tick, bomb, legacy, fuse_ticks))
}

fn classic_reports(reports: &[super::projection_facts::ReportFacts], fuse_ticks: u64) -> bool {
    reports
        .iter()
        .flat_map(|report| &report.events)
        .all(|event| {
            event.audience == crate::game::EventAudience::Both
                && match event.kind {
                    EventKind::BombPlaced { bomb } | EventKind::BombDetonated { bomb } => {
                        bomb.blast == BlastKind::Standard
                            && classic_event_bomb(event.tick, bomb, fuse_ticks)
                    }
                    EventKind::PickupCollected { kind, .. } => matches!(
                        kind,
                        PickupKind::BombCapacity
                            | PickupKind::BlastRange
                            | PickupKind::MovementSpeed
                    ),
                    EventKind::DamagePrevented { .. } => false,
                    EventKind::ActorEliminated { .. } | EventKind::RoundEnded { .. } => true,
                }
        })
}

fn classic_world_bomb(
    physical_tick: crate::protocol::Tick,
    bomb: &super::BombView,
    legacy: &crate::protocol::BombView,
    fuse_ticks: u64,
) -> bool {
    let Some(expected_deadline) = legacy.placed_at.get().checked_add(fuse_ticks) else {
        return false;
    };
    let Some(expected_remaining) = expected_deadline.checked_sub(physical_tick.get()) else {
        return false;
    };
    legacy.id == bomb.id
        && legacy.cell == bomb.cell
        && legacy.owner == bomb.owner
        && legacy.range == bomb.range
        && legacy.placed_at == bomb.placed_at
        && legacy.detonates_at.get() == expected_deadline
        && matches!(
            bomb.trigger.observation(),
            BombTriggerObservation::Timed {
                remaining_ticks,
                paused: false
            } if u64::from(remaining_ticks) == expected_remaining
        )
}

fn classic_event_bomb(
    event_tick: crate::protocol::Tick,
    bomb: super::BombView,
    fuse_ticks: u64,
) -> bool {
    let expected_remaining = bomb
        .placed_at
        .get()
        .checked_add(fuse_ticks)
        .and_then(|deadline| deadline.checked_sub(event_tick.get()));
    matches!(
        (bomb.trigger.observation(), expected_remaining),
        (
            BombTriggerObservation::Timed {
                remaining_ticks,
                paused: false
            },
            Some(expected)
        ) if u64::from(remaining_ticks) == expected
    )
}

fn expected_map(version: crate::protocol::RulesVersion) -> MapId {
    match version {
        crate::protocol::RulesVersion::ClassicV1 => MapId::FixedV1,
        crate::protocol::RulesVersion::ClassicV2
        | crate::protocol::RulesVersion::ClassicV3
        | crate::protocol::RulesVersion::ConfigurableV4 => MapId::GeneratedV1,
    }
}

fn legacy_room(room: &RoomView) -> Result<crate::protocol::v6::RoomView, ProjectionError> {
    let settings = room.settings;
    let speed = match settings.starting_speed_level() {
        0 => SpeedLevel::Base,
        1 => SpeedLevel::One,
        2 => SpeedLevel::Two,
        3 => SpeedLevel::Three,
        4 => SpeedLevel::Four,
        _ => return Err(ProjectionError::InvalidClassicBinding),
    };
    let domain_settings = RoomSettings::new(
        speed,
        settings.turn_assist_units(),
        settings.wins_to_finish(),
        u64::from(settings.bomb_fuse_ticks()),
    )
    .map_err(|_| ProjectionError::InvalidClassicBinding)?;
    Ok(crate::protocol::v6::RoomView {
        settings: crate::protocol::v6::GameSettings::from_domain(domain_settings),
        settings_revision: room.settings_revision,
        settings_editor: room.settings_editor,
        room_id: room.room_id.clone(),
        room_code: room.room_code.clone(),
        round_id: room.round_id,
        revision: room.revision,
        tick: room.tick,
        rules_version: room.rules_version,
        map_id: room.map_id,
        round_number: crate::protocol::v6::RoundCount::new(room.round_number.get())
            .map_err(|_| ProjectionError::InvalidClassicBinding)?,
        scores: crate::protocol::v6::Scores {
            one: crate::protocol::v6::Wins::new(room.scores.one.get())
                .map_err(|_| ProjectionError::InvalidClassicBinding)?,
            two: crate::protocol::v6::Wins::new(room.scores.two.get())
                .map_err(|_| ProjectionError::InvalidClassicBinding)?,
        },
        last_round: room
            .last_round
            .map(|last| crate::protocol::v6::CompletedRound {
                round_id: last.round_id,
                round_number: last.round_number,
                ended_at: last.ended_at,
                result: last.result,
            }),
        seats: room.seats.map(|seat| crate::protocol::v6::SeatStatus {
            seat: seat.seat,
            occupied: seat.occupied,
            ready: seat.ready,
            rematch_ready: seat.rematch_ready,
            connection: legacy_connection(seat.connection),
        }),
        phase: legacy_phase(&room.phase),
    })
}

fn legacy_connection(value: super::ConnectionStatus) -> crate::protocol::v6::ConnectionStatus {
    match value {
        super::ConnectionStatus::Vacant => crate::protocol::v6::ConnectionStatus::Vacant,
        super::ConnectionStatus::Connected => crate::protocol::v6::ConnectionStatus::Connected,
        super::ConnectionStatus::Disconnected { expires_at } => {
            crate::protocol::v6::ConnectionStatus::Disconnected { expires_at }
        }
        super::ConnectionStatus::Expired => crate::protocol::v6::ConnectionStatus::Expired,
    }
}

fn legacy_phase(value: &super::RoomPhase) -> crate::protocol::v6::RoomPhase {
    match value {
        super::RoomPhase::Waiting => crate::protocol::v6::RoomPhase::Waiting,
        super::RoomPhase::Countdown { kind, ends_at } => {
            crate::protocol::v6::RoomPhase::Countdown {
                kind: match kind {
                    super::CountdownKind::Initial => crate::protocol::v6::CountdownKind::Initial,
                    super::CountdownKind::InterRound => {
                        crate::protocol::v6::CountdownKind::InterRound
                    }
                },
                ends_at: *ends_at,
            }
        }
        super::RoomPhase::Running => crate::protocol::v6::RoomPhase::Running,
        super::RoomPhase::Result { hold_until } => crate::protocol::v6::RoomPhase::Result {
            hold_until: *hold_until,
        },
        super::RoomPhase::MatchResult {
            ended_at,
            result,
            rematch_at,
        } => crate::protocol::v6::RoomPhase::MatchResult {
            ended_at: *ended_at,
            result: legacy_match_result(*result),
            rematch_at: *rematch_at,
        },
        super::RoomPhase::Interrupted { ended_at, reason } => {
            crate::protocol::v6::RoomPhase::Interrupted {
                ended_at: *ended_at,
                reason: legacy_interrupt(*reason),
            }
        }
    }
}

fn legacy_match_result(value: super::MatchResult) -> crate::protocol::v6::MatchResult {
    match value {
        super::MatchResult::Winner { seat, reason } => crate::protocol::v6::MatchResult::Winner {
            seat,
            reason: match reason {
                super::MatchWinnerReason::RoundWins => {
                    crate::protocol::v6::MatchWinnerReason::RoundWins
                }
                super::MatchWinnerReason::Forfeit => {
                    crate::protocol::v6::MatchWinnerReason::Forfeit
                }
            },
        },
        super::MatchResult::Abandoned => crate::protocol::v6::MatchResult::Abandoned,
        super::MatchResult::Interrupted { reason } => {
            crate::protocol::v6::MatchResult::Interrupted {
                reason: legacy_interrupt(reason),
            }
        }
    }
}

fn legacy_interrupt(
    value: super::MatchInterruptReason,
) -> crate::protocol::v6::MatchInterruptReason {
    match value {
        super::MatchInterruptReason::ConnectionLost => {
            crate::protocol::v6::MatchInterruptReason::ConnectionLost
        }
        super::MatchInterruptReason::ServerOverloaded => {
            crate::protocol::v6::MatchInterruptReason::ServerOverloaded
        }
        super::MatchInterruptReason::ServerStopped => {
            crate::protocol::v6::MatchInterruptReason::ServerStopped
        }
        super::MatchInterruptReason::InternalError => {
            crate::protocol::v6::MatchInterruptReason::InternalError
        }
    }
}

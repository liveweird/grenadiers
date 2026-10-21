use crate::game;
use crate::protocol::EntityId;

use super::projection_capture_wire::*;
use super::projection_facts::{
    EventFacts, KickBombFacts, ProjectionError, ReportFacts, TimedBombFacts,
};
use super::{BlastKind, BombTrigger, BombView, EventKind};

fn capture_event_bomb(
    bomb: game::BombView,
    event_tick: game::Tick,
    timing: Option<game::events::TimedBombEvent>,
    motion: Option<game::BombMotion>,
    has_been_kicked: bool,
) -> Result<(BombView, Option<TimedBombFacts>, KickBombFacts), ProjectionError> {
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
    let bomb_view = BombView {
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
    };
    Ok((
        bomb_view,
        timed_bomb,
        super::projection_kick::capture_event_bomb(bomb_view, motion, has_been_kicked),
    ))
}

pub(super) fn capture_report(report: &game::StepReport) -> Result<ReportFacts, ProjectionError> {
    let tick = wire_tick(report.tick)?;
    Ok(ReportFacts {
        tick,
        events: report
            .events
            .iter()
            .map(|event| {
                let (kind, timed_bomb, kick_bomb) = match event.kind {
                    game::StepEventKind::BombPlaced(bomb) => {
                        let (bomb, timing, kick) = capture_event_bomb(
                            bomb,
                            event.tick,
                            event.timed_bomb,
                            event.bomb_motion,
                            event.bomb_has_been_kicked,
                        )?;
                        (EventKind::BombPlaced { bomb }, timing, Some(kick))
                    }
                    game::StepEventKind::BombDetonated(bomb) => {
                        let (bomb, timing, kick) = capture_event_bomb(
                            bomb,
                            event.tick,
                            event.timed_bomb,
                            event.bomb_motion,
                            event.bomb_has_been_kicked,
                        )?;
                        (EventKind::BombDetonated { bomb }, timing, Some(kick))
                    }
                    game::StepEventKind::ActorEliminated { id, seat } => (
                        EventKind::ActorEliminated {
                            id: EntityId::new(id.0),
                            seat: wire_seat(seat),
                        },
                        None,
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
                        None,
                    ),
                    game::StepEventKind::RoundEnded(result) => (
                        EventKind::RoundEnded {
                            result: wire_result(Some(result))
                                .ok_or(ProjectionError::InvalidEvent)?,
                        },
                        None,
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
                        None,
                    ),
                };
                Ok(EventFacts {
                    tick: wire_tick(event.tick)?,
                    audience: event.audience,
                    kind,
                    timed_bomb,
                    kick_bomb,
                })
            })
            .collect::<Result<_, ProjectionError>>()?,
    })
}

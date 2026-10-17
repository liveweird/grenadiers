use crate::game;

use super::events::{EventKind, MAX_EVENT_INDEX, MAX_SNAPSHOT_EVENTS, WireEvent};
use super::projection::{
    ProjectionError, project_bomb_record, project_result, wire_cell, wire_pickup_kind, wire_seat,
    wire_tick,
};
use super::scalars::EntityId;

pub fn project_events(reports: &[game::StepReport]) -> Result<Vec<WireEvent>, ProjectionError> {
    if reports.len() > 3
        || reports
            .windows(2)
            .any(|pair| pair[0].tick.0.checked_add(1) != Some(pair[1].tick.0))
    {
        return Err(ProjectionError::InvalidReportWindow);
    }
    let event_count = reports.iter().try_fold(0_usize, |count, report| {
        count.checked_add(report.events.len())
    });
    if event_count.is_none_or(|count| count > MAX_SNAPSHOT_EVENTS) {
        return Err(ProjectionError::InvalidReportWindow);
    }

    let mut projected = Vec::with_capacity(event_count.unwrap_or(0));
    for report in reports {
        let tick = wire_tick(report.tick)?;
        for (index, event) in report.events.iter().enumerate() {
            if event.tick != report.tick || index > usize::from(MAX_EVENT_INDEX) {
                return Err(ProjectionError::InvalidEvent);
            }
            let kind = match event.kind {
                game::StepEventKind::BombPlaced(bomb) => {
                    if bomb.placed_at != event.tick {
                        return Err(ProjectionError::InvalidEvent);
                    }
                    EventKind::BombPlaced {
                        bomb: project_bomb_record(bomb)
                            .map_err(|_| ProjectionError::InvalidEvent)?,
                    }
                }
                game::StepEventKind::BombDetonated(bomb) => {
                    if bomb.placed_at.0 > event.tick.0 {
                        return Err(ProjectionError::InvalidEvent);
                    }
                    EventKind::BombDetonated {
                        bomb: project_bomb_record(bomb)
                            .map_err(|_| ProjectionError::InvalidEvent)?,
                    }
                }
                game::StepEventKind::ActorEliminated { id, seat } => EventKind::ActorEliminated {
                    id: EntityId::new(id.0),
                    seat: wire_seat(seat),
                },
                game::StepEventKind::PickupCollected {
                    id,
                    seat,
                    cell,
                    kind,
                } => EventKind::PickupCollected {
                    id: EntityId::new(id.0),
                    seat: wire_seat(seat),
                    cell: wire_cell(cell).map_err(|_| ProjectionError::InvalidEvent)?,
                    kind: wire_pickup_kind(kind),
                },
                game::StepEventKind::RoundEnded(result) => EventKind::RoundEnded {
                    result: project_result(Some(result)).ok_or(ProjectionError::InvalidEvent)?,
                },
            };
            projected.push(WireEvent {
                tick,
                index: index as u8,
                kind,
            });
        }
    }
    Ok(projected)
}

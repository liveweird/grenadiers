use crate::protocol::Seat;

use super::projection_facts::{BoundPublication, ProjectionError};
use super::{
    ActorView, EffectKind, EventIndex, GameServerMessage, GameState, HiddenActorView,
    SnapshotEvents, WireEvent,
};

pub fn prepare_snapshot_pair(
    publication: &BoundPublication,
) -> Result<[String; 2], ProjectionError> {
    let one = prepare(publication, Seat::One)?;
    let two = prepare(publication, Seat::Two)?;
    Ok([one, two])
}

fn prepare(publication: &BoundPublication, recipient: Seat) -> Result<String, ProjectionError> {
    let facts = &publication.world.0;
    let recipient_actor = &facts.actors[seat_index(recipient)];
    let hides_bombs = has_effect(recipient_actor, EffectKind::InvisibleBombs);
    let actors = facts.actors.clone().map(|actor| {
        if actor.seat != recipient && has_effect(&actor, EffectKind::InvisibilityCloak) {
            HiddenActorView {
                id: actor.id,
                seat: actor.seat,
                alive: actor.alive,
            }
            .into()
        } else {
            ActorView::from(actor)
        }
    });
    let events = publication
        .reports
        .0
        .iter()
        .flat_map(|report| {
            report
                .events
                .iter()
                .filter(|event| event.audience.includes(domain_seat(recipient)))
                .enumerate()
                .map(|(index, event)| {
                    Ok(WireEvent {
                        tick: event.tick,
                        index: EventIndex::new(
                            u8::try_from(index).map_err(|_| ProjectionError::InvalidReports)?,
                        )
                        .map_err(|_| ProjectionError::InvalidReports)?,
                        kind: event.kind,
                    })
                })
        })
        .collect::<Result<Vec<_>, ProjectionError>>()?;
    let message = GameServerMessage::Snapshot {
        room: publication.room.clone(),
        state: GameState {
            running_updates: facts.running_updates,
            remaining_ticks: facts.remaining_ticks,
            physical_tick: facts.physical_tick,
            arena: facts.arena.clone(),
            actors,
            bombs: if hides_bombs {
                Vec::new()
            } else {
                facts.bombs.clone()
            },
            flames: facts.flames.clone(),
            pickups: facts.pickups.clone(),
        },
        events: SnapshotEvents::new(events).map_err(|_| ProjectionError::InvalidReports)?,
    };
    super::encode_server_message(&message).map_err(|_| ProjectionError::EncodingFailed)
}

fn has_effect(actor: &super::VisibleActorView, kind: EffectKind) -> bool {
    actor
        .effects
        .as_slice()
        .iter()
        .any(|effect| effect.kind() == kind)
}

const fn seat_index(seat: Seat) -> usize {
    match seat {
        Seat::One => 0,
        Seat::Two => 1,
    }
}

const fn domain_seat(seat: Seat) -> crate::game::SeatId {
    match seat {
        Seat::One => crate::game::SeatId::One,
        Seat::Two => crate::game::SeatId::Two,
    }
}

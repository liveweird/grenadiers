use super::projection_capture_wire::domain_seat;
use super::projection_facts::{BoundPublication, OilPhase};
use super::{
    ActorView, EffectKind, EventIndex, GameServerMessage, GameState, OilBarrelState, OilBarrelView,
    ProjectionError, Seat, SnapshotEvents, WireEvent,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecipientProjection {
    pub state: GameState,
    pub events: SnapshotEvents,
}

pub fn project_recipient(
    publication: &BoundPublication,
    recipient: Seat,
) -> Result<RecipientProjection, ProjectionError> {
    validate_publication(publication, recipient)?;
    let recipient_actor = &publication.world.actors[recipient.index()].view;
    let hides_bombs = has_effect(recipient_actor, EffectKind::InvisibleBombs);
    let actors = publication
        .world
        .actors
        .iter()
        .map(|facts| {
            let actor = &facts.view;
            if actor.seat != recipient && has_effect(actor, EffectKind::InvisibilityCloak) {
                ActorView::Hidden {
                    id: actor.id,
                    seat: actor.seat,
                    alive: actor.alive,
                }
            } else {
                let mut visible = actor.clone();
                visible.voluntary_actions_frozen =
                    frozen_by_other(&publication.world.actors, actor.seat);
                visible.clone().into()
            }
        })
        .collect();
    let state = GameState {
        running_updates: publication.world.running_updates,
        remaining_ticks: publication.world.remaining_ticks,
        physical_tick: publication.world.physical_tick,
        arena: publication.world.arena.clone(),
        actors,
        bombs: if hides_bombs {
            Vec::new()
        } else {
            publication.world.bombs.clone()
        },
        flames: publication.world.flames.clone(),
        oil_barrels: publication
            .world
            .oil
            .iter()
            .filter_map(|oil| {
                let state = match oil.phase {
                    OilPhase::Unlit => OilBarrelState::Unlit,
                    OilPhase::Burning => OilBarrelState::Burning,
                    OilPhase::Concealed | OilPhase::Pending => return None,
                };
                Some(OilBarrelView {
                    cell: oil.cell,
                    state,
                })
            })
            .collect(),
        pickups: publication.world.pickups.clone(),
    };
    let events = publication
        .reports
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
                        kind: event.kind.clone(),
                    })
                })
        })
        .collect::<Result<Vec<_>, ProjectionError>>()?;
    Ok(RecipientProjection {
        state,
        events: SnapshotEvents::new(events).map_err(|_| ProjectionError::InvalidReports)?,
    })
}

pub fn prepare_snapshots(
    publication: &BoundPublication,
) -> Result<Vec<(Seat, String)>, ProjectionError> {
    let count = publication.world.actors.len();
    [Seat::One, Seat::Two, Seat::Three, Seat::Four]
        .into_iter()
        .take(count)
        .map(|recipient| {
            let projected = project_recipient(publication, recipient)?;
            let encoded = super::encode_server_message(&GameServerMessage::Snapshot {
                room: publication.room.clone(),
                state: projected.state,
                events: projected.events,
            })
            .map_err(|_| ProjectionError::EncodingFailed)?;
            Ok((recipient, encoded))
        })
        .collect()
}

fn validate_publication(
    publication: &BoundPublication,
    recipient: Seat,
) -> Result<(), ProjectionError> {
    let count = publication.room.configuration.domain().controllers().len();
    if recipient.index() >= count
        || publication.world.actors.len() != count
        || publication
            .world
            .actors
            .iter()
            .enumerate()
            .any(|(index, actor)| actor.view.seat.index() != index)
    {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    Ok(())
}

fn has_effect(actor: &super::VisibleActorView, kind: EffectKind) -> bool {
    actor
        .effects
        .as_slice()
        .iter()
        .any(|effect| effect.kind() == kind)
}

fn frozen_by_other(actors: &[super::projection_facts::ActorFacts], recipient: Seat) -> bool {
    actors[recipient.index()].view.alive
        && actors.iter().any(|actor| {
            actor.view.seat != recipient
                && actor.view.alive
                && has_effect(&actor.view, EffectKind::Stopwatch)
        })
}

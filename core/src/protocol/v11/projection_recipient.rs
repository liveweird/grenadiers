use crate::protocol::Seat;

use super::{
    BoundPublication, EffectKind, EventIndex, GameServerMessage, GameState, HiddenActorView,
    OilBarrelState, OilBarrelView, ProjectionError, SnapshotEvents, VisibleActorView, WireEvent,
};

pub fn prepare_snapshot_pair(
    publication: &BoundPublication,
) -> Result<[String; 2], ProjectionError> {
    let one = prepare(publication, Seat::One)?;
    let two = prepare(publication, Seat::Two)?;
    Ok([one, two])
}

fn prepare(publication: &BoundPublication, recipient: Seat) -> Result<String, ProjectionError> {
    let state = recipient_state(publication, recipient);
    let events = recipient_events(publication, recipient)?;
    let message = GameServerMessage::Snapshot {
        room: publication.room.clone(),
        state,
        events,
    };
    super::encode_server_message(&message).map_err(|_| ProjectionError::EncodingFailed)
}

pub(super) fn recipient_state(publication: &BoundPublication, recipient: Seat) -> GameState {
    let facts = &publication.world.0;
    let recipient_actor = &facts.actors[seat_index(recipient)];
    let hides_bombs = has_effect(recipient_actor, EffectKind::InvisibleBombs);
    let frozen = voluntary_actions_frozen(&facts.actors);
    let mut actor_index = 0;
    let actors = facts.actors.clone().map(|actor| {
        let current = actor_index;
        actor_index += 1;
        if actor.seat != recipient && has_effect(&actor, EffectKind::InvisibilityCloak) {
            HiddenActorView {
                id: actor.id,
                seat: actor.seat,
                alive: actor.alive,
            }
            .into()
        } else {
            visible_actor(actor, frozen[current], facts.last_teleport_tick).into()
        }
    });
    GameState {
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
        oil_barrels: facts
            .oil
            .iter()
            .filter_map(|oil| {
                let state = match oil.phase {
                    crate::protocol::v8::projection_facts::OilPhase::Unlit => OilBarrelState::Unlit,
                    crate::protocol::v8::projection_facts::OilPhase::Burning => {
                        OilBarrelState::Burning
                    }
                    crate::protocol::v8::projection_facts::OilPhase::Concealed
                    | crate::protocol::v8::projection_facts::OilPhase::Pending => return None,
                };
                Some(OilBarrelView {
                    cell: oil.cell,
                    state,
                })
            })
            .collect(),
        pickups: facts.pickups.clone(),
    }
}

pub(super) fn visible_actor(
    actor: crate::protocol::v8::VisibleActorView,
    voluntary_actions_frozen: bool,
    last_teleport_tick: Option<crate::protocol::Tick>,
) -> VisibleActorView {
    VisibleActorView {
        id: actor.id,
        seat: actor.seat,
        position: actor.position,
        heading: actor.heading,
        speed_level: actor.speed_level,
        alive: actor.alive,
        bomb_capacity: actor.bomb_capacity,
        bomb_range: actor.bomb_range,
        live_bomb_count: actor.live_bomb_count,
        inventory: actor.inventory,
        effects: actor.effects,
        damage_grace_remaining_ticks: actor.damage_grace_remaining_ticks,
        voluntary_actions_frozen,
        last_teleport_tick,
    }
}

fn voluntary_actions_frozen(actors: &[crate::protocol::v8::VisibleActorView; 2]) -> [bool; 2] {
    [
        actors[0].alive && actors[1].alive && has_effect(&actors[1], EffectKind::Stopwatch),
        actors[1].alive && actors[0].alive && has_effect(&actors[0], EffectKind::Stopwatch),
    ]
}

fn recipient_events(
    publication: &BoundPublication,
    recipient: Seat,
) -> Result<SnapshotEvents, ProjectionError> {
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
    SnapshotEvents::new(events).map_err(|_| ProjectionError::InvalidReports)
}

fn has_effect(actor: &crate::protocol::v8::VisibleActorView, kind: EffectKind) -> bool {
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

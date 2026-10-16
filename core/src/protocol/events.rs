use serde::Serialize;

use super::scalars::{EntityId, Seat, Tick};
use super::server::WireRoundResult;
use super::snapshot::BombView;

pub const MAX_SNAPSHOT_EVENTS: usize = 64;
pub const MAX_EVENT_INDEX: u8 = 63;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WireEvent {
    pub tick: Tick,
    pub index: u8,
    pub kind: EventKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum EventKind {
    BombPlaced { bomb: BombView },
    BombDetonated { bomb: BombView },
    ActorEliminated { id: EntityId, seat: Seat },
    RoundEnded { result: WireRoundResult },
}

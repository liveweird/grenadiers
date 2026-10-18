use super::damage::DamagePreventionSource;
use super::pickups::PickupKind;
use super::simulation::RoundResult;
use super::types::{CellId, EntityId, SeatId, Tick};
use super::view::BombView;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventAudience {
    Neither,
    One,
    Two,
    Both,
}

impl EventAudience {
    #[must_use]
    pub const fn includes(self, seat: SeatId) -> bool {
        matches!(
            (self, seat),
            (Self::One | Self::Both, SeatId::One) | (Self::Two | Self::Both, SeatId::Two)
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepEvent {
    pub tick: Tick,
    pub audience: EventAudience,
    pub kind: StepEventKind,
}

impl StepEvent {
    pub(crate) const fn new(tick: Tick, audience: EventAudience, kind: StepEventKind) -> Self {
        Self {
            tick,
            audience,
            kind,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StepEventKind {
    BombPlaced(BombView),
    BombDetonated(BombView),
    ActorEliminated {
        id: EntityId,
        seat: SeatId,
    },
    DamagePrevented {
        id: EntityId,
        seat: SeatId,
        source: DamagePreventionSource,
    },
    PickupCollected {
        id: EntityId,
        seat: SeatId,
        cell: CellId,
        kind: PickupKind,
    },
    RoundEnded(RoundResult),
}

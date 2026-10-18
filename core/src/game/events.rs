use super::damage::DamagePreventionSource;
use super::pickups::PickupKind;
use super::simulation::RoundResult;
use super::types::{CellId, EntityId, SeatId, Tick};
use super::view::BombView;
use super::{Bomb, TimedBombHistory};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TimedBombEvent {
    pub(crate) history: TimedBombHistory,
    pub(crate) detonates_at: Tick,
    pub(crate) paused: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventAudience {
    Neither,
    One,
    Two,
    Both,
}

impl EventAudience {
    #[must_use]
    pub const fn only(seat: SeatId) -> Self {
        match seat {
            SeatId::One => Self::One,
            SeatId::Two => Self::Two,
        }
    }

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
    pub(crate) timed_bomb: Option<TimedBombEvent>,
}

impl StepEvent {
    pub const fn new(tick: Tick, audience: EventAudience, kind: StepEventKind) -> Self {
        Self {
            tick,
            audience,
            kind,
            timed_bomb: None,
        }
    }

    pub(crate) const fn bomb(
        tick: Tick,
        audience: EventAudience,
        kind: StepEventKind,
        bomb: Bomb,
        paused: bool,
    ) -> Self {
        let timed_bomb = match (bomb.timed_history(), bomb.detonates_at()) {
            (Some(history), Some(detonates_at)) => Some(TimedBombEvent {
                history,
                detonates_at,
                paused,
            }),
            _ => None,
        };
        Self {
            tick,
            audience,
            kind,
            timed_bomb,
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

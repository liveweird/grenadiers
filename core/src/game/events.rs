use super::damage::DamagePreventionSource;
use super::pickups::PickupKind;
use super::simulation::RoundResult;
use super::types::{CellId, EntityId, PlayerCount, SeatId, Tick};
use super::view::BombView;
use super::{Bomb, BombMotion, TimedBombHistory};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TimedBombEvent {
    pub(crate) history: TimedBombHistory,
    pub(crate) detonates_at: Tick,
    pub(crate) paused: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventAudience(u8);

impl EventAudience {
    // These mixed-case names are a compatibility surface used by archived captures.
    #[allow(non_upper_case_globals)]
    pub const Neither: Self = Self(0);
    #[allow(non_upper_case_globals)]
    pub const One: Self = Self(1 << SeatId::One.index());
    #[allow(non_upper_case_globals)]
    pub const Two: Self = Self(1 << SeatId::Two.index());
    #[allow(non_upper_case_globals)]
    pub const Both: Self = Self(Self::One.0 | Self::Two.0);

    #[must_use]
    pub const fn all(count: PlayerCount) -> Self {
        Self((1_u8 << count.get()) - 1)
    }

    #[must_use]
    pub const fn only(seat: SeatId) -> Self {
        Self(1 << seat.index())
    }

    #[must_use]
    pub const fn includes(self, seat: SeatId) -> bool {
        self.0 & Self::only(seat).0 != 0
    }

    #[must_use]
    pub const fn without(self, seat: SeatId) -> Self {
        Self(self.0 & !Self::only(seat).0)
    }

    #[must_use]
    pub const fn is_subset_of(self, other: Self) -> bool {
        self.0 & !other.0 == 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepEvent {
    pub tick: Tick,
    pub audience: EventAudience,
    pub kind: StepEventKind,
    pub(crate) timed_bomb: Option<TimedBombEvent>,
    pub(crate) bomb_motion: Option<BombMotion>,
    pub(crate) bomb_has_been_kicked: bool,
    pub(crate) bomb_has_been_punched: bool,
}

impl StepEvent {
    pub const fn new(tick: Tick, audience: EventAudience, kind: StepEventKind) -> Self {
        Self {
            tick,
            audience,
            kind,
            timed_bomb: None,
            bomb_motion: None,
            bomb_has_been_kicked: false,
            bomb_has_been_punched: false,
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
            bomb_motion: bomb.motion(),
            bomb_has_been_kicked: bomb.has_been_kicked(),
            bomb_has_been_punched: bomb.has_been_punched(),
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

#[cfg(test)]
#[path = "events_tests.rs"]
mod tests;

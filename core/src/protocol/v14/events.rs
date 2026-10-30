use serde::Serialize;

use super::super::{Cell, EntityId, Tick};
use super::{BombView, PickupKind, RoundResult, Seat};

pub const MAX_SNAPSHOT_EVENTS: usize = 192;
pub const MAX_EVENT_INDEX: u8 = 191;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct EventIndex(u8);

impl EventIndex {
    pub const fn new(value: u8) -> Result<Self, super::super::InvalidScalar> {
        Self::new_for_rules(value, 0)
    }
    pub(crate) const fn new_for_rules(
        value: u8,
        rules_version: u8,
    ) -> Result<Self, super::super::InvalidScalar> {
        let maximum = if rules_version == 9 {
            215
        } else {
            MAX_EVENT_INDEX
        };
        if value <= maximum {
            Ok(Self(value))
        } else {
            Err(super::super::InvalidScalar)
        }
    }
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WireEvent {
    pub tick: Tick,
    pub index: EventIndex,
    pub kind: EventKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DamagePreventionSource {
    Heart,
    Shield,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum EventKind {
    BombPlaced {
        bomb: BombView,
    },
    BombDetonated {
        bomb: BombView,
    },
    ActorEliminated {
        id: EntityId,
        seat: Seat,
    },
    PickupCollected {
        id: EntityId,
        seat: Seat,
        cell: Cell,
        kind: PickupKind,
    },
    RoundEnded {
        result: RoundResult,
    },
    DamagePrevented {
        id: EntityId,
        seat: Seat,
        source: DamagePreventionSource,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct SnapshotEvents(Vec<WireEvent>);

impl SnapshotEvents {
    pub fn new(events: Vec<WireEvent>) -> Result<Self, super::super::InvalidScalar> {
        Self::new_for_rules(events, 0)
    }
    pub(crate) fn new_for_rules(
        events: Vec<WireEvent>,
        rules_version: u8,
    ) -> Result<Self, super::super::InvalidScalar> {
        if events.len() <= super::projection_limits::snapshot_events(rules_version) {
            Ok(Self(events))
        } else {
            Err(super::super::InvalidScalar)
        }
    }
    #[must_use]
    pub fn as_slice(&self) -> &[WireEvent] {
        &self.0
    }
}

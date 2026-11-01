use std::sync::Arc;

#[cfg(test)]
use grenadiers_core::room::RoomSettings;
use grenadiers_core::{
    game::{Direction, SeatId},
    protocol::{RoomCode, RoomId, v14, v15, v16, v17, v18, v19},
    room::RoomConfiguration,
};

use super::seat_wire::{
    domain_seat, domain_seat_v15, domain_seat_v16, domain_seat_v17, domain_seat_v18,
    domain_seat_v19,
};
pub(super) use super::seat_wire::{
    wire_seat, wire_seat_v16, wire_seat_v17, wire_seat_v18, wire_seat_v19,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) struct RosterIdentity {
    pub(in crate::game::rooms) id: RoomId,
    pub(in crate::game::rooms) code: RoomCode,
    pub(in crate::game::rooms) registry_serial: u64,
    pub(in crate::game::rooms) configuration: RoomConfiguration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RosterAction {
    Ready {
        ready: bool,
        settings_revision: u32,
    },
    #[cfg(test)]
    UpdateSettings {
        settings_revision: u32,
        settings: RoomSettings,
    },
    Movement(Option<Direction>),
    Bomb(u32),
    Equipment(u32),
    Rematch(bool),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) struct RosterSeatFrames(Vec<(SeatId, Arc<str>)>);

impl RosterSeatFrames {
    pub(super) fn new(frames: Vec<(v14::Seat, String)>) -> Option<Self> {
        frames
            .into_iter()
            .map(|(seat, frame)| Some((domain_seat(seat)?, Arc::from(frame))))
            .collect::<Option<Vec<_>>>()
            .map(Self)
    }

    pub(super) fn new_v15(frames: Vec<(v15::Seat, String)>) -> Option<Self> {
        frames
            .into_iter()
            .map(|(seat, frame)| Some((domain_seat_v15(seat)?, Arc::from(frame))))
            .collect::<Option<Vec<_>>>()
            .map(Self)
    }

    pub(super) fn new_v16(frames: Vec<(v16::Seat, String)>) -> Option<Self> {
        frames
            .into_iter()
            .map(|(seat, frame)| Some((domain_seat_v16(seat)?, Arc::from(frame))))
            .collect::<Option<Vec<_>>>()
            .map(Self)
    }

    pub(super) fn new_v17(frames: Vec<(v17::Seat, String)>) -> Option<Self> {
        frames
            .into_iter()
            .map(|(seat, frame)| Some((domain_seat_v17(seat)?, Arc::from(frame))))
            .collect::<Option<Vec<_>>>()
            .map(Self)
    }

    pub(super) fn new_v18(frames: Vec<(v18::Seat, String)>) -> Option<Self> {
        frames
            .into_iter()
            .map(|(seat, frame)| Some((domain_seat_v18(seat)?, Arc::from(frame))))
            .collect::<Option<Vec<_>>>()
            .map(Self)
    }

    pub(super) fn new_v19(frames: Vec<(v19::Seat, String)>) -> Option<Self> {
        frames
            .into_iter()
            .map(|(seat, frame)| Some((domain_seat_v19(seat)?, Arc::from(frame))))
            .collect::<Option<Vec<_>>>()
            .map(Self)
    }

    pub(in crate::game::rooms) fn select(&self, seat: SeatId) -> Option<Arc<str>> {
        self.0
            .iter()
            .find(|(candidate, _)| *candidate == seat)
            .map(|(_, frame)| Arc::clone(frame))
    }
}

#[derive(Clone)]
pub(crate) enum RosterPublication {
    Legacy(v14::RoomView, Option<Arc<v14::BoundPublication>>),
    Environment(v15::RoomView, Option<Arc<v15::BoundPublication>>),
    Lairs(v16::RoomView, Option<Arc<v16::BoundPublication>>),
    Respawn(v17::RoomView, Option<Arc<v17::BoundPublication>>),
    Spike(v18::RoomView, Option<Arc<v18::BoundPublication>>),
    SpikeV19(v19::RoomView, Option<Arc<v19::BoundPublication>>),
}

#[derive(Clone)]
pub(crate) struct RosterNotice {
    pub(in crate::game::rooms) publication: RosterPublication,
    pub(in crate::game::rooms) frames: Option<RosterSeatFrames>,
}

impl RosterNotice {
    pub(in crate::game::rooms) fn frame(&self, seat: SeatId) -> Option<Arc<str>> {
        self.frames.as_ref()?.select(seat)
    }

    pub(in crate::game::rooms) const fn legacy_room(&self) -> Option<&v14::RoomView> {
        match &self.publication {
            RosterPublication::Legacy(room, _) => Some(room),
            RosterPublication::Environment(_, _)
            | RosterPublication::Lairs(_, _)
            | RosterPublication::Respawn(_, _)
            | RosterPublication::Spike(_, _)
            | RosterPublication::SpikeV19(_, _) => None,
        }
    }

    pub(in crate::game::rooms) fn legacy_cpu_publication(&self) -> Option<&v14::BoundPublication> {
        match &self.publication {
            RosterPublication::Legacy(_, publication) => publication.as_deref(),
            RosterPublication::Environment(_, _)
            | RosterPublication::Lairs(_, _)
            | RosterPublication::Respawn(_, _)
            | RosterPublication::Spike(_, _)
            | RosterPublication::SpikeV19(_, _) => None,
        }
    }

    pub(in crate::game::rooms) const fn environment_room(&self) -> Option<&v15::RoomView> {
        match &self.publication {
            RosterPublication::Environment(room, _) => Some(room),
            RosterPublication::Legacy(_, _) => None,
            RosterPublication::Lairs(_, _)
            | RosterPublication::Respawn(_, _)
            | RosterPublication::Spike(_, _)
            | RosterPublication::SpikeV19(_, _) => None,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::game::rooms) const fn lair_room(&self) -> Option<&v16::RoomView> {
        match &self.publication {
            RosterPublication::Lairs(room, _) => Some(room),
            RosterPublication::Legacy(_, _)
            | RosterPublication::Environment(_, _)
            | RosterPublication::Respawn(_, _)
            | RosterPublication::Spike(_, _)
            | RosterPublication::SpikeV19(_, _) => None,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::game::rooms) fn lair_cpu_publication(&self) -> Option<&v16::BoundPublication> {
        match &self.publication {
            RosterPublication::Lairs(_, publication) => publication.as_deref(),
            RosterPublication::Legacy(_, _)
            | RosterPublication::Environment(_, _)
            | RosterPublication::Respawn(_, _)
            | RosterPublication::Spike(_, _)
            | RosterPublication::SpikeV19(_, _) => None,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::game::rooms) const fn respawn_room(&self) -> Option<&v17::RoomView> {
        match &self.publication {
            RosterPublication::Respawn(room, _) => Some(room),
            RosterPublication::Legacy(_, _)
            | RosterPublication::Environment(_, _)
            | RosterPublication::Lairs(_, _)
            | RosterPublication::Spike(_, _)
            | RosterPublication::SpikeV19(_, _) => None,
        }
    }

    pub(in crate::game::rooms) const fn spike_room(&self) -> Option<&v18::RoomView> {
        match &self.publication {
            RosterPublication::Spike(room, _) => Some(room),
            _ => None,
        }
    }

    pub(in crate::game::rooms) fn spike_cpu_publication(&self) -> Option<&v18::BoundPublication> {
        match &self.publication {
            RosterPublication::Spike(_, publication) => publication.as_deref(),
            _ => None,
        }
    }

    pub(in crate::game::rooms) const fn spike_v19_room(&self) -> Option<&v19::RoomView> {
        match &self.publication {
            RosterPublication::SpikeV19(room, _) => Some(room),
            _ => None,
        }
    }

    pub(in crate::game::rooms) fn spike_v19_cpu_publication(
        &self,
    ) -> Option<&v19::BoundPublication> {
        match &self.publication {
            RosterPublication::SpikeV19(_, publication) => publication.as_deref(),
            _ => None,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::game::rooms) fn respawn_cpu_publication(&self) -> Option<&v17::BoundPublication> {
        match &self.publication {
            RosterPublication::Respawn(_, publication) => publication.as_deref(),
            RosterPublication::Legacy(_, _)
            | RosterPublication::Environment(_, _)
            | RosterPublication::Lairs(_, _)
            | RosterPublication::Spike(_, _)
            | RosterPublication::SpikeV19(_, _) => None,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::game::rooms) fn environment_cpu_publication(
        &self,
    ) -> Option<&v15::BoundPublication> {
        match &self.publication {
            RosterPublication::Environment(_, publication) => publication.as_deref(),
            RosterPublication::Legacy(_, _)
            | RosterPublication::Lairs(_, _)
            | RosterPublication::Respawn(_, _)
            | RosterPublication::Spike(_, _)
            | RosterPublication::SpikeV19(_, _) => None,
        }
    }
}

impl std::fmt::Debug for RosterNotice {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RosterNotice")
            .field("environment", &self.environment_room().is_some())
            .field("frames", &self.frames)
            .finish_non_exhaustive()
    }
}

impl PartialEq for RosterNotice {
    fn eq(&self, other: &Self) -> bool {
        self.frames == other.frames
            && match (&self.publication, &other.publication) {
                (
                    RosterPublication::Legacy(a_room, a_cpu),
                    RosterPublication::Legacy(b_room, b_cpu),
                ) => a_room == b_room && a_cpu == b_cpu,
                (
                    RosterPublication::Environment(a_room, a_cpu),
                    RosterPublication::Environment(b_room, b_cpu),
                ) => {
                    a_room == b_room
                        && match (a_cpu, b_cpu) {
                            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                            (None, None) => true,
                            _ => false,
                        }
                }
                (
                    RosterPublication::Lairs(a_room, a_cpu),
                    RosterPublication::Lairs(b_room, b_cpu),
                ) => {
                    a_room == b_room
                        && match (a_cpu, b_cpu) {
                            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                            (None, None) => true,
                            _ => false,
                        }
                }
                (
                    RosterPublication::Respawn(a_room, a_cpu),
                    RosterPublication::Respawn(b_room, b_cpu),
                ) => {
                    a_room == b_room
                        && match (a_cpu, b_cpu) {
                            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                            (None, None) => true,
                            _ => false,
                        }
                }
                (
                    RosterPublication::Spike(a_room, a_cpu),
                    RosterPublication::Spike(b_room, b_cpu),
                ) => {
                    a_room == b_room
                        && match (a_cpu, b_cpu) {
                            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                            (None, None) => true,
                            _ => false,
                        }
                }
                (
                    RosterPublication::SpikeV19(a_room, a_cpu),
                    RosterPublication::SpikeV19(b_room, b_cpu),
                ) => {
                    a_room == b_room
                        && match (a_cpu, b_cpu) {
                            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                            (None, None) => true,
                            _ => false,
                        }
                }
                _ => false,
            }
    }
}

impl Eq for RosterNotice {}

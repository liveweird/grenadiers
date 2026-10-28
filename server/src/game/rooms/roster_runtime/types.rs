use std::sync::Arc;

use grenadiers_core::{
    game::{Direction, SeatId},
    protocol::{RoomCode, RoomId, v14},
    room::{RoomConfiguration, RoomSettings},
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

    pub(in crate::game::rooms) fn select(&self, seat: SeatId) -> Option<Arc<str>> {
        self.0
            .iter()
            .find(|(candidate, _)| *candidate == seat)
            .map(|(_, frame)| Arc::clone(frame))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RosterNotice {
    pub(in crate::game::rooms) room: v14::RoomView,
    pub(in crate::game::rooms) frames: Option<RosterSeatFrames>,
    pub(in crate::game::rooms) cpu_publication: Option<v14::BoundPublication>,
}

impl RosterNotice {
    pub(in crate::game::rooms) fn frame(&self, seat: SeatId) -> Option<Arc<str>> {
        self.frames.as_ref()?.select(seat)
    }
}

pub(super) const fn wire_seat(seat: SeatId) -> v14::Seat {
    match seat {
        SeatId::One => v14::Seat::One,
        SeatId::Two => v14::Seat::Two,
        SeatId::Three => v14::Seat::Three,
        SeatId::Four => v14::Seat::Four,
    }
}

const fn domain_seat(seat: v14::Seat) -> Option<SeatId> {
    Some(match seat {
        v14::Seat::One => SeatId::One,
        v14::Seat::Two => SeatId::Two,
        v14::Seat::Three => SeatId::Three,
        v14::Seat::Four => SeatId::Four,
    })
}

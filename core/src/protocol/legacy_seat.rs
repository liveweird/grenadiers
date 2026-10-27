use std::fmt;

use crate::game::SeatId;

use super::Seat;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnsupportedLegacySeat(pub SeatId);

impl fmt::Display for UnsupportedLegacySeat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("seat is not supported by this archived protocol")
    }
}

impl std::error::Error for UnsupportedLegacySeat {}

impl TryFrom<SeatId> for Seat {
    type Error = UnsupportedLegacySeat;

    fn try_from(seat: SeatId) -> Result<Self, Self::Error> {
        match seat {
            SeatId::One => Ok(Self::One),
            SeatId::Two => Ok(Self::Two),
            SeatId::Three | SeatId::Four => Err(UnsupportedLegacySeat(seat)),
        }
    }
}

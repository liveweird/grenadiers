use std::sync::Arc;

use grenadiers_core::game::SeatId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SeatFrames([Arc<str>; 2]);

impl SeatFrames {
    pub(super) fn new(frames: [String; 2]) -> Self {
        Self(frames.map(Arc::from))
    }

    pub(crate) fn select(&self, seat: SeatId) -> Arc<str> {
        Arc::clone(&self.0[seat.index()])
    }
}

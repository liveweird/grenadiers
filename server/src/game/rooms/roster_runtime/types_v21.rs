use grenadiers_core::protocol::v21;

use super::types::{RosterNotice, RosterPublication};

impl RosterNotice {
    pub(in crate::game::rooms) fn spike_v21_cpu_publication(
        &self,
    ) -> Option<&v21::BoundPublication> {
        match &self.publication {
            RosterPublication::SpikeV21(_, publication) => publication.as_deref(),
            _ => None,
        }
    }

    pub(in crate::game::rooms) const fn spike_v21_room(&self) -> Option<&v21::RoomView> {
        match &self.publication {
            RosterPublication::SpikeV21(room, _) => Some(room),
            _ => None,
        }
    }
}

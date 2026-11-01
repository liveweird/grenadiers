use super::{admission::PreparedRosterAdmission, runtime::RosterRuntime, types::RosterPublication};

impl RosterRuntime {
    pub(in crate::game::rooms) fn commit_admission(&mut self, prepared: PreparedRosterAdmission) {
        if prepared.is_replacement {
            self.stop_seat_inputs(prepared.seat);
        }
        self.cpu_publication = prepared.notice.legacy_cpu_publication().cloned();
        self.environment_cpu_publication = match &prepared.notice.publication {
            RosterPublication::Environment(_, publication) => publication.clone(),
            _ => None,
        };
        self.lair_cpu_publication = match &prepared.notice.publication {
            RosterPublication::Lairs(_, publication) => publication.clone(),
            _ => None,
        };
        self.respawn_cpu_publication = match &prepared.notice.publication {
            RosterPublication::Respawn(_, publication) => publication.clone(),
            _ => None,
        };
        self.spike_cpu_publication = match &prepared.notice.publication {
            RosterPublication::Spike(_, publication) => publication.clone(),
            _ => None,
        };
        self.spike_v19_cpu_publication = match &prepared.notice.publication {
            RosterPublication::SpikeV19(_, publication) => publication.clone(),
            _ => None,
        };
        self.lifecycle = prepared.lifecycle;
        self.revision = prepared.revision;
    }
}

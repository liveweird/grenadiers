use grenadiers_core::room::{MatchChange, MatchPhase};

use super::{
    error::RosterRuntimeError,
    publication, publication_v15,
    runtime::RosterRuntime,
    types::{RosterNotice, RosterPublication},
};

impl RosterRuntime {
    pub(super) fn publish_current(&mut self) -> Result<RosterNotice, RosterRuntimeError> {
        let next = self.next_revision()?;
        let body = self.sim.as_ref().filter(|_| {
            matches!(
                self.lifecycle.phase(),
                MatchPhase::Running | MatchPhase::Result { .. } | MatchPhase::MatchResult { .. }
            )
        });
        let notice = if let Some(environment) = self.environment {
            let room = publication_v15::room_view(
                &self.identity,
                &self.lifecycle,
                environment,
                next,
                self.settings_revision,
            )?;
            if let Some(sim) = body {
                publication_v15::snapshot(
                    room,
                    sim.epoch,
                    &sim.state,
                    &self.reports,
                    sim.generation,
                )?
            } else {
                publication_v15::view_notice(room)
            }
        } else {
            let room = publication::room_view(
                &self.identity,
                &self.lifecycle,
                next,
                self.settings_revision,
            )?;
            if let Some(sim) = body {
                publication::snapshot(room, sim.epoch, &sim.state, &self.reports, sim.generation)?
            } else {
                publication::view_notice(room)
            }
        };
        self.cpu_publication = notice.legacy_cpu_publication().cloned();
        self.environment_cpu_publication = match &notice.publication {
            RosterPublication::Environment(_, publication) => publication.clone(),
            RosterPublication::Legacy(_, _) => None,
        };
        self.revision = next;
        self.reports.clear();
        Ok(notice)
    }

    pub(super) fn publish_view(&mut self) -> Result<RosterNotice, RosterRuntimeError> {
        let next = self.next_revision()?;
        let notice = if let Some(environment) = self.environment {
            publication_v15::view_notice(publication_v15::room_view(
                &self.identity,
                &self.lifecycle,
                environment,
                next,
                self.settings_revision,
            )?)
        } else {
            publication::view_notice(publication::room_view(
                &self.identity,
                &self.lifecycle,
                next,
                self.settings_revision,
            )?)
        };
        self.revision = next;
        self.cpu_publication = None;
        self.environment_cpu_publication = None;
        Ok(notice)
    }

    pub(super) fn notice_for(
        &mut self,
        change: MatchChange,
        include_state: bool,
    ) -> Result<Option<RosterNotice>, RosterRuntimeError> {
        if change == MatchChange::Unchanged {
            Ok(None)
        } else if include_state {
            self.publish_current().map(Some)
        } else {
            self.publish_view().map(Some)
        }
    }
}

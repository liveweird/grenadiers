use grenadiers_core::room::{MatchChange, MatchPhase};

use super::{InternalWireProfile, MatchNotice, MatchRuntime, MatchRuntimeError};
use crate::game::rooms::match_runtime::{SeatFrames, publication};

impl MatchRuntime {
    pub(super) fn notice_for_change(
        &mut self,
        change: MatchChange,
        include_state: bool,
    ) -> Result<Option<MatchNotice>, MatchRuntimeError> {
        if change == MatchChange::Unchanged {
            Ok(None)
        } else if include_state && self.sim.is_some() {
            self.publish_snapshot().map(Some)
        } else {
            self.publish_view().map(Some)
        }
    }

    pub(super) fn publish_current(&mut self) -> Result<MatchNotice, MatchRuntimeError> {
        if self.sim.is_some()
            && matches!(
                self.lifecycle.phase(),
                MatchPhase::Running | MatchPhase::Result { .. } | MatchPhase::MatchResult { .. }
            )
        {
            self.publish_snapshot()
        } else {
            self.publish_view()
        }
    }

    pub(super) fn publish_view(&mut self) -> Result<MatchNotice, MatchRuntimeError> {
        let next = self.next_revision()?;
        let room = publication::room_view(
            &self.identity,
            &self.lifecycle,
            next,
            self.settings_revision,
            self.settings_editor,
        )?;
        let notice = self.view_notice(room)?;
        self.revision = next;
        Ok(notice)
    }

    fn publish_snapshot(&mut self) -> Result<MatchNotice, MatchRuntimeError> {
        let next = self.next_revision()?;
        let sim = self.sim.as_ref().ok_or(MatchRuntimeError::Publication)?;
        let notice = match self.wire_profile {
            InternalWireProfile::Legacy13 => publication::snapshot(
                &self.identity,
                &self.lifecycle,
                next,
                self.settings_revision,
                self.settings_editor,
                sim,
                &self.reports,
            )?,
            InternalWireProfile::Live14 => {
                let room = publication::room_view(
                    &self.identity,
                    &self.lifecycle,
                    next,
                    self.settings_revision,
                    self.settings_editor,
                )?;
                let frames = crate::game::rooms::legacy_v14::snapshot(
                    &room,
                    publication::wire_counter(sim.epoch)?,
                    &sim.state,
                    &self.reports,
                    sim.generation_record,
                )
                .map_err(|_| MatchRuntimeError::Publication)?;
                let cpu_observation =
                    publication::cpu_observation(&self.identity, &room, sim, &self.reports)?;
                MatchNotice {
                    room,
                    frames: Some(SeatFrames::new(frames)),
                    cpu_observation,
                }
            }
        };
        self.revision = next;
        if !self.reports.is_empty() {
            self.capture(
                self.lifecycle.tick(),
                crate::game::rooms::replay::Operation::ReportFlush,
            );
        }
        self.reports.clear();
        Ok(notice)
    }
}

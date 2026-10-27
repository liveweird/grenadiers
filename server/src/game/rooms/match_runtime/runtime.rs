use grenadiers_core::{
    game::{Direction, GenerationRecord, SeatId, State, StepReport, Tick},
    protocol::v13::RoomView,
    room::{
        DepartureReason, InterruptReason, MatchChange, MatchLifecycle, MatchPhase, RoomSettings,
    },
};

use super::super::{input::InputMailbox, types::RoomIdentity};
use super::SeatFrames;
use super::error::{MatchRuntimeError, map_lifecycle_error};
use super::publication;
use crate::round_seed::RoundSeedSource;

pub(super) mod admission;
mod capture;
mod input_action;
mod maintenance;
mod owner_helpers;
#[cfg(test)]
pub(super) mod replay;
mod settings;

const REPORTS_PER_SNAPSHOT: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MatchAction {
    #[cfg_attr(not(test), allow(dead_code))]
    Ready(bool),
    ReadyChecked {
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
pub(crate) struct MatchNotice {
    pub(crate) room: RoomView,
    pub(super) frames: Option<SeatFrames>,
    pub(in crate::game::rooms) cpu_observation: Option<grenadiers_core::cpu::Observation>,
}

impl MatchNotice {
    pub(crate) fn frame(&self, seat: SeatId) -> Option<Arc<str>> {
        self.frames.as_ref().map(|frames| frames.select(seat))
    }

    #[cfg(test)]
    pub(crate) fn replace_frames_for_test(&mut self, frames: [&str; 2]) {
        self.frames = Some(SeatFrames::new(frames.map(str::to_owned)));
    }
}

pub(in crate::game::rooms) struct MatchRuntime {
    identity: RoomIdentity,
    lifecycle: MatchLifecycle,
    revision: u32,
    settings_revision: u32,
    settings_editor: Option<SeatId>,
    round_seed_source: RoundSeedSource,
    inputs: InputMailbox,
    sim: Option<SimRound>,
    reports: Vec<StepReport>,
    journal: Option<super::super::replay::Journal>,
}

pub(in crate::game::rooms) struct SimRound {
    pub(in crate::game::rooms) epoch: u64,
    pub(in crate::game::rooms) state: State,
    pub(in crate::game::rooms) generation_record: Option<GenerationRecord>,
}

pub(in crate::game::rooms) const fn cpu_seat_for_mode(
    mode: grenadiers_core::room::RoomMode,
) -> Option<SeatId> {
    match mode {
        grenadiers_core::room::RoomMode::Solo(_) => Some(SeatId::Two),
        grenadiers_core::room::RoomMode::HumanDuel => None,
    }
}

impl MatchRuntime {
    pub(in crate::game::rooms) fn new(
        identity: RoomIdentity,
        round_seed_source: RoundSeedSource,
    ) -> Self {
        let settings = identity.profile.initial_settings();
        let settings_editor = identity.profile.initial_editor();
        Self::with_configuration(
            identity,
            round_seed_source,
            None,
            settings,
            1,
            settings_editor,
        )
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn occupy(
        &mut self,
        seat: SeatId,
    ) -> Result<MatchNotice, MatchRuntimeError> {
        let tick = self.lifecycle.tick();
        self.ensure_revision()?;
        self.lifecycle.occupy(seat).map_err(map_lifecycle_error)?;
        let notice = self.publish_view()?;
        self.capture(tick, super::super::replay::Operation::Occupy(seat));
        Ok(notice)
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn tick(
        &mut self,
    ) -> Result<Option<MatchNotice>, MatchRuntimeError> {
        self.settle_boundary([None; 2])
    }

    pub(in crate::game::rooms) fn settle_boundary(
        &mut self,
        departures: [Option<DepartureReason>; 2],
    ) -> Result<Option<MatchNotice>, MatchRuntimeError> {
        let tick = self.lifecycle.tick();
        let previous_phase = self.lifecycle.phase();
        self.ensure_revision()?;
        let has_departures = departures.iter().any(Option::is_some);
        let mut publish = false;
        if previous_phase == MatchPhase::Running {
            publish |= self.running_tick()?;
            self.capture_tick(tick, previous_phase);
            publish |= self.apply_grace_departures(departures)?;
        } else if !matches!(previous_phase, MatchPhase::Interrupted { .. }) {
            publish |= self.apply_grace_departures(departures)?;
            publish |= self.idle_tick(previous_phase)?;
            self.capture_tick(tick, previous_phase);
        } else {
            return Ok(None);
        }
        let expiry = self
            .lifecycle
            .expire_reconnect_grace()
            .map_err(map_lifecycle_error)?;
        self.release_expired_editor();
        self.capture(
            self.lifecycle.tick(),
            super::super::replay::Operation::Expiry,
        );
        publish |= expiry != MatchChange::Unchanged || has_departures;
        if publish {
            self.publish_current().map(Some)
        } else {
            Ok(None)
        }
    }

    fn running_tick(&mut self) -> Result<bool, MatchRuntimeError> {
        let next_tick = self
            .lifecycle
            .tick()
            .0
            .checked_add(1)
            .map(Tick)
            .ok_or(MatchRuntimeError::CapacityReached)?;
        let inputs = self.inputs.take(next_tick);
        let sim = self.sim.as_mut().ok_or(MatchRuntimeError::Publication)?;
        let report = sim
            .state
            .step(&inputs)
            .map_err(|_| MatchRuntimeError::CapacityReached)?;
        let result = sim.state.result();
        let change = self
            .lifecycle
            .complete_running_tick(result)
            .map_err(map_lifecycle_error)?;
        if report.tick != self.lifecycle.tick() || sim.state.tick() != self.lifecycle.tick() {
            return Err(MatchRuntimeError::Publication);
        }
        self.reports.push(report);
        if result.is_some() {
            self.inputs.clear_pending();
            return Ok(true);
        }
        if self.reports.len() == REPORTS_PER_SNAPSHOT {
            Ok(true)
        } else if matches!(change, MatchChange::Changed) {
            Ok(false)
        } else {
            Err(MatchRuntimeError::Publication)
        }
    }

    fn idle_tick(&mut self, previous_phase: MatchPhase) -> Result<bool, MatchRuntimeError> {
        let change = self
            .lifecycle
            .advance_idle_tick()
            .map_err(map_lifecycle_error)?;
        match change {
            MatchChange::RoundStarted(assignment) => {
                self.start_round(assignment)?;
                Ok(true)
            }
            MatchChange::CountdownStarted
                if matches!(previous_phase, MatchPhase::Result { .. }) =>
            {
                self.inputs.clear_pending();
                Ok(true)
            }
            _ if self.lifecycle.tick().0.is_multiple_of(3)
                && matches!(
                    self.lifecycle.phase(),
                    MatchPhase::Countdown { .. }
                        | MatchPhase::Result { .. }
                        | MatchPhase::MatchResult { .. }
                ) =>
            {
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn start_round(
        &mut self,
        assignment: grenadiers_core::game::RoundAssignment,
    ) -> Result<(), MatchRuntimeError> {
        if !self.reports.is_empty() {
            return self.fail_round_construction();
        }
        let Some(initial) = self.identity.profile.initial_round_with_settings(
            self.lifecycle.settings(),
            self.lifecycle.tick(),
            assignment,
            &self.round_seed_source,
        ) else {
            return self.fail_round_construction();
        };
        if initial.state.tick() != self.lifecycle.tick()
            || initial.state.running_updates() != 0
            || initial.state.round_assignment() != assignment
        {
            return self.fail_round_construction();
        }
        self.inputs.reset_round();
        self.sim = Some(SimRound {
            epoch: self.lifecycle.round_attempt(),
            state: initial.state,
            generation_record: initial.generation_record,
        });
        Ok(())
    }

    fn fail_round_construction(&mut self) -> Result<(), MatchRuntimeError> {
        self.sim = None;
        self.reports.clear();
        self.inputs.clear_pending();
        self.lifecycle
            .interrupt(InterruptReason::InternalError)
            .map_err(map_lifecycle_error)?;
        Ok(())
    }

    fn apply_grace_departures(
        &mut self,
        departures: [Option<DepartureReason>; 2],
    ) -> Result<bool, MatchRuntimeError> {
        if departures.iter().all(Option::is_none) {
            return Ok(false);
        }
        let change = self
            .lifecycle
            .depart_with_grace(departures)
            .map_err(map_lifecycle_error)?;
        self.release_departed_editor(departures);
        for (seat, departure) in SeatId::ALL.into_iter().zip(departures) {
            if departure.is_some() {
                self.inputs.stop(seat);
            }
        }
        self.capture(
            self.lifecycle.tick(),
            super::super::replay::Operation::GraceDepart(departures),
        );
        Ok(change != MatchChange::Unchanged)
    }

    fn validate_actor(&self, seat: SeatId, expected_epoch: u64) -> Result<(), MatchRuntimeError> {
        if expected_epoch != self.lifecycle.round_attempt() {
            return Err(MatchRuntimeError::StaleEpoch);
        }
        if !self.lifecycle.seats()[seat.index()].occupied() {
            return Err(MatchRuntimeError::VacantSeat);
        }
        Ok(())
    }

    fn require_running(&self) -> Result<(), MatchRuntimeError> {
        (self.lifecycle.phase() == MatchPhase::Running)
            .then_some(())
            .ok_or(MatchRuntimeError::InvalidPhase)
    }

    fn notice_for_change(
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

    fn publish_current(&mut self) -> Result<MatchNotice, MatchRuntimeError> {
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

    fn publish_view(&mut self) -> Result<MatchNotice, MatchRuntimeError> {
        let next = self.next_revision()?;
        let room = publication::room_view(
            &self.identity,
            &self.lifecycle,
            next,
            self.settings_revision,
            self.settings_editor,
        )?;
        self.revision = next;
        Ok(MatchNotice {
            room,
            frames: None,
            cpu_observation: None,
        })
    }

    fn publish_snapshot(&mut self) -> Result<MatchNotice, MatchRuntimeError> {
        let next = self.next_revision()?;
        let sim = self.sim.as_ref().ok_or(MatchRuntimeError::Publication)?;
        let notice = publication::snapshot(
            &self.identity,
            &self.lifecycle,
            next,
            self.settings_revision,
            self.settings_editor,
            sim,
            &self.reports,
        )?;
        self.revision = next;
        if !self.reports.is_empty() {
            self.capture(
                self.lifecycle.tick(),
                super::super::replay::Operation::ReportFlush,
            );
        }
        self.reports.clear();
        Ok(notice)
    }
}
use std::sync::Arc;

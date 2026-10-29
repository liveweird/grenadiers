use std::sync::Arc;

use grenadiers_core::{
    game::{GenerationRecord, SeatId, SimulationSeed, State, StepReport, StormCloudConfig, Tick},
    protocol::{v14, v15, v16},
    room::{
        DepartureReason, MatchChange, MatchPhase, RoomSettings, RosterMatchLifecycle,
        RosterParticipant,
    },
};

use crate::{game::rooms::input::InputMailbox, round_seed::RoundSeedSource};

use super::{
    error::{RosterRuntimeError, input, lifecycle},
    publication,
    round::mark_eliminated,
    types::{RosterAction, RosterIdentity, RosterNotice},
};

pub(super) type SimulationSeedDerivation = dyn Fn(u64) -> Result<SimulationSeed, ()> + Send + Sync;

#[derive(Clone)]
pub(in crate::game::rooms) struct RosterRuntime {
    pub(super) identity: RosterIdentity,
    pub(super) lifecycle: RosterMatchLifecycle,
    pub(super) revision: u32,
    pub(super) settings_revision: u32,
    pub(super) settings_locked: bool,
    pub(super) creator_settings_revoked: bool,
    pub(super) round_seed_source: RoundSeedSource,
    pub(super) simulation_seed: Arc<SimulationSeedDerivation>,
    pub(super) inputs: InputMailbox,
    pub(super) sim: Option<SimRound>,
    pub(super) reports: Vec<StepReport>,
    pub(super) cpu_publication: Option<v14::BoundPublication>,
    pub(super) environment_cpu_publication: Option<Arc<v15::BoundPublication>>,
    pub(super) lair_cpu_publication: Option<Arc<v16::BoundPublication>>,
    pub(super) environment_profile: Option<EnvironmentProfile>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EnvironmentProfile {
    StormV7(StormCloudConfig),
    LairsV8 {
        cloud: StormCloudConfig,
        tuning: grenadiers_core::room::LairTuning,
    },
}

impl EnvironmentProfile {
    pub(super) const fn cloud(self) -> StormCloudConfig {
        match self {
            Self::StormV7(cloud) | Self::LairsV8 { cloud, .. } => cloud,
        }
    }
}

#[derive(Clone)]
pub(super) struct SimRound {
    pub(super) epoch: u64,
    pub(super) state: State,
    pub(super) generation: GenerationRecord,
}

impl RosterRuntime {
    pub(in crate::game::rooms) fn new(
        identity: RosterIdentity,
        settings: RoomSettings,
        round_seed_source: RoundSeedSource,
        simulation_seed: Arc<SimulationSeedDerivation>,
    ) -> Result<Self, RosterRuntimeError> {
        Ok(Self {
            lifecycle: RosterMatchLifecycle::new(identity.configuration, settings),
            inputs: InputMailbox::for_configuration(identity.configuration),
            identity,
            revision: 1,
            settings_revision: 1,
            settings_locked: false,
            creator_settings_revoked: false,
            round_seed_source,
            simulation_seed,
            sim: None,
            reports: Vec::new(),
            cpu_publication: None,
            environment_cpu_publication: None,
            lair_cpu_publication: None,
            environment_profile: None,
        })
    }

    pub(in crate::game::rooms) fn action(
        &mut self,
        seat: SeatId,
        expected_epoch: u64,
        action: RosterAction,
    ) -> Result<Option<RosterNotice>, RosterRuntimeError> {
        if self.identity.configuration.controller(seat)
            != Some(grenadiers_core::room::SeatControl::Human)
        {
            return Err(RosterRuntimeError::InvalidPhase);
        }
        let mut next = self.clone();
        let notice = next.apply_action(seat, expected_epoch, action)?;
        *self = next;
        Ok(notice)
    }

    pub(super) fn apply_action(
        &mut self,
        seat: SeatId,
        expected_epoch: u64,
        action: RosterAction,
    ) -> Result<Option<RosterNotice>, RosterRuntimeError> {
        self.validate_actor(seat, expected_epoch)?;
        match action {
            RosterAction::Ready {
                ready,
                settings_revision,
            } => {
                if ready && settings_revision != self.settings_revision {
                    return Err(RosterRuntimeError::StaleSettings);
                }
                let change = self.lifecycle.set_ready(seat, ready).map_err(lifecycle)?;
                self.notice_for(change.change(), false)
            }
            #[cfg(test)]
            RosterAction::UpdateSettings {
                settings_revision,
                settings,
            } => self.update_settings(seat, settings_revision, settings),
            RosterAction::Movement(direction) => {
                self.require_running()?;
                self.inputs
                    .movement(seat, direction, self.lifecycle.tick())
                    .map_err(input)?;
                Ok(None)
            }
            RosterAction::Bomb(action_id) => {
                self.require_running()?;
                publication::counter(action_id).map_err(|_| RosterRuntimeError::DuplicateAction)?;
                self.inputs.bomb(seat, action_id).map_err(input)?;
                Ok(None)
            }
            RosterAction::Equipment(action_id) => {
                self.require_running()?;
                publication::counter(action_id).map_err(|_| RosterRuntimeError::DuplicateAction)?;
                if !self.lifecycle.settings().enabled_items().contains_punch() {
                    return Err(RosterRuntimeError::InvalidPhase);
                }
                self.inputs.equipment(seat, action_id).map_err(input)?;
                Ok(None)
            }
            RosterAction::Rematch(ready) => {
                let change = self
                    .lifecycle
                    .set_rematch_vote(seat, ready)
                    .map_err(lifecycle)?;
                if change.change() == MatchChange::RematchStarted {
                    self.reset_for_rematch();
                }
                self.notice_for(change.change(), self.sim.is_some())
            }
        }
    }

    pub(in crate::game::rooms) fn settle_boundary(
        &mut self,
        departures: [Option<DepartureReason>; 4],
    ) -> Result<Option<RosterNotice>, RosterRuntimeError> {
        let mut next = self.clone();
        let notice = next.settle(departures)?;
        *self = next;
        Ok(notice)
    }

    fn settle(
        &mut self,
        departures: [Option<DepartureReason>; 4],
    ) -> Result<Option<RosterNotice>, RosterRuntimeError> {
        if matches!(self.lifecycle.phase(), MatchPhase::Interrupted { .. }) {
            return Ok(None);
        }
        if self.lifecycle.phase() != MatchPhase::Running {
            return self.settle_idle(departures);
        }
        let mut publish = self.step_physics()?;
        // Preview the whole human-loss batch first. Physical elimination can then
        // finish the current round at this tick before lifecycle forfeits settle.
        let mut preview = self.lifecycle.clone();
        let mut eliminated = [false; 4];
        if departures.iter().any(Option::is_some) {
            let update = preview.depart_with_grace(departures).map_err(lifecycle)?;
            mark_eliminated(&mut eliminated, update.newly_ineligible_seats());
        }
        let preview_expiry = preview.expire_reconnect_grace().map_err(lifecycle)?;
        mark_eliminated(&mut eliminated, preview_expiry.newly_ineligible_seats());
        if eliminated.into_iter().any(|value| value) {
            self.eliminate_departed(eliminated)?;
            if self.lifecycle.phase() == MatchPhase::Running
                && let Some(result) = self.sim.as_ref().and_then(|sim| sim.state.result())
            {
                self.lifecycle
                    .complete_current_tick(result)
                    .map_err(lifecycle)?;
            }
            publish = true;
        }
        if departures.iter().any(Option::is_some) {
            self.apply_departures(departures)?;
            for (index, departure) in departures.into_iter().enumerate() {
                if departure.is_some() {
                    self.inputs.stop(SeatId::ALL[index]);
                }
            }
            publish = true;
        }
        let expiry = self.expire_departures()?;
        publish |= expiry.change() != MatchChange::Unchanged;
        if publish {
            self.publish_current().map(Some)
        } else {
            Ok(None)
        }
    }

    fn settle_idle(
        &mut self,
        departures: [Option<DepartureReason>; 4],
    ) -> Result<Option<RosterNotice>, RosterRuntimeError> {
        let mut publish = false;
        let mut rematch_started = false;
        if departures.iter().any(Option::is_some) {
            let update = self.apply_departures(departures)?;
            rematch_started |= update.change() == MatchChange::RematchStarted;
            for (index, departure) in departures.into_iter().enumerate() {
                if departure.is_some() {
                    self.inputs.stop(SeatId::ALL[index]);
                }
            }
            publish = true;
        }
        if !rematch_started && !matches!(self.lifecycle.phase(), MatchPhase::Interrupted { .. }) {
            publish |= self.idle_tick()?;
        }
        let expiry = self.expire_departures()?;
        rematch_started |= expiry.change() == MatchChange::RematchStarted;
        publish |= expiry.change() != MatchChange::Unchanged;
        if rematch_started {
            self.reset_for_rematch();
        }
        if publish {
            self.publish_current().map(Some)
        } else {
            Ok(None)
        }
    }

    fn step_physics(&mut self) -> Result<bool, RosterRuntimeError> {
        let next_tick = self
            .lifecycle
            .tick()
            .0
            .checked_add(1)
            .map(Tick)
            .ok_or(RosterRuntimeError::CapacityReached)?;
        let inputs = self.inputs.take(next_tick);
        let sim = self.sim.as_mut().ok_or(RosterRuntimeError::Publication)?;
        let report = sim
            .state
            .step(&inputs)
            .map_err(|_| RosterRuntimeError::CapacityReached)?;
        let result = sim.state.result();
        self.lifecycle
            .complete_running_tick(result)
            .map_err(lifecycle)?;
        self.reports.push(report);
        if result.is_some() {
            self.inputs.clear_pending();
        }
        Ok(result.is_some() || self.reports.len() == 3)
    }

    fn idle_tick(&mut self) -> Result<bool, RosterRuntimeError> {
        let update = self.lifecycle.advance_idle_tick().map_err(lifecycle)?;
        if let MatchChange::RoundStarted(assignment) = update.change() {
            self.start_round(assignment)?;
            return Ok(true);
        }
        Ok(update.change() != MatchChange::Unchanged && self.lifecycle.tick().0.is_multiple_of(3))
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn prepare_cpu_publication(
        &self,
    ) -> Result<Option<v14::BoundPublication>, RosterRuntimeError> {
        Ok(self
            .reports
            .is_empty()
            .then(|| self.cpu_publication.clone())
            .flatten())
    }

    #[cfg(test)]
    fn update_settings(
        &mut self,
        seat: SeatId,
        expected: u32,
        settings: RoomSettings,
    ) -> Result<Option<RosterNotice>, RosterRuntimeError> {
        if self.settings_locked || self.lifecycle.settings_editor() != Some(seat) {
            return Err(RosterRuntimeError::SettingsLocked);
        }
        if expected != self.settings_revision {
            return Err(RosterRuntimeError::StaleSettings);
        }
        if settings == self.lifecycle.settings() {
            let room = publication::room_view(
                &self.identity,
                &self.lifecycle,
                self.revision,
                self.settings_revision,
            )?;
            return Ok(Some(publication::view_notice(room)));
        }
        let change = self
            .lifecycle
            .set_settings(seat, settings)
            .map_err(lifecycle)?;
        self.settings_revision = self
            .settings_revision
            .checked_add(1)
            .ok_or(RosterRuntimeError::CapacityReached)?;
        self.notice_for(change.change(), false)
    }

    pub(super) fn validate_actor(
        &self,
        seat: SeatId,
        epoch: u64,
    ) -> Result<(), RosterRuntimeError> {
        if epoch != self.lifecycle.round_attempt() {
            return Err(RosterRuntimeError::StaleEpoch);
        }
        if !matches!(
            self.lifecycle.participant(seat).map_err(lifecycle)?,
            RosterParticipant::Connected | RosterParticipant::Cpu
        ) {
            return Err(RosterRuntimeError::VacantSeat);
        }
        Ok(())
    }

    fn require_running(&self) -> Result<(), RosterRuntimeError> {
        (self.lifecycle.phase() == MatchPhase::Running)
            .then_some(())
            .ok_or(RosterRuntimeError::InvalidPhase)
    }

    pub(super) fn next_revision(&self) -> Result<u32, RosterRuntimeError> {
        self.revision
            .checked_add(1)
            .and_then(|value| publication::counter(value).ok())
            .map(|counter| counter.get())
            .ok_or(RosterRuntimeError::CapacityReached)
    }

    pub(super) fn reset_for_rematch(&mut self) {
        self.sim = None;
        self.reports.clear();
        self.cpu_publication = None;
        self.environment_cpu_publication = None;
        self.lair_cpu_publication = None;
        self.inputs.reset_round();
    }
}

use grenadiers_core::{
    game::{EntityId, MobNavigationPolicy, State},
    room::MatchPhase,
};

use super::{error::RosterRuntimeError, runtime::RosterRuntime, types::RosterNotice};
use crate::game::rooms::input::InputMailbox;

impl RosterRuntime {
    pub(in crate::game::rooms) fn install_running_scene_for_test(
        &mut self,
        state: State,
    ) -> Result<RosterNotice, RosterRuntimeError> {
        if self.lifecycle.phase() != MatchPhase::Running {
            return Err(RosterRuntimeError::InvalidPhase);
        }
        let current = self.sim.as_ref().ok_or(RosterRuntimeError::Publication)?;
        let configuration = self.identity.configuration;
        let eligible: Vec<_> = self.lifecycle.eligible_seats().collect();
        let actors_match = state.actors().iter().enumerate().all(|(index, actor)| {
            let seat = grenadiers_core::game::SeatId::ALL[index];
            actor.id() == EntityId(u32::try_from(index + 1).unwrap())
                && actor.seat() == seat
                && actor.alive() == eligible.contains(&seat)
        });
        if current.epoch != self.lifecycle.round_attempt()
            || state.tick() != self.lifecycle.tick()
            || state.running_updates() != current.state.running_updates()
            || state.rules() != current.state.rules()
            || state.rules().room_settings() != Some(self.lifecycle.settings())
            || state.rules().mob_navigation_policy() != MobNavigationPolicy::TimedEscapeV3
            || state.round_assignment() != current.state.round_assignment()
            || state.player_count() != configuration.player_count()
            || state.actors().len() != configuration.player_count().get()
            || !actors_match
            || state.world().arena().board() != configuration.board()
            || state.result().is_some()
        {
            return Err(RosterRuntimeError::Publication);
        }

        let mut next = self.clone();
        // The retained generation record is mandatory publication context only.
        // This test seam makes no generated-world or replay-provenance claim for `state`.
        next.sim.as_mut().unwrap().state = state;
        next.inputs = InputMailbox::for_configuration(configuration);
        next.reports.clear();
        next.cpu_publication = None;
        next.environment_cpu_publication = None;
        next.lair_cpu_publication = None;
        next.respawn_cpu_publication = None;
        next.spike_cpu_publication = None;
        next.spike_v19_cpu_publication = None;
        next.spike_v20_cpu_publication = None;
        let notice = next.publish_current()?;
        *self = next;
        Ok(notice)
    }

    pub(in crate::game::rooms) fn state_for_test(&self) -> Option<&State> {
        self.sim.as_ref().map(|sim| &sim.state)
    }

    pub(in crate::game::rooms) fn pending_report_count_for_test(&self) -> usize {
        self.reports.len()
    }

    pub(in crate::game::rooms) fn publication_caches_for_test(&self) -> [bool; 7] {
        [
            self.cpu_publication.is_some(),
            self.environment_cpu_publication.is_some(),
            self.lair_cpu_publication.is_some(),
            self.respawn_cpu_publication.is_some(),
            self.spike_cpu_publication.is_some(),
            self.spike_v19_cpu_publication.is_some(),
            self.spike_v20_cpu_publication.is_some(),
        ]
    }
}

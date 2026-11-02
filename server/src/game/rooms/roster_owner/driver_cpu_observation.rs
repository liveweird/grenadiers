use grenadiers_core::{
    cpu::{
        WorldId,
        policy::EscapeExecution,
        survival::{PulseStatus, RouteStatus},
    },
    protocol::{v14, v15, v16, v17, v18, v19, v20},
    room::MatchPhase,
};

use super::{ContactStatus, Driver, wire_seat};
#[cfg(test)]
use grenadiers_core::game::Tick;

impl Driver {
    pub(in crate::game::rooms::roster_owner::driver) fn observe_cpus(
        &mut self,
        notice: &crate::game::rooms::roster_runtime::RosterNotice,
    ) {
        let Some(publication_sequence) = self.publication_sequence else {
            for cpu in &mut self.cpus {
                let seat = cpu.seat();
                if !cpu.observe_continuation_publication(u64::MAX, None) {
                    self.cpu_stop_at[seat.index()] = Some(self.runtime.lifecycle().tick());
                }
            }
            return;
        };
        if self.runtime.lifecycle().phase() != MatchPhase::Running {
            for cpu in &mut self.cpus {
                let _ = cpu.observe_continuation_publication(publication_sequence, None);
                cpu.cancel();
            }
            return;
        }
        let physical_round = self.runtime.lifecycle().round_attempt();
        let room_instance = self.runtime.identity().registry_serial;
        let world = WorldId {
            room_instance,
            physical_round,
        };
        let observations: Vec<Option<_>> =
            if let Some(publication) = notice.spike_v20_cpu_publication() {
                self.cpus
                    .iter()
                    .map(|cpu| {
                        v20::observe_cpu_recipient(publication, wire_seat(cpu.seat()), world).ok()
                    })
                    .collect()
            } else if let Some(publication) = notice.spike_v19_cpu_publication() {
                self.cpus
                    .iter()
                    .map(|cpu| {
                        v19::observe_cpu_recipient(publication, wire_seat(cpu.seat()), world).ok()
                    })
                    .collect()
            } else if let Some(publication) = notice.spike_cpu_publication() {
                self.cpus
                    .iter()
                    .map(|cpu| {
                        v18::observe_cpu_recipient(publication, wire_seat(cpu.seat()), world).ok()
                    })
                    .collect()
            } else if let Some(publication) = notice.respawn_cpu_publication() {
                self.cpus
                    .iter()
                    .map(|cpu| {
                        v17::observe_cpu_recipient(publication, wire_seat(cpu.seat()), world).ok()
                    })
                    .collect()
            } else if let Some(publication) = notice.lair_cpu_publication() {
                self.cpus
                    .iter()
                    .map(|cpu| {
                        v16::observe_cpu_recipient(publication, wire_seat(cpu.seat()), world).ok()
                    })
                    .collect()
            } else if let Some(publication) = notice.environment_cpu_publication() {
                self.cpus
                    .iter()
                    .map(|cpu| {
                        v15::observe_cpu_recipient(publication, wire_seat(cpu.seat()), world).ok()
                    })
                    .collect()
            } else if let Some(publication) = notice.legacy_cpu_publication() {
                self.cpus
                    .iter()
                    .map(|cpu| {
                        v14::observe_cpu_recipient(publication, wire_seat(cpu.seat()), world).ok()
                    })
                    .collect()
            } else {
                for cpu in &mut self.cpus {
                    if !cpu.observe_continuation_publication(publication_sequence, None) {
                        let seat = cpu.seat();
                        self.cpu_stop_at[seat.index()] = Some(self.runtime.lifecycle().tick());
                    }
                }
                return;
            };
        let epoch = self.runtime.lifecycle().round_attempt();
        let mut urgent_candidates = vec![false; observations.len()];
        let mut refuge_candidates = vec![false; observations.len()];
        for (index, observation) in observations.iter().enumerate() {
            let seat = self.cpus[index].seat();
            if !self.cpus[index]
                .observe_continuation_publication(publication_sequence, observation.as_ref())
            {
                self.cpu_stop_at[seat.index()] = Some(self.runtime.lifecycle().tick());
                continue;
            }
            let Some(observation) = observation.as_ref() else {
                continue;
            };
            self.cpus[index].validate_escape_epoch(epoch);
            if !self.cpus[index].accept_observation(observation) {
                continue;
            }
            match self.cpus[index].observe_teleporter(observation) {
                Some(grenadiers_core::cpu::survival::TeleporterStepStatus::Completed(_)) => {}
                Some(grenadiers_core::cpu::survival::TeleporterStepStatus::Invalidated) => {
                    self.cpu_stop_at[seat.index()] = Some(self.runtime.lifecycle().tick());
                    continue;
                }
                Some(_) => continue,
                None if self.cpus[index].teleporter_controlled() => continue,
                None => {}
            }
            if self.cpus[index].validate_contact(observation, observation.parts().tick)
                == Some(ContactStatus::Invalidated)
            {
                self.cpu_stop_at[seat.index()] = Some(self.runtime.lifecycle().tick());
                continue;
            }
            let route = self.cpus[index].observe_route(observation);
            if route == Some(RouteStatus::Invalidated) {
                self.cpu_stop_at[seat.index()] = Some(self.runtime.lifecycle().tick());
                continue;
            }
            if matches!(
                route,
                Some(RouteStatus::Continue | RouteStatus::Wait | RouteStatus::Submit(_))
            ) {
                refuge_candidates[index] = route == Some(RouteStatus::Wait)
                    && self.cpu_stationary[seat.index()]
                    && !self.cpus[index].has_pending();
                continue;
            }
            let pulse = self.cpus[index].validate_pulse(observation, observation.parts().tick);
            if pulse == Some(PulseStatus::Invalidated) && !self.cpu_stationary[seat.index()] {
                self.cpu_stop_at[seat.index()] = Some(self.runtime.lifecycle().tick());
                continue;
            }
            if matches!(pulse, Some(PulseStatus::Continue | PulseStatus::Wait)) {
                continue;
            }
            if self.cpus[index].is_escaping() {
                match self.cpus[index].follow_escape(observation).ok() {
                    Some(EscapeExecution::Continue(plan) | EscapeExecution::Wait(plan))
                        if self.cpu_stationary[seat.index()] =>
                    {
                        let _ = self.apply_escape(index, plan);
                        continue;
                    }
                    Some(EscapeExecution::Continue(_) | EscapeExecution::Wait(_)) => continue,
                    Some(EscapeExecution::Invalidated) if !self.cpu_stationary[seat.index()] => {
                        self.cpu_stop_at[seat.index()] = Some(self.runtime.lifecycle().tick());
                        continue;
                    }
                    Some(EscapeExecution::Completed | EscapeExecution::Invalidated) | None => {}
                }
            }
            if self.cpu_stationary[seat.index()] && self.cpus[index].needs_stop_confirmation() {
                self.cpu_stop_at[seat.index()] = Some(self.runtime.lifecycle().tick());
            } else if self.cpu_stationary[seat.index()] && !self.cpus[index].has_pending() {
                urgent_candidates[index] = true;
            }
        }
        let available = observations
            .iter()
            .map(|value| value.as_ref())
            .collect::<Vec<_>>();
        self.submit_cpu_candidates(&available, &urgent_candidates, &refuge_candidates, epoch);
    }

    pub(super) fn submit_cpu_candidates(
        &mut self,
        observations: &[Option<&grenadiers_core::cpu::Observation>],
        urgent: &[bool],
        refuge: &[bool],
        epoch: u64,
    ) {
        for (index, observation) in observations.iter().enumerate() {
            if urgent[index]
                && let Some(observation) = observation
            {
                let _ = self.cpus[index].submit((*observation).clone(), epoch);
            }
        }
        // Refuge replacement is opportunistic. Every seat's ordinary/urgent
        // work gets first access to the bounded shared queue.
        for (index, observation) in observations.iter().enumerate() {
            if refuge[index]
                && let Some(observation) = observation
            {
                let _ = self.cpus[index].submit_refuge_handoff((*observation).clone(), epoch);
            }
        }
    }

    #[cfg(test)]
    pub(in crate::game::rooms::roster_owner) fn expire_contact_for_test(
        &mut self,
        index: usize,
    ) -> bool {
        let Some(observation) = self.cpus[index].latest_observation_for_test() else {
            return false;
        };
        if self.cpus[index].validate_contact(&observation, Tick(u64::MAX - 1))
            != Some(ContactStatus::Invalidated)
        {
            return false;
        }
        let seat = self.cpus[index].seat();
        self.cpu_stop_at[seat.index()] = Some(self.runtime.lifecycle().tick());
        true
    }
}

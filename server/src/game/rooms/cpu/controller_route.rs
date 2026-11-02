use grenadiers_core::cpu::Observation;
#[cfg(test)]
use grenadiers_core::cpu::routing::PlantPresence;
#[cfg(test)]
use grenadiers_core::cpu::survival::CommittedRoute;
use grenadiers_core::cpu::survival::{RouteCommand, RouteStatus};
use grenadiers_core::game::Tick;

use super::{AcceptedPlan, CpuController};

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) enum RouteTracePhase {
    Observe,
    Poll,
    Commit,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) struct RouteTrace {
    pub tick: Tick,
    pub phase: RouteTracePhase,
    pub status: RouteStatus,
    pub invalidator: Option<grenadiers_core::cpu::survival::RouteInvalidator>,
    pub plant: Option<PlantPresence>,
    pub escape_origin_after: bool,
}

impl AcceptedPlan {
    pub(in crate::game::rooms) const fn has_route(&self) -> bool {
        self.committed_route.is_some()
    }
}

impl CpuController {
    #[cfg(test)]
    pub(in crate::game::rooms) const fn has_committed_route(&self) -> bool {
        self.committed_route.is_some()
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn plant_presence(&self) -> Option<PlantPresence> {
        self.committed_route
            .as_ref()
            .and_then(CommittedRoute::plant_presence)
    }

    pub(in crate::game::rooms) fn observe_route(
        &mut self,
        observation: &Observation,
    ) -> Option<RouteStatus> {
        let settled_stop_tick =
            (!self.escaping && self.escape_origin.is_none() && self.committed_pulse.is_none())
                .then_some(self.settled_stop_tick)
                .flatten();
        #[cfg(test)]
        let invalidator = self
            .committed_route
            .as_ref()
            .and_then(|route| route.first_invalidator(observation));
        let status = self
            .committed_route
            .as_mut()?
            .observe_with_stop(observation, settled_stop_tick);
        #[cfg(test)]
        let plant = self
            .committed_route
            .as_ref()
            .and_then(CommittedRoute::plant_presence);
        if matches!(status, RouteStatus::Completed | RouteStatus::Invalidated) {
            let origin = (status == RouteStatus::Invalidated)
                .then(|| self.committed_route.as_ref()?.escape_origin(observation))
                .flatten();
            let origin_epoch = origin.as_ref().and(self.committed_route_epoch);
            self.clear_route(status == RouteStatus::Invalidated);
            self.escape_origin = origin;
            self.escape_origin_epoch = origin_epoch;
        }
        #[cfg(test)]
        self.route_trace.push(RouteTrace {
            tick: observation.parts().tick,
            phase: RouteTracePhase::Observe,
            status,
            invalidator,
            plant,
            escape_origin_after: self.escape_origin.is_some(),
        });
        Some(status)
    }

    pub(in crate::game::rooms) fn poll_route_command(
        &mut self,
        current_tick: Tick,
        command_epoch: u64,
    ) -> Option<RouteStatus> {
        self.committed_route.as_ref()?;
        if self.committed_route_epoch != Some(command_epoch) {
            self.clear_route(true);
            return Some(RouteStatus::Invalidated);
        }
        let status = self
            .committed_route
            .as_ref()?
            .poll_command(current_tick, self.settled_stop_tick);
        #[cfg(test)]
        let plant = self
            .committed_route
            .as_ref()
            .and_then(CommittedRoute::plant_presence);
        if matches!(status, RouteStatus::Completed | RouteStatus::Invalidated) {
            self.clear_route(status == RouteStatus::Invalidated);
        }
        #[cfg(test)]
        self.route_trace.push(RouteTrace {
            tick: current_tick,
            phase: RouteTracePhase::Poll,
            status,
            invalidator: None,
            plant,
            escape_origin_after: self.escape_origin.is_some(),
        });
        Some(status)
    }

    pub(in crate::game::rooms) fn commit_route_command(&mut self, command: RouteCommand) -> bool {
        let committed = self
            .committed_route
            .as_mut()
            .is_some_and(|route| route.commit_command(command));
        #[cfg(test)]
        self.route_trace.push(RouteTrace {
            tick: command.admission_tick(),
            phase: RouteTracePhase::Commit,
            status: if committed {
                RouteStatus::Submit(command)
            } else {
                RouteStatus::Invalidated
            },
            invalidator: None,
            plant: self
                .committed_route
                .as_ref()
                .and_then(CommittedRoute::plant_presence),
            escape_origin_after: self.escape_origin.is_some(),
        });
        committed
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn route_trace_for_test(&self) -> &[RouteTrace] {
        &self.route_trace
    }

    pub(in crate::game::rooms) fn invalidate_route(&mut self) {
        self.clear_route(true);
    }

    fn clear_route(&mut self, reset_memory: bool) {
        self.committed_route = None;
        self.committed_route_epoch = None;
        self.committed_contact = None;
        self.cancel();
        self.clear_escape_origin();
        if reset_memory {
            self.memory = Some(
                self.memory
                    .expect("world memory initialized")
                    .without_execution_authority(),
            );
        }
    }
}

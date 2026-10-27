use grenadiers_core::cpu::Observation;
use grenadiers_core::cpu::policy::UtilityMemory;
#[cfg(test)]
use grenadiers_core::cpu::routing::PlantPresence;
#[cfg(test)]
use grenadiers_core::cpu::survival::CommittedRoute;
use grenadiers_core::cpu::survival::{RouteCommand, RouteStatus};
use grenadiers_core::game::Tick;

use super::{AcceptedPlan, CpuController};

impl AcceptedPlan {
    pub(in crate::game::rooms) const fn has_route(&self) -> bool {
        self.committed_route.is_some()
    }
}

impl CpuController {
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
        let status = self.committed_route.as_mut()?.observe(observation);
        if matches!(status, RouteStatus::Completed | RouteStatus::Invalidated) {
            self.clear_route(status == RouteStatus::Invalidated);
        }
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
        if matches!(status, RouteStatus::Completed | RouteStatus::Invalidated) {
            self.clear_route(status == RouteStatus::Invalidated);
        }
        Some(status)
    }

    pub(in crate::game::rooms) fn commit_route_command(&mut self, command: RouteCommand) -> bool {
        self.committed_route
            .as_mut()
            .is_some_and(|route| route.commit_command(command))
    }

    pub(in crate::game::rooms) fn invalidate_route(&mut self) {
        self.clear_route(true);
    }

    fn clear_route(&mut self, reset_memory: bool) {
        self.committed_route = None;
        self.committed_route_epoch = None;
        self.cancel();
        if reset_memory {
            self.memory = Some(UtilityMemory::new(
                self.memory
                    .as_ref()
                    .expect("world memory initialized")
                    .key(),
            ));
        }
    }
}

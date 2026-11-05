use crate::cpu::policy::PolicyError;
use crate::cpu::routing::PlantPresence;
use crate::cpu::{MovementIntent, Observation, WorkBudget};
use crate::game::Tick;

use super::compatibility::{charge_route_actor_compatibility, plant_actor_compatible_charged};
use super::route_execution::CommittedPlant;
use super::{CommittedRoute, ExecutionCertificate, RouteCommand, RouteStatus};

impl CommittedRoute {
    #[must_use]
    pub const fn requires_owned_combination_support(&self) -> bool {
        self.owned_combination
    }

    pub fn admit_reactive_execution(
        execution: ExecutionCertificate,
        basis: Observation,
        current: &Observation,
        admission_tick: Tick,
        accepted_movement: MovementIntent,
    ) -> Result<Self, PolicyError> {
        let certificate = match execution {
            ExecutionCertificate::ReactiveRoute(certificate) => {
                let (route, assumptions) = certificate.into_parts();
                let mut work = WorkBudget::new(super::reactive_plant::REACTIVE_GUARD_ALLOWANCE);
                let charge = route
                    .evolution()
                    .reactive_scene_work(&basis, current)
                    .ok_or(PolicyError::InvalidObservation)?;
                if !assumptions.matches(current, &mut work)
                    || work.try_take(charge).is_err()
                    || !charge_route_actor_compatibility(
                        &basis,
                        current,
                        route.evolution(),
                        &mut work,
                    )
                {
                    return Err(PolicyError::InvalidObservation);
                }
                let mut admitted = Self::admit_inner(
                    route,
                    None,
                    basis,
                    current,
                    admission_tick,
                    accepted_movement,
                )?;
                admitted.reactive = Some(assumptions);
                return Ok(admitted);
            }
            ExecutionCertificate::ReactivePlant(certificate) => certificate,
            _ => return Err(PolicyError::InvalidObservation),
        };
        let (plant, assumptions, owned_combination) = certificate.into_parts();
        if owned_combination.is_some() {
            return Err(PolicyError::InvalidObservation);
        }
        let route = plant.route().clone();
        let mut work = WorkBudget::new(super::reactive_plant::REACTIVE_GUARD_ALLOWANCE);
        if !assumptions.matches(current, &mut work)
            || plant.reactive_scene_candidate(&basis, current, PlantPresence::Awaiting, &mut work)
                != Some(PlantPresence::Awaiting)
            || !plant_actor_compatible_charged(
                &basis,
                current,
                &plant,
                PlantPresence::Awaiting,
                &mut work,
            )
        {
            return Err(PolicyError::InvalidObservation);
        }
        let mut admitted = Self::admit_inner(
            route,
            Some(CommittedPlant {
                certificate: plant,
                presence: PlantPresence::Awaiting,
                contact: None,
            }),
            basis,
            current,
            admission_tick,
            accepted_movement,
        )?;
        admitted.reactive = Some(assumptions);
        Ok(admitted)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn admit_owned_combination_execution(
        execution: ExecutionCertificate,
        basis: Observation,
        current: &Observation,
        admission_tick: Tick,
        accepted_movement: MovementIntent,
        handoff: &super::TerminalRefugeHandoff,
    ) -> Result<Self, PolicyError> {
        let mut work = WorkBudget::new(super::reactive_plant::REACTIVE_GUARD_ALLOWANCE);
        if !execution.requires_owned_combination_support()
            || !execution.owned_combination_handoff_matches(handoff, &mut work)
            || !handoff.authorizes_owned_combination(&basis, current, &mut work)
            || !handoff.accepts_replacement_charged(&execution, &mut work)
        {
            return Err(PolicyError::InvalidObservation);
        }
        let ExecutionCertificate::ReactivePlant(certificate) = execution else {
            return Err(PolicyError::InvalidObservation);
        };
        let (plant, assumptions, Some(_owned_combination)) = certificate.into_parts() else {
            return Err(PolicyError::InvalidObservation);
        };
        let route = plant.route().clone();
        if !assumptions.matches(current, &mut work)
            || plant.reactive_scene_candidate(&basis, current, PlantPresence::Awaiting, &mut work)
                != Some(PlantPresence::Awaiting)
            || !plant_actor_compatible_charged(
                &basis,
                current,
                &plant,
                PlantPresence::Awaiting,
                &mut work,
            )
        {
            return Err(PolicyError::InvalidObservation);
        }
        let mut admitted = Self::admit_inner(
            route,
            Some(CommittedPlant {
                certificate: plant,
                presence: PlantPresence::Awaiting,
                contact: None,
            }),
            basis,
            current,
            admission_tick,
            accepted_movement,
        )?;
        admitted.reactive = Some(assumptions);
        admitted.owned_combination = true;
        Ok(admitted)
    }

    #[must_use]
    pub fn poll_command(&self, current_tick: Tick, settled_stop_tick: Option<Tick>) -> RouteStatus {
        if self.requires_reactive_support() {
            return RouteStatus::Invalidated;
        }
        self.poll_command_inner(current_tick, settled_stop_tick)
    }

    pub fn poll_reactive_command(
        &self,
        current_tick: Tick,
        latest: &Observation,
        settled_stop_tick: Option<Tick>,
    ) -> RouteStatus {
        if !self.reactive_guard_matches(latest) {
            return RouteStatus::Invalidated;
        }
        self.poll_command_inner(current_tick, settled_stop_tick)
    }

    pub fn commit_command(&mut self, command: RouteCommand) -> bool {
        if self.requires_reactive_support() {
            return false;
        }
        self.commit_command_inner(command)
    }

    pub fn commit_reactive_command(
        &mut self,
        command: RouteCommand,
        current: &Observation,
    ) -> bool {
        self.reactive_guard_matches(current) && self.commit_command_inner(command)
    }

    fn reactive_guard_matches(&self, current: &Observation) -> bool {
        current.parts().tick == self.last_observation_tick
            && current == &self.last_observation
            && self.reactive.as_ref().is_some_and(|assumptions| {
                assumptions.matches(
                    current,
                    &mut WorkBudget::new(super::reactive_plant::REACTIVE_GUARD_ALLOWANCE),
                )
            })
    }
}

use crate::cpu::policy::PolicyError;
use crate::cpu::routing::{PlantCertificate, PlantPresence, RouteCertificate, RouteStep};
use crate::cpu::{MemoryKey, MovementIntent, Observation, WorkBudget};
use crate::game::Tick;

use super::compatibility::{
    actor, plant_actor_compatible, route_compatible, visible_rivals_allow_segment,
};
use super::manual_execution::CommittedPunch;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteStatus {
    Continue,
    Wait,
    Submit(RouteCommand),
    Completed,
    Invalidated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RouteCommand {
    movement: MovementIntent,
    admission_tick: Tick,
    step_index: u8,
}

impl RouteCommand {
    #[must_use]
    pub const fn movement(self) -> MovementIntent {
        self.movement
    }

    #[must_use]
    pub const fn admission_tick(self) -> Tick {
        self.admission_tick
    }

    pub const fn step_index(self) -> u8 {
        self.step_index
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Transition {
    pub(super) command: RouteCommand,
    pub(super) required_stop_snapshot: Option<Tick>,
}

#[derive(Clone, Debug)]
pub struct CommittedRoute {
    pub(super) certificate: RouteCertificate,
    pub(super) plant: Option<CommittedPlant>,
    pub(super) punch: Option<CommittedPunch>,
    pub(super) basis: Observation,
    pub(super) first_update: Tick,
    pub(super) transitions: Vec<Transition>,
    pub(super) next_transition: usize,
    pub(super) last_observation_tick: Tick,
    pub(super) last_observation: Observation,
    pub(super) movement: MovementIntent,
}
#[derive(Clone, Debug)]
pub(super) struct CommittedPlant {
    pub(super) certificate: PlantCertificate,
    pub(super) presence: PlantPresence,
}

impl CommittedRoute {
    pub fn admit(
        certificate: RouteCertificate,
        basis: Observation,
        current: &Observation,
        admission_tick: Tick,
        accepted_movement: MovementIntent,
    ) -> Result<Self, PolicyError> {
        Self::admit_inner(
            certificate,
            None,
            basis,
            current,
            admission_tick,
            accepted_movement,
        )
    }

    pub(super) fn admit_inner(
        certificate: RouteCertificate,
        plant: Option<CommittedPlant>,
        basis: Observation,
        current: &Observation,
        admission_tick: Tick,
        accepted_movement: MovementIntent,
    ) -> Result<Self, PolicyError> {
        let first_update = admission_tick
            .0
            .checked_add(1)
            .map(Tick)
            .ok_or(PolicyError::InvalidObservation)?;
        let age = admission_tick
            .0
            .checked_sub(basis.parts().tick.0)
            .ok_or(PolicyError::InvalidObservation)?;
        let key = MemoryKey {
            world: basis.parts().world,
            cpu_seat: basis.parts().recipient,
        };
        let lane = certificate
            .lane(first_update)
            .ok_or(PolicyError::InvalidObservation)?;
        let expected_pose = lane
            .pose_at(current.parts().tick)
            .ok_or(PolicyError::InvalidObservation)?;
        let current_actor = actor(current).ok_or(PolicyError::InvalidObservation)?;
        let scene_matches = plant.as_ref().map_or_else(
            || route_compatible(&basis, current, certificate.evolution()),
            |plant| {
                current.parts().tick == admission_tick
                    && plant
                        .certificate
                        .matches_scene(&basis, current, PlantPresence::Awaiting)
                    && plant_actor_compatible(
                        &basis,
                        current,
                        &plant.certificate,
                        PlantPresence::Awaiting,
                    )
            },
        );
        let spike_pits_match = current.parts().rules.version != 10
            || lane
                .spike_pits_match_at(
                    current,
                    &mut WorkBudget::new(crate::cpu::survival::WORK_ALLOWANCE),
                )
                .unwrap_or(false);
        if age > super::MAX_PLAN_AGE_TICKS
            || current.parts().tick > admission_tick
            || certificate.key() != key
            || certificate.basis_tick() != basis.parts().tick
            || !certificate.start_window().contains(first_update)
            || !scene_matches
            || !spike_pits_match
            || !crate::cpu::routing::storm::observation_fits(
                &basis,
                current,
                lane.storm_deadlines_at(current.parts().tick),
            )
            || expected_pose != (current_actor.position, current_actor.heading)
            || lane.pose_at(first_update).is_none_or(|next| {
                !visible_rivals_allow_segment(current, current_actor.position, next.0)
            })
        {
            return Err(PolicyError::InvalidObservation);
        }
        let transitions = transitions(&certificate, first_update)?;
        let Some(initial) = transitions.first() else {
            return Err(PolicyError::InvalidObservation);
        };
        if initial.command.admission_tick != admission_tick
            || initial.command.movement != accepted_movement
        {
            return Err(PolicyError::InvalidObservation);
        }
        Ok(Self {
            certificate,
            plant,
            punch: None,
            basis,
            first_update,
            transitions,
            next_transition: 1,
            last_observation_tick: current.parts().tick,
            last_observation: current.clone(),
            movement: accepted_movement,
        })
    }

    pub fn observe(&mut self, current: &Observation) -> RouteStatus {
        self.observe_with_stop(current, None)
    }

    pub fn observe_with_stop(
        &mut self,
        current: &Observation,
        settled_stop_tick: Option<Tick>,
    ) -> RouteStatus {
        let tick = current.parts().tick;
        if tick <= self.last_observation_tick {
            return RouteStatus::Invalidated;
        }
        if !self.scene_matches(current) {
            return RouteStatus::Invalidated;
        }
        if tick >= self.certificate.survive_until() {
            let current_key = MemoryKey {
                world: current.parts().world,
                cpu_seat: current.parts().recipient,
            };
            let same_actor = actor(&self.basis)
                .zip(actor(current))
                .is_some_and(|(before, after)| before.id == after.id && after.alive);
            if self.next_transition != self.transitions.len()
                || current_key != self.certificate.key()
                || !same_actor
            {
                return RouteStatus::Invalidated;
            }
            self.last_observation_tick = tick;
            self.last_observation = current.clone();
            return RouteStatus::Completed;
        }
        let Some(expected) = self
            .certificate
            .lane(self.first_update)
            .and_then(|lane| lane.pose_at(tick))
        else {
            return RouteStatus::Invalidated;
        };
        let deadlines = self
            .certificate
            .lane(self.first_update)
            .and_then(|lane| lane.storm_deadlines_at(tick));
        let spike_pits_match = current.parts().rules.version != 10
            || self
                .certificate
                .lane(self.first_update)
                .is_some_and(|lane| {
                    lane.spike_pits_match_at(
                        current,
                        &mut WorkBudget::new(crate::cpu::survival::WORK_ALLOWANCE),
                    )
                    .unwrap_or(false)
                });
        if !crate::cpu::routing::storm::observation_fits(&self.basis, current, deadlines) {
            return RouteStatus::Invalidated;
        }
        if !spike_pits_match {
            return RouteStatus::Invalidated;
        }
        let Some(current_actor) = actor(current) else {
            return RouteStatus::Invalidated;
        };
        if expected != (current_actor.position, current_actor.heading) {
            return RouteStatus::Invalidated;
        }
        if let Some(next_tick) = tick.0.checked_add(1).map(Tick)
            && let Some(next) = self
                .certificate
                .lane(self.first_update)
                .and_then(|lane| lane.pose_at(next_tick))
            && !visible_rivals_allow_segment(current, current_actor.position, next.0)
        {
            return RouteStatus::Invalidated;
        }
        if super::clear_gap::can_complete(super::clear_gap::Input {
            basis: &self.basis,
            current,
            certificate: &self.certificate,
            nonplant: self.plant.is_none() && self.punch.is_none(),
            movement: self.movement,
            no_pending: self.next_transition == self.transitions.len(),
            first_update: self.first_update,
            settled_stop_tick,
        }) {
            self.last_observation_tick = tick;
            self.last_observation = current.clone();
            return RouteStatus::Completed;
        }
        self.last_observation_tick = tick;
        self.last_observation = current.clone();
        self.active_status()
    }

    #[must_use]
    pub fn poll_command(&self, current_tick: Tick, settled_stop_tick: Option<Tick>) -> RouteStatus {
        let Some(transition) = self.transitions.get(self.next_transition) else {
            return if current_tick >= self.certificate.survive_until() {
                RouteStatus::Completed
            } else {
                self.active_status()
            };
        };
        match current_tick.cmp(&transition.command.admission_tick) {
            std::cmp::Ordering::Less => self.active_status(),
            std::cmp::Ordering::Greater => RouteStatus::Invalidated,
            std::cmp::Ordering::Equal => {
                if transition.required_stop_snapshot.is_some_and(|required| {
                    self.last_observation_tick < required
                        || settled_stop_tick.is_none_or(|settled| settled < required)
                }) {
                    RouteStatus::Invalidated
                } else {
                    RouteStatus::Submit(transition.command)
                }
            }
        }
    }

    pub fn commit_command(&mut self, command: RouteCommand) -> bool {
        let Some(expected) = self.transitions.get(self.next_transition) else {
            return false;
        };
        if expected.command != command {
            return false;
        }
        self.next_transition += 1;
        self.movement = command.movement;
        true
    }

    fn active_status(&self) -> RouteStatus {
        match self.movement {
            MovementIntent::Stop => RouteStatus::Wait,
            MovementIntent::Direction(_) => RouteStatus::Continue,
        }
    }
}

fn transitions(
    certificate: &RouteCertificate,
    first_update: Tick,
) -> Result<Vec<Transition>, PolicyError> {
    let lane = certificate
        .lane(first_update)
        .ok_or(PolicyError::InvalidObservation)?;
    let mut transitions = Vec::with_capacity(certificate.steps().len().saturating_mul(2));
    let mut previous_stop = None;
    for (index, step) in certificate.steps().iter().copied().enumerate() {
        let step_index = u8::try_from(index).map_err(|_| PolicyError::InvalidObservation)?;
        let start = lane
            .step_start(index)
            .ok_or(PolicyError::InvalidObservation)?;
        match step {
            RouteStep::Move { direction, .. } => {
                push_transition(
                    &mut transitions,
                    start,
                    MovementIntent::Direction(direction),
                    step_index,
                    previous_stop,
                )?;
                let stop = lane
                    .stop_update(index)
                    .ok_or(PolicyError::InvalidObservation)?;
                push_transition(
                    &mut transitions,
                    stop,
                    MovementIntent::Stop,
                    step_index,
                    None,
                )?;
                previous_stop = Some(stop);
            }
            RouteStep::Wait { .. } if index == 0 => {
                push_transition(
                    &mut transitions,
                    start,
                    MovementIntent::Stop,
                    step_index,
                    None,
                )?;
                previous_stop = Some(
                    lane.stop_update(index)
                        .ok_or(PolicyError::InvalidObservation)?,
                );
            }
            RouteStep::Wait { .. } => {}
        }
    }
    Ok(transitions)
}

fn push_transition(
    transitions: &mut Vec<Transition>,
    update: Tick,
    movement: MovementIntent,
    step_index: u8,
    required_stop_snapshot: Option<Tick>,
) -> Result<(), PolicyError> {
    let admission_tick = update
        .0
        .checked_sub(1)
        .map(Tick)
        .ok_or(PolicyError::InvalidObservation)?;
    if transitions
        .last()
        .is_some_and(|previous| previous.command.admission_tick >= admission_tick)
    {
        return Err(PolicyError::InvalidObservation);
    }
    transitions.push(Transition {
        command: RouteCommand {
            movement,
            admission_tick,
            step_index,
        },
        required_stop_snapshot,
    });
    Ok(())
}

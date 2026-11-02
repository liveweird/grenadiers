use crate::cpu::routing::PlantPresence;
use crate::cpu::{Observation, WorkBudget};
use crate::game::Tick;

use super::compatibility::{actor, plant_actor_compatible, visible_rivals_allow_segment};
use super::{CommittedRoute, RouteStatus, WORK_ALLOWANCE};

/// Probes the fresh contact and Teleporter obligations for a sealed stationary
/// continuation suffix. This grants no authority and is intentionally limited
/// to the zero-residual `Clear` contract.
#[doc(hidden)]
pub fn stationary_suffix_preflight(
    observation: &Observation,
    safe_until: Tick,
    hold_ticks: u8,
    work: &mut WorkBudget,
) -> Result<(), &'static str> {
    let envelope = crate::cpu::lair_contact::MobContactEnvelope::new(observation, work)
        .map_err(|_| "contact-construction")?;
    if !crate::cpu::lair_contact::teleport::is_clear(
        &envelope,
        observation,
        safe_until,
        hold_ticks,
        work,
    )
    .map_err(|_| "teleporter-work")?
    {
        return Err("teleporter");
    }
    Ok(())
}

/// Read-only witness classification for an already committed route.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteInvalidator {
    NonMonotonicObservation,
    Compatibility,
    PlantScene,
    PlantActor,
    ContactCompatibility,
    PunchCompatibility,
    ContactDeadline,
    MissingExpectedPose,
    StormMismatch,
    SpikePitMismatch,
    MissingActor,
    PoseMismatch,
    RivalCollision,
    IncompleteAtSurvivalDeadline,
}

impl CommittedRoute {
    /// Returns the first predicate that `observe_with_stop` would reject.
    #[doc(hidden)]
    pub fn first_invalidator(&self, current: &Observation) -> Option<RouteInvalidator> {
        let tick = current.parts().tick;
        if tick <= self.last_observation_tick {
            return Some(RouteInvalidator::NonMonotonicObservation);
        }
        if self.has_contact_authority() && tick >= self.certificate.survive_until() {
            return completion_failure(self, current);
        }
        if let Some(plant) = self.plant.as_ref().filter(|plant| plant.contact.is_none()) {
            let presence = if plant.presence == PlantPresence::Awaiting {
                let Ok(presence) = plant.certificate.confirm(&self.basis, current) else {
                    return Some(RouteInvalidator::PlantScene);
                };
                presence
            } else {
                plant.presence
            };
            if !plant
                .certificate
                .matches_scene(&self.basis, current, presence)
            {
                return Some(RouteInvalidator::PlantScene);
            }
            if !plant_actor_compatible(&self.basis, current, &plant.certificate, presence) {
                return Some(RouteInvalidator::PlantActor);
            }
        } else if !self.clone().scene_matches(current) {
            return Some(if self.punch.is_some() {
                RouteInvalidator::PunchCompatibility
            } else if self
                .plant
                .as_ref()
                .is_some_and(|plant| plant.contact.is_some())
            {
                RouteInvalidator::ContactCompatibility
            } else {
                RouteInvalidator::Compatibility
            });
        }
        if tick >= self.certificate.survive_until() {
            return completion_failure(self, current);
        }
        if self.plant.as_ref().is_some_and(|plant| {
            plant.contact.as_ref().is_some_and(|proof| {
                tick.0
                    .checked_add(1)
                    .map(Tick)
                    .is_none_or(|next| next >= proof.safe_until())
            })
        }) {
            return Some(RouteInvalidator::ContactDeadline);
        }
        let Some(lane) = self.certificate.lane(self.first_update) else {
            return Some(RouteInvalidator::MissingExpectedPose);
        };
        let Some(expected) = lane.pose_at(tick) else {
            return Some(RouteInvalidator::MissingExpectedPose);
        };
        if !crate::cpu::routing::storm::observation_fits(
            &self.basis,
            current,
            lane.storm_deadlines_at(tick),
        ) {
            return Some(RouteInvalidator::StormMismatch);
        }
        if current.parts().rules.version == 10
            && !lane
                .spike_pits_match_at(current, &mut WorkBudget::new(WORK_ALLOWANCE))
                .unwrap_or(false)
        {
            return Some(RouteInvalidator::SpikePitMismatch);
        }
        let Some(current_actor) = actor(current) else {
            return Some(RouteInvalidator::MissingActor);
        };
        if expected != (current_actor.position, current_actor.heading) {
            return Some(RouteInvalidator::PoseMismatch);
        }
        if let Some(next) = tick
            .0
            .checked_add(1)
            .map(Tick)
            .and_then(|next| lane.pose_at(next))
            && !visible_rivals_allow_segment(current, current_actor.position, next.0)
        {
            return Some(RouteInvalidator::RivalCollision);
        }
        None
    }
}

fn completion_failure(route: &CommittedRoute, current: &Observation) -> Option<RouteInvalidator> {
    let mut candidate = route.clone();
    (candidate.finish(current) == RouteStatus::Invalidated)
        .then_some(RouteInvalidator::IncompleteAtSurvivalDeadline)
}

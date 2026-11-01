use crate::game::geometry::MovementSegment;
use crate::game::{EntityId, Position, Tick};
use crate::room::{LairConfig, RespawnLairConfig};

use super::routing::{RouteCertificate, RouteFailure};
use super::{Observation, WorkBudget};

#[cfg(test)]
#[path = "lair_contact_blocked_birth_parity_tests.rs"]
mod blocked_birth_parity_tests;
#[cfg(test)]
#[path = "lair_contact_connectivity_tests.rs"]
mod connectivity_tests;
#[path = "lair_contact_sources.rs"]
mod contact_sources;
#[cfg(test)]
#[path = "lair_contact_late_farm_fixture.rs"]
mod late_farm_fixture;
#[cfg(test)]
#[path = "lair_contact_late_farm_tests.rs"]
mod late_farm_tests;
#[cfg(test)]
#[path = "lair_contact_v138_farm_witness_fixture.rs"]
mod v138_farm_witness_fixture;
#[cfg(test)]
#[path = "lair_contact_v138_farm_witness_tests.rs"]
mod v138_farm_witness_tests;
#[cfg(test)]
#[path = "lair_contact_v139_teleporter_fixture.rs"]
pub(crate) mod v139_teleporter_fixture;
#[cfg(test)]
#[path = "lair_contact_v139_teleporter_tests.rs"]
mod v139_teleporter_tests;
#[cfg(test)]
use crate::game::types::HALF_FOOTPRINT;
use contact_sources::position_fits;
pub(crate) use contact_sources::{distance_to_interval, source_reaches_segment, travel_units};
#[cfg(test)]
#[path = "lair_contact_physical_fixture.rs"]
mod physical_fixture;
#[cfg(test)]
#[path = "lair_contact_physical_tests.rs"]
mod physical_tests;
#[path = "lair_contact_teleport.rs"]
pub(crate) mod teleport;
#[path = "lair_contact_topology.rs"]
mod topology;
#[cfg(test)]
#[path = "lair_contact_topology_tests.rs"]
mod topology_tests;
#[cfg(test)]
pub(crate) use topology::blocked_birth_binding_for_test;
pub(crate) use topology::{PLANT_CONTACT_RENEWAL_ALLOWANCE, PlantContactProof};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ContactEnvelopeError {
    Unsupported,
    InvalidTime,
    WorkExhausted,
    Capacity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SourceKind {
    Existing { id: EntityId, lair: u8 },
    PossibleSpawn { lair: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Source {
    pub(crate) kind: SourceKind,
    pub(crate) origin: Position,
    pub(crate) begins_at: Tick,
}

/// Finite wall-agnostic envelopes for at most 16 mobs and eight spawn origins.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MobContactEnvelope {
    world: super::WorldId,
    cpu_seat: crate::game::SeatId,
    cpu_id: EntityId,
    last_teleport_tick: Option<Tick>,
    basis_tick: Tick,
    config: LairIdentity,
    sources: Vec<Source>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LairIdentity {
    Legacy(LairConfig),
    Respawn(RespawnLairConfig),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MobContactLease {
    envelope: MobContactEnvelope,
    safe_until: Tick,
}

impl MobContactLease {
    pub(crate) const fn new(envelope: MobContactEnvelope, safe_until: Tick) -> Self {
        Self {
            envelope,
            safe_until,
        }
    }

    pub(crate) fn certify_route(
        observation: &Observation,
        route: &RouteCertificate,
        work: &mut WorkBudget,
    ) -> Result<Self, RouteFailure> {
        let envelope = MobContactEnvelope::new(observation, work).map_err(route_failure)?;
        let basis = route.basis_tick();
        let last = route
            .survive_until()
            .0
            .checked_sub(1)
            .ok_or(RouteFailure::InvalidRequest)?;
        let first_route_tick = basis.0.checked_add(1).ok_or(RouteFailure::InvalidRequest)?;
        for first in route.start_window().start.0..route.start_window().end.0 {
            let lane = route
                .lane(Tick(first))
                .ok_or(RouteFailure::InvalidRequest)?;
            let mut previous = lane.pose_at(basis).ok_or(RouteFailure::InvalidRequest)?.0;
            let mut stationary_end = None;
            for tick in first_route_tick..=last {
                let tick = Tick(tick);
                let current = lane.pose_at(tick).ok_or(RouteFailure::InvalidRequest)?.0;
                if current == previous {
                    stationary_end = Some(tick);
                } else {
                    if let Some(end) = stationary_end.take()
                        && !envelope
                            .stationary_is_clear_until(previous, end, work)
                            .map_err(route_failure)?
                    {
                        return Err(RouteFailure::NoRoute);
                    }
                    if !envelope
                        .sweep_is_clear(tick, &[MovementSegment::new(previous, current)], work)
                        .map_err(route_failure)?
                    {
                        return Err(RouteFailure::NoRoute);
                    }
                }
                previous = current;
            }
            if let Some(end) = stationary_end
                && !envelope
                    .stationary_is_clear_until(previous, end, work)
                    .map_err(route_failure)?
            {
                return Err(RouteFailure::NoRoute);
            }
        }
        Ok(Self::new(envelope, route.survive_until()))
    }

    #[must_use]
    pub const fn safe_until(&self) -> Tick {
        self.safe_until
    }

    pub fn observation_fits(&self, current: &Observation, work: &mut WorkBudget) -> bool {
        matches!(
            self.envelope.contains(current, self.safe_until, work),
            Ok(true)
        )
    }
}

fn route_failure(error: ContactEnvelopeError) -> RouteFailure {
    match error {
        ContactEnvelopeError::WorkExhausted => RouteFailure::WorkExhausted,
        ContactEnvelopeError::Capacity => RouteFailure::Capacity,
        ContactEnvelopeError::Unsupported | ContactEnvelopeError::InvalidTime => {
            RouteFailure::InvalidRequest
        }
    }
}

impl MobContactEnvelope {
    pub(crate) fn new(
        observation: &Observation,
        work: &mut WorkBudget,
    ) -> Result<Self, ContactEnvelopeError> {
        let (config, sources) = contact_sources::build(observation, work)?;
        let actor = observation
            .parts()
            .actors
            .iter()
            .find_map(|actor| match actor {
                super::ObservedActor::Visible(actor)
                    if actor.seat == observation.parts().recipient =>
                {
                    Some(actor)
                }
                _ => None,
            })
            .ok_or(ContactEnvelopeError::Unsupported)?;
        Ok(Self {
            world: observation.parts().world,
            cpu_seat: actor.seat,
            cpu_id: actor.id,
            last_teleport_tick: actor.last_teleport_tick,
            basis_tick: observation.parts().tick,
            config,
            sources,
        })
    }

    pub(crate) fn sources(&self) -> &[Source] {
        &self.sources
    }

    /// Checks actual piecewise actor motion at one update after mob movement.
    pub(crate) fn sweep_is_clear(
        &self,
        tick: Tick,
        segments: &[MovementSegment],
        work: &mut WorkBudget,
    ) -> Result<bool, ContactEnvelopeError> {
        if tick <= self.basis_tick {
            return Err(ContactEnvelopeError::InvalidTime);
        }
        take_work(
            work,
            self.sources
                .len()
                .checked_mul(segments.len())
                .ok_or(ContactEnvelopeError::Capacity)?,
        )?;
        for source in &self.sources {
            if tick < source.begins_at {
                continue;
            }
            let travel = travel_units(source.begins_at, tick)?;
            if segments
                .iter()
                .any(|segment| source_reaches_segment(*source, travel, *segment))
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Stationary reachability is monotone, so the interval coalesces to its end.
    pub(crate) fn stationary_is_clear_until(
        &self,
        position: Position,
        end: Tick,
        work: &mut WorkBudget,
    ) -> Result<bool, ContactEnvelopeError> {
        self.sweep_is_clear(end, &[MovementSegment::new(position, position)], work)
    }

    /// Allows ordinary motion/death and predicted births while rejecting facts
    /// outside the original finite envelope or beyond the lease deadline.
    pub(crate) fn contains(
        &self,
        current: &Observation,
        valid_until: Tick,
        work: &mut WorkBudget,
    ) -> Result<bool, ContactEnvelopeError> {
        if current.parts().world != self.world
            || current.parts().recipient != self.cpu_seat
            || current.parts().tick < self.basis_tick
            || current.parts().tick >= valid_until
        {
            return Ok(false);
        }
        let actor_matches = current.parts().actors.iter().any(|actor| {
            matches!(actor, super::ObservedActor::Visible(actor)
                if actor.seat == self.cpu_seat && actor.id == self.cpu_id
                    && actor.last_teleport_tick == self.last_teleport_tick)
        });
        if !actor_matches {
            return Ok(false);
        }
        let (config, mobs) = match self.config {
            LairIdentity::Legacy(expected) => {
                let Some(lairs) = current.lairs() else {
                    return Ok(false);
                };
                if lairs.config() != expected {
                    return Ok(false);
                }
                (LairIdentity::Legacy(lairs.config()), lairs.mobs())
            }
            LairIdentity::Respawn(expected) => {
                let Some(lairs) = current.respawn_lairs() else {
                    return Ok(false);
                };
                if lairs.config() != expected {
                    return Ok(false);
                }
                (LairIdentity::Respawn(lairs.config()), lairs.mobs())
            }
        };
        if config != self.config {
            return Ok(false);
        }
        take_work(
            work,
            mobs.len()
                .checked_mul(self.sources.len())
                .ok_or(ContactEnvelopeError::Capacity)?,
        )?;
        for mob in mobs {
            let existing = self.sources.iter().find(
                |source| matches!(source.kind, SourceKind::Existing { id, .. } if id == mob.id),
            );
            let fits = existing.map_or_else(
                || {
                    self.sources.iter().any(|source| {
                        matches!(
                            source.kind,
                            SourceKind::PossibleSpawn { lair } if lair == mob.lair
                        ) && position_fits(*source, current.parts().tick, mob.position)
                            .unwrap_or(false)
                    })
                },
                |source| {
                    matches!(
                        source.kind,
                        SourceKind::Existing { lair, .. } if lair == mob.lair
                    ) && position_fits(*source, current.parts().tick, mob.position).unwrap_or(false)
                },
            );
            if !fits {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// Lets a queued Easy driver suppress an optional pause near NPC/spawn danger.
#[must_use]
pub fn has_urgent_lair_threat(
    observation: &Observation,
    through: Tick,
    work: &mut WorkBudget,
) -> bool {
    let Some(actor) = observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            super::ObservedActor::Visible(actor) if actor.seat == observation.parts().recipient => {
                Some(actor)
            }
            _ => None,
        })
    else {
        return true;
    };
    let Ok(envelope) = MobContactEnvelope::new(observation, work) else {
        return true;
    };
    !matches!(
        envelope.stationary_is_clear_until(actor.position, through, work),
        Ok(true)
    )
}

fn take_work(work: &mut WorkBudget, units: usize) -> Result<(), ContactEnvelopeError> {
    work.try_take(u32::try_from(units).map_err(|_| ContactEnvelopeError::Capacity)?)
        .map_err(|_| ContactEnvelopeError::WorkExhausted)
}

#[cfg(test)]
#[path = "lair_contact_tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
#[path = "lair_contact_rules_v9_tests.rs"]
mod rules_v9_tests;

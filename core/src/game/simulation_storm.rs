use super::{State, StepError};
use crate::game::geometry::{MovementSegment, MovementTrace};
use crate::game::storm_geometry::{footprints_overlap, swept_hits_position};
use crate::game::types::HALF_FOOTPRINT;
use crate::game::{STORM_CLOUD_HALF_EXTENT, STORM_SLOW_TICKS, Tick};

impl State {
    pub(super) fn expire_storm_slow(&mut self, tick: Tick) {
        for actor in &mut self.actors {
            if actor.storm_slow_until.is_some_and(|expiry| expiry <= tick) {
                actor.storm_slow_until = None;
            }
        }
    }

    pub(super) fn preflight_storm_slow(&self, tick: Tick) -> Result<Option<Tick>, StepError> {
        if self.storm_cloud.is_none() {
            return Ok(None);
        }
        tick.0
            .checked_add(STORM_SLOW_TICKS)
            .map(Tick)
            .map(Some)
            .ok_or(StepError::StormSlowExpiryOverflow)
    }

    pub(super) fn advance_storm_cloud(&mut self) -> Option<MovementSegment> {
        self.storm_cloud.as_mut().map(|cloud| cloud.advance())
    }

    pub(super) fn storm_contacts_at_start(&self, cloud: Option<MovementSegment>) -> Vec<bool> {
        self.actors
            .iter()
            .map(|actor| {
                actor.alive
                    && cloud.is_some_and(|segment| {
                        swept_hits_position(
                            segment,
                            STORM_CLOUD_HALF_EXTENT,
                            actor.position,
                            HALF_FOOTPRINT,
                        )
                    })
            })
            .collect()
    }

    pub(super) fn include_storm_movement_contacts(
        &self,
        contacts: &mut [bool],
        traces: &[MovementTrace],
        forced: &[super::contract::ForcedMovement],
    ) {
        let Some(cloud) = self.storm_cloud() else {
            return;
        };
        for (index, trace) in traces.iter().enumerate() {
            if self.actors[index].alive
                && trace.segments().any(|segment| {
                    swept_hits_position(
                        segment,
                        HALF_FOOTPRINT,
                        cloud.position,
                        STORM_CLOUD_HALF_EXTENT,
                    )
                })
            {
                contacts[index] = true;
            }
        }
        for movement in forced {
            let index = movement.seat.index();
            if self.actors[index].alive
                && swept_hits_position(
                    movement.segment,
                    HALF_FOOTPRINT,
                    cloud.position,
                    STORM_CLOUD_HALF_EXTENT,
                )
            {
                contacts[index] = true;
            }
        }
    }

    pub(super) fn include_storm_endpoint_contacts(&self, contacts: &mut [bool]) {
        let Some(cloud) = self.storm_cloud() else {
            return;
        };
        for (contact, actor) in contacts.iter_mut().zip(&self.actors) {
            if actor.alive
                && footprints_overlap(
                    actor.position,
                    HALF_FOOTPRINT,
                    cloud.position,
                    STORM_CLOUD_HALF_EXTENT,
                )
            {
                *contact = true;
            }
        }
    }

    pub(super) fn apply_storm_contacts(&mut self, contacts: &[bool], expiry: Option<Tick>) {
        let Some(expiry) = expiry else { return };
        for (actor, &contact) in self.actors.iter_mut().zip(contacts) {
            if actor.alive && contact {
                actor.storm_slow_until = Some(expiry);
            }
        }
    }

    pub(super) fn clear_storm_slow(&mut self) {
        for actor in &mut self.actors {
            actor.storm_slow_until = None;
        }
    }
}

//! Contact authority for enabled and explicitly disabled rules-10 lairs.

use crate::cpu::lair_contact::{ContactEnvelopeError, MobContactEnvelope, teleport};
use crate::cpu::{Observation, ObservedActor, WorkBudget};
use crate::game::{EntityId, MovementSegment, SeatId, Tick};

#[derive(Clone)]
pub(super) enum ContinuationContact {
    Enabled(MobContactEnvelope),
    Disabled(DisabledContact),
}

#[derive(Clone)]
pub(super) struct DisabledContact {
    world: crate::cpu::WorldId,
    cpu_seat: SeatId,
    cpu_id: EntityId,
    last_teleport_tick: Option<Tick>,
    basis_tick: Tick,
}

impl ContinuationContact {
    pub(super) fn new(
        observation: &Observation,
        work: &mut WorkBudget,
    ) -> Result<Self, ContactEnvelopeError> {
        if observation.respawn_lairs().is_some() {
            return MobContactEnvelope::new(observation, work).map(Self::Enabled);
        }
        if observation.parts().rules.version != 10 || observation.lairs().is_some() {
            return Err(ContactEnvelopeError::Unsupported);
        }
        take(work, observation.parts().actors.len())?;
        let actor = recipient(observation).ok_or(ContactEnvelopeError::Unsupported)?;
        Ok(Self::Disabled(DisabledContact {
            world: observation.parts().world,
            cpu_seat: actor.seat,
            cpu_id: actor.id,
            last_teleport_tick: actor.last_teleport_tick,
            basis_tick: observation.parts().tick,
        }))
    }

    pub(super) fn contains(
        &self,
        current: &Observation,
        valid_until: Tick,
        work: &mut WorkBudget,
    ) -> Result<bool, ContactEnvelopeError> {
        match self {
            Self::Enabled(envelope) => envelope.contains(current, valid_until, work),
            Self::Disabled(contact) => contact.contains(current, valid_until, work),
        }
    }

    pub(super) fn contains_with_expected_marker(
        &self,
        current: &Observation,
        valid_until: Tick,
        expected_marker: Option<Tick>,
        work: &mut WorkBudget,
    ) -> Result<bool, ContactEnvelopeError> {
        match self {
            Self::Enabled(envelope) => {
                envelope.contains_with_expected_marker(current, valid_until, expected_marker, work)
            }
            Self::Disabled(contact) => {
                contact.contains_with_expected_marker(current, valid_until, expected_marker, work)
            }
        }
    }

    pub(super) fn sweep_is_clear(
        &self,
        tick: Tick,
        segments: &[MovementSegment],
        work: &mut WorkBudget,
    ) -> Result<bool, ContactEnvelopeError> {
        match self {
            Self::Enabled(envelope) => envelope.sweep_is_clear(tick, segments, work),
            Self::Disabled(contact) => {
                let _ = (segments, work);
                if tick <= contact.basis_tick {
                    return Err(ContactEnvelopeError::InvalidTime);
                }
                Ok(true)
            }
        }
    }

    pub(super) fn stationary_is_clear_until(
        &self,
        position: crate::game::Position,
        end: Tick,
        work: &mut WorkBudget,
    ) -> Result<bool, ContactEnvelopeError> {
        self.sweep_is_clear(end, &[MovementSegment::new(position, position)], work)
    }

    pub(super) fn teleport_is_clear(
        &self,
        observation: &Observation,
        safe_until: Tick,
        hold_ticks: u8,
        work: &mut WorkBudget,
    ) -> Result<bool, ContactEnvelopeError> {
        match self {
            Self::Enabled(envelope) => {
                teleport::is_clear(envelope, observation, safe_until, hold_ticks, work)
            }
            Self::Disabled(_) => {
                teleport::is_clear_sources(&[], observation, safe_until, hold_ticks, work)
            }
        }
    }

    pub(super) const fn is_enabled(&self) -> bool {
        matches!(self, Self::Enabled(_))
    }
}

impl DisabledContact {
    fn contains(
        &self,
        current: &Observation,
        valid_until: Tick,
        work: &mut WorkBudget,
    ) -> Result<bool, ContactEnvelopeError> {
        self.contains_with_expected_marker(current, valid_until, self.last_teleport_tick, work)
    }

    fn contains_with_expected_marker(
        &self,
        current: &Observation,
        valid_until: Tick,
        expected_marker: Option<Tick>,
        work: &mut WorkBudget,
    ) -> Result<bool, ContactEnvelopeError> {
        if current.parts().world != self.world
            || current.parts().recipient != self.cpu_seat
            || current.parts().tick < self.basis_tick
            || current.parts().tick >= valid_until
            || current.parts().rules.version != 10
            || current.respawn_lairs().is_some()
            || current.lairs().is_some()
        {
            return Ok(false);
        }
        take(work, current.parts().actors.len())?;
        Ok(recipient(current).is_some_and(|actor| {
            actor.seat == self.cpu_seat
                && actor.id == self.cpu_id
                && actor.last_teleport_tick == expected_marker
        }))
    }
}

fn recipient(observation: &Observation) -> Option<&crate::cpu::VisibleActor> {
    observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == observation.parts().recipient => {
                Some(actor)
            }
            _ => None,
        })
}

fn take(work: &mut WorkBudget, units: usize) -> Result<(), ContactEnvelopeError> {
    work.try_take(u32::try_from(units).map_err(|_| ContactEnvelopeError::Capacity)?)
        .map_err(|_| ContactEnvelopeError::WorkExhausted)
}

#[cfg(test)]
mod expected_marker_tests {
    use crate::cpu::lair_contact::v139_teleporter_fixture::captured_tick_6121;

    use super::*;

    #[test]
    fn sealed_marker_delegation_preserves_result_and_work() {
        let observation = captured_tick_6121();
        let mut construction = WorkBudget::new(65_536);
        let contact = ContinuationContact::new(&observation, &mut construction).unwrap();
        let marker = recipient(&observation).unwrap().last_teleport_tick;
        let mut old_work = WorkBudget::new(65_536);
        let mut explicit_work = WorkBudget::new(65_536);
        assert_eq!(
            contact.contains(&observation, Tick(6_139), &mut old_work),
            contact.contains_with_expected_marker(
                &observation,
                Tick(6_139),
                marker,
                &mut explicit_work,
            )
        );
        assert_eq!(old_work.spent(), explicit_work.spent());
    }
}

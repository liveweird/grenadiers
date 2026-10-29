use crate::cpu::policy::PolicyError;
use crate::cpu::routing::PulseLease;
use crate::cpu::{MemoryKey, MovementIntent, Observation};
use crate::game::Tick;

use super::MAX_PLAN_AGE_TICKS;
use super::compatibility::{actor, compatible};
use super::origin::{PlantEscapeOrigin, valid_origin};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PulseStatus {
    Continue,
    Wait,
    Completed,
    Invalidated,
}

#[derive(Clone, Debug)]
pub struct CommittedPulse {
    lease: PulseLease,
    basis: Observation,
    first_execution: Tick,
}

impl CommittedPulse {
    pub fn admit(
        lease: PulseLease,
        basis: Observation,
        current: &Observation,
        admission_tick: Tick,
    ) -> Result<Self, PolicyError> {
        let first_execution = admission_tick
            .0
            .checked_add(1)
            .map(Tick)
            .ok_or(PolicyError::InvalidObservation)?;
        let age = admission_tick
            .0
            .checked_sub(basis.parts().tick.0)
            .ok_or(PolicyError::InvalidObservation)?;
        let current_actor = actor(current).ok_or(PolicyError::InvalidObservation)?;
        let storm_deadlines = if current.parts().tick == admission_tick {
            lease.storm_deadlines_after(first_execution, 0)
        } else {
            crate::cpu::routing::storm::stationary_deadlines_at(&basis, current.parts().tick)
        };
        if age > MAX_PLAN_AGE_TICKS
            || current.parts().tick > admission_tick
            || lease.basis_tick() != basis.parts().tick
            || lease.key()
                != (MemoryKey {
                    world: basis.parts().world,
                    cpu_seat: basis.parts().recipient,
                })
            || !lease.start_window().contains(first_execution)
            || !compatible(&basis, current)
            || !crate::cpu::routing::storm::observation_fits(&basis, current, storm_deadlines)
            || !lease.bracelet_guard_allows(&basis)
            || !lease.bracelet_guard_allows(current)
            || !lease.matches_initial_pose(current_actor.position, current_actor.heading)
        {
            return Err(PolicyError::InvalidObservation);
        }
        Ok(Self {
            lease,
            basis,
            first_execution,
        })
    }

    #[must_use]
    pub fn validate(&self, current: &Observation, current_tick: Tick) -> PulseStatus {
        let Some(next_update) = current_tick.0.checked_add(1) else {
            return PulseStatus::Invalidated;
        };
        if current.parts().tick > current_tick
            || next_update >= self.lease.safe_until().0
            || !compatible(&self.basis, current)
            || !crate::cpu::routing::storm::observation_fits(
                &self.basis,
                current,
                self.lease.storm_deadlines_after(
                    self.first_execution,
                    current
                        .parts()
                        .tick
                        .0
                        .checked_sub(self.first_execution.0)
                        .map_or(0, |ticks| ticks.saturating_add(1))
                        .min(u64::from(self.lease.hold_ticks())) as u8,
                ),
            )
            || !self.lease.bracelet_guard_allows(current)
        {
            return PulseStatus::Invalidated;
        }
        let Some(actor) = actor(current) else {
            return PulseStatus::Invalidated;
        };
        let completed = current
            .parts()
            .tick
            .0
            .checked_sub(self.first_execution.0)
            .map_or(0, |ticks| ticks.saturating_add(1));
        let completed = completed.min(u64::from(self.lease.hold_ticks())) as u8;
        if !self
            .lease
            .pose_after(completed)
            .contains(actor.position, actor.heading)
        {
            return PulseStatus::Invalidated;
        }
        let stop_update = self.first_execution.0 + u64::from(self.lease.hold_ticks());
        if current.parts().tick.0 >= stop_update {
            PulseStatus::Completed
        } else if self.lease.movement() == MovementIntent::Stop || next_update >= stop_update {
            PulseStatus::Wait
        } else {
            PulseStatus::Continue
        }
    }

    #[must_use]
    pub const fn first_execution(&self) -> Tick {
        self.first_execution
    }

    #[must_use]
    pub const fn lease(&self) -> &PulseLease {
        &self.lease
    }

    #[must_use]
    pub fn retain_escape_origin(
        &self,
        origin: &PlantEscapeOrigin,
        current: &Observation,
        current_tick: Tick,
    ) -> Option<PlantEscapeOrigin> {
        let next_update = current_tick.0.checked_add(1)?;
        let origin_key = MemoryKey {
            world: origin.basis.parts().world,
            cpu_seat: origin.basis.parts().recipient,
        };
        if current.parts().tick > current_tick
            || next_update >= self.lease.safe_until().0
            || self.lease.key() != origin_key
            || self.basis.parts().world != origin_key.world
            || self.basis.parts().recipient != origin_key.cpu_seat
            || !valid_origin(origin, current)
        {
            return None;
        }
        let current_actor = actor(current)?;
        let completed = current
            .parts()
            .tick
            .0
            .checked_sub(self.first_execution.0)
            .map_or(0, |ticks| ticks.saturating_add(1))
            .min(u64::from(self.lease.hold_ticks())) as u8;
        self.lease
            .pose_after(completed)
            .contains(current_actor.position, current_actor.heading)
            .then(|| origin.with_checkpoint(current))
    }
}

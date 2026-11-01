//! Eligibility for replacing a live plant certificate from its terminal refuge.

use crate::cpu::routing::PlantPresence;
use crate::cpu::{BombKnowledge, MemoryKey, MovementIntent, Observation};
use crate::game::geometry::position_overlaps_cell;
use crate::game::{EntityId, Position, Tick};

use super::compatibility::actor;
use super::{CommittedRoute, ExecutionCertificate};

/// Opaque proof that an observed live plant route reached its confirmed refuge.
#[derive(Clone, Debug)]
pub struct TerminalRefugeHandoff {
    key: MemoryKey,
    route_basis_tick: Tick,
    route_first_update: Tick,
    bomb_id: EntityId,
    refuge: Position,
    checkpoint: Observation,
    old_survive_until: Tick,
}

impl TerminalRefugeHandoff {
    pub(super) fn new(
        key: MemoryKey,
        route_basis_tick: Tick,
        route_first_update: Tick,
        bomb_id: EntityId,
        refuge: Position,
        checkpoint: Observation,
        old_survive_until: Tick,
    ) -> Self {
        Self {
            key,
            route_basis_tick,
            route_first_update,
            bomb_id,
            refuge,
            checkpoint,
            old_survive_until,
        }
    }

    #[must_use]
    pub const fn key(&self) -> MemoryKey {
        self.key
    }

    #[must_use]
    pub const fn route_identity(&self) -> (Tick, Tick) {
        (self.route_basis_tick, self.route_first_update)
    }

    #[must_use]
    pub const fn bomb_id(&self) -> EntityId {
        self.bomb_id
    }

    #[must_use]
    pub const fn refuge(&self) -> Position {
        self.refuge
    }

    #[must_use]
    pub fn checkpoint(&self) -> &Observation {
        &self.checkpoint
    }

    #[must_use]
    pub const fn old_survive_until(&self) -> Tick {
        self.old_survive_until
    }

    /// Checks only the replacement certificate. Contact and usefulness remain
    /// controller admission requirements and are intentionally not inferred here.
    #[must_use]
    pub fn accepts_replacement(&self, replacement: &ExecutionCertificate) -> bool {
        let Some(plant) = replacement.plant() else {
            return false;
        };
        plant.route().key() == self.key
            && plant.route().basis_tick() == self.checkpoint.parts().tick
            && horizon_preserved(plant.route().survive_until(), self.old_survive_until)
            && matches!(
                plant.confirm(&self.checkpoint, &self.checkpoint),
                Ok(PlantPresence::Awaiting)
            )
    }
}

const fn horizon_preserved(candidate: Tick, old: Tick) -> bool {
    candidate.0 >= old.0
}

impl CommittedRoute {
    /// Reads the last successfully validated publication without observing it
    /// again. The installed route remains authoritative after this snapshot.
    #[must_use]
    pub fn terminal_refuge_handoff(
        &self,
        current: &Observation,
        settled_stop_tick: Option<Tick>,
    ) -> Option<TerminalRefugeHandoff> {
        if &self.last_observation != current
            || self.next_transition != self.transitions.len()
            || self.movement != MovementIntent::Stop
            || self.punch.is_some()
        {
            return None;
        }
        let plant = self.plant.as_ref()?;
        let PlantPresence::Present(bomb_id) = plant.presence else {
            return None;
        };
        let final_stop_update = self
            .transitions
            .last()
            .filter(|transition| transition.command.movement() == MovementIntent::Stop)?
            .command
            .admission_tick()
            .0
            .checked_add(1)
            .map(Tick)?;
        let settled = settled_stop_tick?;
        let actor = actor(current)?;
        let key = MemoryKey {
            world: current.parts().world,
            cpu_seat: current.parts().recipient,
        };
        let visible = matches!(&current.parts().bombs, BombKnowledge::Visible(bombs)
            if bombs.iter().any(|bomb| bomb.id == bomb_id));
        let enough_hold = current
            .parts()
            .tick
            .0
            .checked_add(super::MAX_PLAN_AGE_TICKS)
            .is_some_and(|admission| admission < self.certificate.survive_until().0);
        if key != self.certificate.key()
            || settled < final_stop_update
            || settled > current.parts().tick
            || position_overlaps_cell(actor.position, plant.certificate.placement().cell)
            || actor.live_bomb_count >= actor.bomb_capacity
            || !visible
            || !enough_hold
        {
            return None;
        }
        Some(TerminalRefugeHandoff::new(
            key,
            self.certificate.basis_tick(),
            self.first_update,
            bomb_id,
            actor.position,
            current.clone(),
            self.certificate.survive_until(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::horizon_preserved;
    use crate::game::Tick;

    #[test]
    fn replacement_horizon_must_not_shorten_the_installed_hold() {
        assert!(!horizon_preserved(Tick(99), Tick(100)));
        assert!(horizon_preserved(Tick(100), Tick(100)));
        assert!(horizon_preserved(Tick(101), Tick(100)));
    }
}

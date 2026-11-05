//! Eligibility for replacing a live plant certificate from its terminal refuge.

use crate::cpu::routing::PlantPresence;
use crate::cpu::{BombKnowledge, MemoryKey, MovementIntent, Observation};
use crate::game::geometry::position_overlaps_cell;
use crate::game::{EntityId, Position, Tick};

use super::compatibility::actor;
use super::{CommittedRoute, ExecutionCertificate};

/// Opaque proof that an observed live plant route reached its confirmed refuge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalRefugeHandoff {
    key: MemoryKey,
    route_basis_tick: Tick,
    route_first_update: Tick,
    bomb_id: EntityId,
    refuge: Position,
    checkpoint: Observation,
    old_survive_until: Tick,
    settled_stop_tick: Tick,
}

impl TerminalRefugeHandoff {
    pub(super) fn new(
        key: MemoryKey,
        route_identity: (Tick, Tick),
        bomb_id: EntityId,
        refuge: Position,
        checkpoint: Observation,
        old_survive_until: Tick,
        settled_stop_tick: Tick,
    ) -> Self {
        Self {
            key,
            route_basis_tick: route_identity.0,
            route_first_update: route_identity.1,
            bomb_id,
            refuge,
            checkpoint,
            old_survive_until,
            settled_stop_tick,
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

    #[must_use]
    pub const fn settled_stop_tick(&self) -> Tick {
        self.settled_stop_tick
    }

    pub(super) fn authorizes_combination_planning(
        &self,
        observation: &Observation,
        settled_stop_tick: Tick,
        work: &mut crate::cpu::WorkBudget,
    ) -> bool {
        charge_observation_equality(&self.checkpoint, observation, work)
            && self.checkpoint == *observation
            && self.settled_stop_tick == settled_stop_tick
    }

    pub(super) fn authorizes_owned_combination(
        &self,
        basis: &Observation,
        current: &Observation,
        work: &mut crate::cpu::WorkBudget,
    ) -> bool {
        let actor_scan = u32::try_from(basis.parts().actors.len());
        let bomb_scan = match &basis.parts().bombs {
            BombKnowledge::Visible(bombs) => u32::try_from(bombs.len()),
            BombKnowledge::Unavailable => Ok(1),
        };
        self.authorizes_combination_planning(basis, self.settled_stop_tick, work)
            && actor_scan.is_ok_and(|units| work.try_take(units).is_ok())
            && bomb_scan.is_ok_and(|units| work.try_take(units).is_ok())
            && current.parts().world == basis.parts().world
            && current.parts().recipient == basis.parts().recipient
            && self.key
                == MemoryKey {
                    world: basis.parts().world,
                    cpu_seat: basis.parts().recipient,
                }
            && actor(basis).is_some_and(|actor| actor.position == self.refuge)
            && matches!(&basis.parts().bombs, BombKnowledge::Visible(bombs)
                if bombs.len() == 1 && bombs[0].id == self.bomb_id)
    }

    pub(super) fn matches_sealed_charged(
        &self,
        other: &Self,
        work: &mut crate::cpu::WorkBudget,
    ) -> bool {
        charge_observation_equality(&self.checkpoint, &other.checkpoint, work) && self == other
    }

    pub(super) fn accepts_replacement_charged(
        &self,
        replacement: &ExecutionCertificate,
        work: &mut crate::cpu::WorkBudget,
    ) -> bool {
        let Some(plant) = replacement.plant() else {
            return false;
        };
        plant.route().key() == self.key
            && plant.route().basis_tick() == self.checkpoint.parts().tick
            && horizon_preserved(plant.route().survive_until(), self.old_survive_until)
            && plant.reactive_scene_candidate(
                &self.checkpoint,
                &self.checkpoint,
                PlantPresence::Awaiting,
                work,
            ) == Some(PlantPresence::Awaiting)
    }

    /// Checks only the replacement certificate. Contact and usefulness remain
    /// controller admission requirements and are intentionally not inferred here.
    #[must_use]
    pub fn accepts_replacement(&self, replacement: &ExecutionCertificate) -> bool {
        let Some(plant) = replacement.plant() else {
            return false;
        };
        plant.route().key() == self.key
            // Awaiting confirmation repeats this through the forecast header;
            // retain the explicit identity fence at this API boundary.
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
            (self.certificate.basis_tick(), self.first_update),
            bomb_id,
            actor.position,
            current.clone(),
            self.certificate.survive_until(),
            settled,
        ))
    }
}

fn charge_observation_equality(
    left: &Observation,
    right: &Observation,
    work: &mut crate::cpu::WorkBudget,
) -> bool {
    fn charge(observation: &Observation, work: &mut crate::cpu::WorkBudget) -> Option<()> {
        let parts = observation.parts();
        let bombs = match &parts.bombs {
            BombKnowledge::Unavailable => 1,
            BombKnowledge::Visible(bombs) => bombs.len(),
        };
        let lairs = observation.respawn_lairs().map_or(0, |value| {
            value.lairs().len().saturating_add(value.mobs().len())
        });
        let pits = observation
            .spike_pits()
            .map_or(0, |value| value.pits().len());
        let flat = parts
            .arena
            .terrain
            .len()
            .checked_add(parts.arena.blocks.len())?
            .checked_add(parts.actors.len())?
            .checked_add(bombs)?
            .checked_add(parts.flames.len())?
            .checked_add(parts.oil.len())?
            .checked_add(parts.pickups.len())?
            .checked_add(lairs)?
            .checked_add(pits)?
            .checked_add(8)?;
        work.try_take(u32::try_from(flat).ok()?).ok()?;
        for actor in &parts.actors {
            let effects = match actor {
                crate::cpu::ObservedActor::Visible(actor) => actor.effects.len(),
                crate::cpu::ObservedActor::Hidden { .. } => 0,
            };
            work.try_take(u32::try_from(effects).ok()?).ok()?;
        }
        Some(())
    }
    charge(left, work).is_some() && charge(right, work).is_some()
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

use crate::cpu::routing::{PlantCertificate, PlantPresence};
use crate::cpu::{BombKnowledge, MemoryKey, Observation, ObservedBomb, ObservedBombTrigger};
use crate::game::arena::Terrain;
use crate::game::geometry::position_overlaps_cell;
use crate::game::{Direction, EntityId, Position, Tick};

use super::CommittedRoute;
use super::compatibility::{actor, plant_actor_compatible};

impl CommittedRoute {
    #[must_use]
    pub fn escape_origin(&self, current: &Observation) -> Option<PlantEscapeOrigin> {
        if current.parts().tick < self.last_observation_tick {
            return None;
        }
        let plant = self.plant.as_ref()?;
        let PlantPresence::Present(actual_id) = plant.presence else {
            return None;
        };
        let expected = self
            .certificate
            .lane(self.first_update)?
            .pose_at(current.parts().tick)?;
        let current_actor = actor(current)?;
        (expected == (current_actor.position, current_actor.heading))
            .then(|| {
                PlantEscapeOrigin::new(
                    self.basis.clone(),
                    plant.certificate.clone(),
                    actual_id,
                    current,
                )
            })
            .flatten()
    }
}

/// Opaque proof that a confirmed certified plant still grants origin-exit grace.
#[derive(Clone)]
pub struct PlantEscapeOrigin {
    pub(super) basis: Observation,
    pub(super) certificate: PlantCertificate,
    pub(super) actual_id: EntityId,
    pub(super) checkpoint_tick: Tick,
    pub(super) checkpoint_position: Position,
    pub(super) checkpoint_heading: Direction,
}

impl PlantEscapeOrigin {
    #[must_use]
    pub fn retain_stopped(&self, current: &Observation) -> Option<Self> {
        let current_actor = actor(current)?;
        (current_actor.position == self.checkpoint_position
            && current_actor.heading == self.checkpoint_heading
            && valid_origin(self, current))
        .then(|| self.with_checkpoint(current))
    }

    #[must_use]
    pub fn same_origin(&self, other: &Self) -> bool {
        let left = self.certificate.placement();
        let right = other.certificate.placement();
        MemoryKey {
            world: self.basis.parts().world,
            cpu_seat: self.basis.parts().recipient,
        } == MemoryKey {
            world: other.basis.parts().world,
            cpu_seat: other.basis.parts().recipient,
        } && left == right
            && self.actual_id == other.actual_id
    }

    pub(super) fn new(
        basis: Observation,
        certificate: PlantCertificate,
        actual_id: EntityId,
        current: &Observation,
    ) -> Option<Self> {
        let current_actor = actor(current)?;
        let origin = Self {
            basis,
            certificate,
            actual_id,
            checkpoint_tick: current.parts().tick,
            checkpoint_position: current_actor.position,
            checkpoint_heading: current_actor.heading,
        };
        valid_origin(&origin, current).then_some(origin)
    }

    pub(super) fn with_checkpoint(&self, current: &Observation) -> Self {
        let current_actor = actor(current).expect("validated origin actor");
        Self {
            checkpoint_tick: current.parts().tick,
            checkpoint_position: current_actor.position,
            checkpoint_heading: current_actor.heading,
            ..self.clone()
        }
    }

    pub(super) const fn cell(&self) -> crate::game::CellId {
        self.certificate.placement().cell
    }

    pub(super) const fn bomb_id(&self) -> EntityId {
        self.actual_id
    }
}

pub(super) fn valid_origin(origin: &PlantEscapeOrigin, current: &Observation) -> bool {
    let parts = current.parts();
    let placement = origin.certificate.placement();
    if parts.world != origin.basis.parts().world
        || parts.tick < origin.checkpoint_tick
        || parts.recipient != origin.basis.parts().recipient
        || parts.rules.version != origin.basis.parts().rules.version
        || parts.rules.tick_rate != origin.basis.parts().rules.tick_rate
        || parts.rules.settings != origin.basis.parts().rules.settings
        || !plant_actor_compatible(
            &origin.basis,
            current,
            &origin.certificate,
            PlantPresence::Present(origin.actual_id),
        )
    {
        return false;
    }
    let Some(current_actor) = actor(current) else {
        return false;
    };
    let cell = placement.cell;
    let index = usize::from(cell.row) * usize::from(parts.arena.width) + usize::from(cell.column);
    let valid_floor = parts.arena.terrain.get(index) == Some(&Terrain::Floor)
        && !parts.arena.blocks.contains(&cell)
        && !parts.oil.iter().any(|oil| oil.cell == cell);
    let BombKnowledge::Visible(bombs) = &parts.bombs else {
        return false;
    };
    valid_floor
        && position_overlaps_cell(current_actor.position, cell)
        && bombs.iter().filter(|bomb| bomb.cell == cell).count() == 1
        && bombs
            .iter()
            .find(|bomb| bomb.id == origin.actual_id)
            .is_some_and(|bomb| matches_placement(bomb, placement, parts.tick))
}

fn matches_placement(
    bomb: &ObservedBomb,
    placement: &crate::cpu::routing::PlannedBomb,
    tick: Tick,
) -> bool {
    let remaining = placement
        .detonates_at
        .0
        .checked_sub(tick.0)
        .filter(|value| *value > 0)
        .and_then(|value| u16::try_from(value).ok());
    bomb.placed_at == placement.placed_at
        && bomb.cell == placement.cell
        && bomb.owner == placement.owner
        && bomb.range == placement.range
        && bomb.blast == placement.blast
        && matches!(
            bomb.trigger,
            ObservedBombTrigger::Timed {
                remaining_ticks,
                paused: false
            } if Some(remaining_ticks) == remaining
        )
}

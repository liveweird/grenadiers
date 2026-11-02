use crate::cpu::routing::{PlantCertificate, PlantPresence};
use crate::cpu::{BombKnowledge, MemoryKey, Observation};
use crate::game::arena::Terrain;
use crate::game::geometry::position_overlaps_cell;
use crate::game::{Direction, EntityId, Position, Tick};

use super::CommittedRoute;
use super::compatibility::{actor, plant_actor_compatible};

impl CommittedRoute {
    pub fn select_absent_plant(&mut self) -> bool {
        let Some(plant) = self.plant.as_mut() else {
            return false;
        };
        if matches!(plant.presence, PlantPresence::Present(_)) {
            return false;
        }
        plant.presence = PlantPresence::Absent;
        true
    }

    pub fn plant_presence(&self) -> Option<PlantPresence> {
        self.plant.as_ref().map(|plant| plant.presence)
    }

    #[must_use]
    pub fn escape_origin(&self, current: &Observation) -> Option<PlantEscapeOrigin> {
        if current.parts().tick < self.last_observation_tick {
            return None;
        }
        let plant = self.plant.as_ref()?;
        let actual_id = match plant.presence {
            PlantPresence::Present(id) => id,
            PlantPresence::Awaiting => plant
                .certificate
                .observed_placement_id(&self.basis, current)
                .ok()??,
            PlantPresence::Absent => return None,
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

    pub(super) fn continuation_hint(
        &self,
        work: &mut crate::cpu::WorkBudget,
    ) -> Option<(crate::game::CellId, Direction)> {
        let route = self.certificate.route();
        let lane_count = route
            .start_window()
            .end
            .0
            .checked_sub(route.start_window().start.0)?;
        let units = u32::try_from(route.steps().len())
            .ok()?
            .checked_add(u32::try_from(lane_count).ok()?)?
            .checked_add(1)?;
        work.try_take(units).ok()?;
        let first_axis = route.steps().iter().find_map(|step| match step {
            crate::cpu::routing::RouteStep::Move { direction, .. } => Some(*direction),
            crate::cpu::routing::RouteStep::Wait { .. } => None,
        })?;
        let final_tick = Tick(route.survive_until().0.checked_sub(1)?);
        let mut refuge = None;
        for first in route.start_window().start.0..route.start_window().end.0 {
            let cell = route.lane(Tick(first))?.pose_at(final_tick)?.0.cell()?;
            if refuge.is_some_and(|previous| previous != cell) {
                return None;
            }
            refuge = Some(cell);
        }
        Some((refuge?, first_axis))
    }
}

pub(super) fn valid_origin(origin: &PlantEscapeOrigin, current: &Observation) -> bool {
    let parts = current.parts();
    let placement = origin.certificate.placement();
    if parts.world != origin.basis.parts().world
        || parts.tick < origin.checkpoint_tick
        || parts.recipient != origin.basis.parts().recipient
        || parts.rules.version != origin.basis.parts().rules.version
        || parts.rules.punch_activation_policy != origin.basis.parts().rules.punch_activation_policy
        || parts.rules.tick_rate != origin.basis.parts().rules.tick_rate
        || parts.rules.settings != origin.basis.parts().rules.settings
        || parts.arena.width != origin.basis.parts().arena.width
        || parts.arena.height != origin.basis.parts().arena.height
        || parts.arena.terrain != origin.basis.parts().arena.terrain
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
    let valid_floor = parts
        .arena
        .index(cell)
        .and_then(|index| parts.arena.terrain.get(index))
        == Some(&Terrain::Floor)
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
            .is_some_and(|bomb| placement.matches_observed(bomb, parts.tick))
}

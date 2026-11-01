use crate::cpu::forecast::{ForecastEvolution, ForecastFailure};
use crate::cpu::{
    BombKnowledge, MemoryKey, MovementIntent, Observation, ObservedBomb, ObservedBombTrigger,
    WorkBudget,
};
use crate::game::{BlastKind, CellId, EntityId, SpecialWeapon, Tick};

use super::{RouteCertificate, RouteFailure, RouteLimits};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlantRequest {
    pub execution_tick: Tick,
    pub settled_stop_tick: Tick,
    pub first_movement: MovementIntent,
    pub limits: RouteLimits,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlannedBomb {
    pub placed_at: Tick,
    pub owner: EntityId,
    pub cell: CellId,
    pub range: u16,
    pub blast: BlastKind,
    pub detonates_at: Tick,
}

impl PlannedBomb {
    pub(crate) fn matches_observed(&self, bomb: &ObservedBomb, tick: Tick) -> bool {
        bomb.placed_at == self.placed_at
            && bomb.cell == self.cell
            && bomb.owner == self.owner
            && bomb.range == self.range
            && bomb.blast == self.blast
            && matches!(
                bomb.trigger,
                ObservedBombTrigger::Timed {
                    remaining_ticks,
                    paused: false,
                } if remaining_ticks > 0
                    && tick.0.checked_add(u64::from(remaining_ticks))
                        == Some(self.detonates_at.0)
            )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlantPresence {
    Awaiting,
    Absent,
    Present(EntityId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlantFailure {
    InvalidRequest,
    Unsupported,
    Forecast(ForecastFailure),
    Route(RouteFailure),
    SceneMismatch,
    AmbiguousPresence,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlantCertificate {
    key: MemoryKey,
    route: RouteCertificate,
    placement: PlannedBomb,
    hypothetical_id: EntityId,
    absent: ForecastEvolution,
    present: ForecastEvolution,
    weapon_before: Option<SpecialWeapon>,
}

impl PlantCertificate {
    pub(super) fn new(
        route: RouteCertificate,
        placement: PlannedBomb,
        hypothetical_id: EntityId,
        absent: ForecastEvolution,
        present: ForecastEvolution,
        weapon_before: Option<SpecialWeapon>,
    ) -> Self {
        Self {
            key: route.key(),
            route,
            placement,
            hypothetical_id,
            absent,
            present,
            weapon_before,
        }
    }

    #[must_use]
    pub const fn route(&self) -> &RouteCertificate {
        &self.route
    }

    #[must_use]
    pub const fn placement(&self) -> &PlannedBomb {
        &self.placement
    }

    #[must_use]
    pub const fn expected_weapon(&self, presence: PlantPresence) -> Option<SpecialWeapon> {
        match presence {
            PlantPresence::Awaiting | PlantPresence::Absent => self.weapon_before,
            PlantPresence::Present(_) => None,
        }
    }

    #[must_use]
    pub(crate) const fn weapon_before(&self) -> Option<SpecialWeapon> {
        self.weapon_before
    }

    pub fn confirm(
        &self,
        basis: &Observation,
        current: &Observation,
    ) -> Result<PlantPresence, PlantFailure> {
        if current.parts().tick < self.placement.placed_at {
            return self
                .absent
                .matches_scene(basis, current)
                .then_some(PlantPresence::Awaiting)
                .ok_or(PlantFailure::SceneMismatch);
        }
        if let Some(id) = self.observed_placement_id(basis, current)? {
            return self
                .matches_scene(basis, current, PlantPresence::Present(id))
                .then_some(PlantPresence::Present(id))
                .ok_or(PlantFailure::SceneMismatch);
        }
        self.matches_scene(basis, current, PlantPresence::Absent)
            .then_some(PlantPresence::Absent)
            .ok_or(PlantFailure::SceneMismatch)
    }

    pub(crate) fn observed_placement_id(
        &self,
        basis: &Observation,
        current: &Observation,
    ) -> Result<Option<EntityId>, PlantFailure> {
        if current.parts().tick < self.placement.placed_at {
            return Ok(None);
        }
        let candidates = self.matching_new_bombs(basis, current);
        match candidates.as_slice() {
            [] => Ok(None),
            [id] => Ok(Some(*id)),
            _ => Err(PlantFailure::AmbiguousPresence),
        }
    }

    #[must_use]
    pub fn matches_scene(
        &self,
        basis: &Observation,
        current: &Observation,
        presence: PlantPresence,
    ) -> bool {
        if self.key.world != basis.parts().world || self.key.cpu_seat != basis.parts().recipient {
            return false;
        }
        match presence {
            PlantPresence::Awaiting => {
                current.parts().tick < self.placement.placed_at
                    && self.absent.matches_scene(basis, current)
            }
            PlantPresence::Absent => self.absent.matches_scene(basis, current),
            PlantPresence::Present(id) => {
                let alive = !self
                    .present
                    .hypothetical_removed(self.hypothetical_id, current.parts().tick);
                (!alive || self.bomb_matches(current, id))
                    && self.present.matches_scene_with_hypothetical(
                        basis,
                        current,
                        self.hypothetical_id,
                        id,
                    )
            }
        }
    }

    pub(crate) fn contact_scene_candidate(
        &self,
        basis: &Observation,
        current: &Observation,
        established: PlantPresence,
        work: &mut WorkBudget,
    ) -> Option<PlantPresence> {
        let charge = self
            .absent
            .contact_scene_work(basis, current)?
            .max(self.present.contact_scene_work(basis, current)?);
        work.try_take(charge).ok()?;
        if established == PlantPresence::Awaiting {
            self.confirm(basis, current).ok()
        } else {
            self.matches_scene(basis, current, established)
                .then_some(established)
        }
    }

    pub(crate) fn contact_actor_work(&self) -> Option<u32> {
        Some(
            self.absent
                .contact_actor_work()?
                .max(self.present.contact_actor_work()?),
        )
    }

    #[must_use]
    pub fn expected_live_bombs(
        &self,
        owner: EntityId,
        tick: Tick,
        presence: PlantPresence,
    ) -> Option<u16> {
        match presence {
            PlantPresence::Awaiting | PlantPresence::Absent => {
                self.absent.expected_live_bombs(owner, tick)
            }
            PlantPresence::Present(_) => self.present.expected_live_bombs(owner, tick),
        }
    }

    fn matching_new_bombs(&self, basis: &Observation, current: &Observation) -> Vec<EntityId> {
        let (BombKnowledge::Visible(old), BombKnowledge::Visible(new)) =
            (&basis.parts().bombs, &current.parts().bombs)
        else {
            return Vec::new();
        };
        new.iter()
            .filter(|bomb| !old.iter().any(|known| known.id == bomb.id))
            .filter(|bomb| self.placement.matches_observed(bomb, current.parts().tick))
            .map(|bomb| bomb.id)
            .collect()
    }

    fn bomb_matches(&self, current: &Observation, id: EntityId) -> bool {
        let BombKnowledge::Visible(bombs) = &current.parts().bombs else {
            return false;
        };
        bombs.iter().any(|bomb| {
            bomb.id == id && self.placement.matches_observed(bomb, current.parts().tick)
        })
    }
}

impl From<ForecastFailure> for PlantFailure {
    fn from(value: ForecastFailure) -> Self {
        Self::Forecast(value)
    }
}

impl From<RouteFailure> for PlantFailure {
    fn from(value: RouteFailure) -> Self {
        Self::Route(value)
    }
}

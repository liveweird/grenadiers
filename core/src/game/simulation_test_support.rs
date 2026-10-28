use super::{Direction, State, StateError};
use crate::game::arena::Terrain;
use crate::game::bombs::{Bomb, BombTrigger};
use crate::game::effects::EffectKind;
use crate::game::pickups::{Pickup, PickupKind};
use crate::game::types::{CellId, EntityId, Position, SeatId, Tick};

impl State {
    pub(crate) fn apply_test_effect(&mut self, seat: SeatId, kind: EffectKind, at: Tick) {
        self.actors[seat.index()].effects.apply(kind, at).unwrap();
    }

    pub(crate) fn clear_test_effects(&mut self, seat: SeatId) {
        self.actors[seat.index()].effects.clear();
    }

    pub(crate) fn insert_test_flame(&mut self, cell: CellId, expires_at: Tick) {
        self.world.hazards.insert(cell, expires_at);
    }

    pub(crate) fn insert_test_pickup(&mut self, cell: CellId, kind: PickupKind) {
        self.world.pickups.exposed.insert(cell, Pickup::new(kind));
    }

    pub(crate) fn set_test_damage_grace(&mut self, seat: SeatId, received_at: Tick) {
        self.actors[seat.index()].damage_grace =
            Some(crate::game::DamageGrace::for_test(received_at));
    }

    pub(crate) fn set_test_actor_pose(
        &mut self,
        seat: SeatId,
        position: Position,
        heading: Direction,
    ) {
        self.actors[seat.index()].position = position;
        self.actors[seat.index()].heading = heading;
    }

    pub(crate) fn replace_test_actor_equipment(
        &mut self,
        seat: SeatId,
        equipment: crate::game::Equipment,
    ) {
        self.replace_actor_equipment(seat, equipment);
    }

    pub(crate) fn commit_test_teleport(&mut self, requests: impl Into<Vec<bool>>, tick: Tick) {
        self.commit_teleport(requests, tick);
    }

    pub(crate) fn insert_test_bomb(
        &mut self,
        id: EntityId,
        cell: CellId,
        owner: EntityId,
        range: u16,
        placed_at: Tick,
        trigger: BombTrigger,
    ) -> Result<(), StateError> {
        if self.actors.iter().any(|actor| actor.id == id) || self.world.bombs.contains_key(&id) {
            return Err(StateError::DuplicateEntityId(id));
        }
        if !self.actors.iter().any(|actor| actor.id == owner) {
            return Err(StateError::UnknownBombOwner(owner));
        }
        if self.world.arena.terrain(cell) != Some(Terrain::Floor)
            || self.world.arena.blocks().contains(&cell)
        {
            return Err(StateError::IllegalBombCell(cell));
        }
        if self.world.bombs.values().any(|bomb| bomb.cell == cell) {
            return Err(StateError::OccupiedBombCell(cell));
        }

        let bomb = Bomb::new(id, cell, owner, range, placed_at, trigger)
            .expect("test helpers construct valid bomb timing");
        self.insert_bomb(bomb);
        Ok(())
    }
}

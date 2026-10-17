use super::{State, StateError};
use crate::game::arena::Terrain;
use crate::game::bombs::{Bomb, BombTrigger};
use crate::game::types::{CellId, EntityId, Tick};

impl State {
    pub(crate) fn insert_test_flame(&mut self, cell: CellId, expires_at: Tick) {
        self.world.hazards.insert(cell, expires_at);
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

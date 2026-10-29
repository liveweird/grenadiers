use super::State;
use crate::game::{OilState, Tick, movement};

impl State {
    pub(super) fn reveal_safe_oil(&mut self, tick: Tick) {
        let ready: Vec<_> = self
            .world
            .oil
            .iter()
            .filter_map(|(cell, state)| {
                if state != OilState::Pending
                    || self.world.arena.blocks().contains(&cell)
                    || self.world.has_fire(cell, tick)
                    || self.world.bombs.values().any(|bomb| bomb.cell() == cell)
                    || self
                        .world
                        .pickups
                        .iter()
                        .any(|(pickup_cell, _)| pickup_cell == cell)
                    || self.actors.iter().any(|actor| {
                        actor.alive && movement::position_overlaps_cell(actor.position, cell)
                    })
                    || self
                        .lairs
                        .as_ref()
                        .is_some_and(|lairs| lairs.overlaps_cell(cell))
                {
                    None
                } else {
                    Some(cell)
                }
            })
            .collect();
        for cell in ready {
            self.world.oil.set_state(cell, OilState::Unlit);
        }
    }
}

use std::collections::BTreeSet;

use super::{LairLifecycle, LairState};
use crate::game::{EntityId, Tick, World, movement};

impl LairState {
    pub(crate) fn kill_on_fire(&mut self, world: &World, tick: Tick) -> Result<(), ()> {
        let killed = self
            .mobs
            .iter()
            .filter(|mob| {
                world
                    .committed_fire_cells()
                    .any(|cell| movement::position_overlaps_cell(mob.position, cell))
            })
            .map(|mob| mob.id)
            .collect();
        self.kill_mobs(&killed, tick)
    }

    pub(crate) fn kill_mobs(&mut self, killed: &BTreeSet<EntityId>, tick: Tick) -> Result<(), ()> {
        let killed_lairs = self
            .mobs
            .iter()
            .filter(|mob| killed.contains(&mob.id))
            .map(|mob| usize::from(mob.lair))
            .collect::<BTreeSet<_>>();
        let replacement =
            if self.lifecycle == LairLifecycle::ReplaceOnDeath && !killed_lairs.is_empty() {
                Some(Tick(
                    tick.0.checked_add(self.settings.spawn_ticks()).ok_or(())?,
                ))
            } else {
                None
            };
        self.mobs.retain(|mob| !killed.contains(&mob.id));
        if let Some(next_spawn_at) = replacement {
            for index in killed_lairs {
                if self.living_mobs(index) == 0 {
                    self.lairs[index].next_spawn_at = Some(next_spawn_at);
                }
            }
        }
        Ok(())
    }
}

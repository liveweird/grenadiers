use super::State;
use crate::game::events::{StepEvent, StepEventKind};
use crate::game::explosions;
use crate::game::types::Tick;
use crate::game::view::BombView;

impl State {
    pub(super) fn expire_flames(&mut self, tick: Tick) {
        self.world.hazards.retain(|_, expiry| *expiry > tick);
    }

    pub(super) fn resolve_explosions(&mut self, tick: Tick, events: &mut Vec<StepEvent>) {
        let plan = explosions::plan(&self.world, tick);
        if plan.detonated.is_empty() {
            return;
        }

        for id in &plan.detonated {
            let bomb = *self
                .world
                .bombs
                .get(id)
                .expect("planned detonations reference existing bombs");
            events.push(StepEvent::new(
                tick,
                StepEventKind::BombDetonated(BombView::from(bomb)),
            ));
            self.world.bombs.remove(id);
        }
        for actor in &mut self.actors {
            actor.bomb_grace.retain(|id| !plan.detonated.contains(id));
        }
        self.world.arena.remove_blocks(&plan.destroyed_blocks);

        // Tick is bounded by u64. On its final representable value there can be
        // no later step, so clamping preserves deterministic terminal state.
        let expiry = Tick(tick.0.saturating_add(self.rules.flame_ticks()));
        for cell in plan.flames {
            self.world
                .hazards
                .entry(cell)
                .and_modify(|current| *current = (*current).max(expiry))
                .or_insert(expiry);
        }
    }
}

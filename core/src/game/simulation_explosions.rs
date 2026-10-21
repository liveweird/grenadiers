use super::State;
use crate::game::events::{EventAudience, StepEvent, StepEventKind};
use crate::game::explosions;
use crate::game::simulation::StepError;
use crate::game::types::{EntityId, Tick};
use crate::game::view::BombView;
use std::collections::BTreeSet;

impl State {
    pub(super) fn expire_flames(&mut self, tick: Tick) {
        self.world.hazards.retain(|_, expiry| *expiry > tick);
    }

    pub(super) fn resolve_explosions(
        &mut self,
        tick: Tick,
        remote_targets: [Option<EntityId>; 2],
        contact_targets: BTreeSet<EntityId>,
        events: &mut Vec<StepEvent>,
    ) -> Result<(), StepError> {
        let plan =
            explosions::plan_with_contacts(&self.world, tick, remote_targets, &contact_targets);
        if plan.detonated.is_empty() && plan.destroyed_mines.is_empty() {
            return Ok(());
        }

        for id in &plan.detonated {
            let bomb = *self
                .world
                .bombs
                .get(id)
                .expect("planned detonations reference existing bombs");
            events.push(StepEvent::new(
                tick,
                EventAudience::Both,
                StepEventKind::BombDetonated(BombView::from(bomb)),
            ));
            self.world.bombs.remove(id);
        }
        for actor in &mut self.actors {
            actor
                .bomb_grace
                .retain(|id| !plan.detonated.contains(id) && !plan.destroyed_mines.contains(id));
        }
        for id in &plan.destroyed_mines {
            self.world.bombs.remove(id);
        }
        self.world
            .pickups
            .apply_explosion(&plan.destroyed_blocks, &plan.flames);
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
        Ok(())
    }
}

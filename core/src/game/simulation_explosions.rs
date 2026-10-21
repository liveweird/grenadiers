use super::State;
use crate::game::events::{StepEvent, StepEventKind};
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
        timed_paused: bool,
        events: &mut Vec<StepEvent>,
    ) -> Result<(), StepError> {
        let plan =
            explosions::plan_with_contacts(&self.world, tick, remote_targets, &contact_targets);
        if plan.detonated.is_empty()
            && plan.destroyed_mines.is_empty()
            && plan.ignited_oil.is_empty()
        {
            return Ok(());
        }

        for id in &plan.detonated {
            let bomb = *self
                .world
                .bombs
                .get(id)
                .expect("planned detonations reference existing bombs");
            events.push(StepEvent::bomb(
                tick,
                self.bomb_event_audience_for(bomb.owner(), tick),
                StepEventKind::BombDetonated(BombView::from(bomb)),
                bomb,
                timed_paused,
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
        let concealed_oil: Vec<_> = self
            .world
            .oil
            .iter()
            .filter_map(|(cell, state)| {
                (state == crate::game::OilState::Concealed && plan.destroyed_blocks.contains(&cell))
                    .then_some(cell)
            })
            .collect();
        for cell in concealed_oil {
            self.world
                .oil
                .set_state(cell, crate::game::OilState::Pending);
        }
        self.world.arena.remove_blocks(&plan.destroyed_blocks);
        for actor in &mut self.actors {
            crate::game::movement::prune_soft_wall_grace(&self.world, actor);
        }

        // Tick is bounded by u64. On its final representable value there can be
        // no later step, so clamping preserves deterministic terminal state.
        let expiry = Tick(tick.0.saturating_add(self.rules.flame_ticks()));
        for cell in &plan.ignited_oil {
            self.world
                .oil
                .set_state(*cell, crate::game::OilState::Burning);
            self.world.hazards.remove(cell);
        }
        for actor in &mut self.actors {
            crate::game::movement::prune_unlit_oil_grace(&self.world, actor);
        }
        for cell in plan.flames {
            if self.world.oil.burning(cell) {
                continue;
            }
            self.world
                .hazards
                .entry(cell)
                .and_modify(|current| *current = (*current).max(expiry))
                .or_insert(expiry);
        }
        Ok(())
    }
}

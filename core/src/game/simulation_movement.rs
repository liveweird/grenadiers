use super::State;
use crate::game::{
    Tick,
    geometry::MovementTrace,
    movement::{self, MovementOutcome},
};
use std::collections::BTreeSet;

impl State {
    pub(super) fn move_living(
        &mut self,
        tick: Tick,
        frozen: &[bool],
        moving_at_start: &BTreeSet<crate::game::EntityId>,
    ) -> (
        Vec<MovementOutcome>,
        Vec<Option<(crate::game::EntityId, crate::game::Direction)>>,
    ) {
        if self.rules.version() >= 9 {
            return self.move_living_rules9(tick, frozen, moving_at_start);
        }
        let turn_assist = self.rules.turn_assist();
        let slow_foot_enabled = self.rules.enabled_items().contains_slow_foot();
        let wall_pass_enabled = self.rules.enabled_items().contains_wall_pass();
        let kick_rule_enabled = self.rules.enabled_items().contains_kick();
        let mut requests = vec![None; self.actors.len()];
        let outcomes = self
            .actors
            .iter_mut()
            .enumerate()
            .map(|(current, actor)| {
                if actor.alive && !frozen[current] {
                    let can_kick = kick_rule_enabled && actor.inventory().can_kick();
                    let config = movement::AdvanceConfig {
                        turn_assist,
                        tick,
                        slow_foot_enabled,
                        wall_pass_enabled,
                        kick_enabled: can_kick,
                        moving_at_start,
                        player_obstacles: None,
                    };
                    let outcome = movement::advance_actor(&self.world, actor, &config);
                    requests[current] = outcome.kick;
                    outcome
                } else {
                    MovementOutcome {
                        trace: MovementTrace::new(actor.position),
                        kick: None,
                        original_budget: 0,
                        forward_segment: None,
                        blocking_player: None,
                        punch_contact: None,
                    }
                }
            })
            .collect();
        (outcomes, requests)
    }
}

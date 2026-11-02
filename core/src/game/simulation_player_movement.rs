use std::collections::BTreeSet;

use super::{Direction, EntityId, State, Tick};
use crate::game::geometry::MovementTrace;
use crate::game::movement::{self, MovementOutcome};
use crate::game::player_collision::PlayerObstacle;

impl State {
    pub(super) fn move_living_rules9(
        &mut self,
        tick: Tick,
        frozen: &[bool],
        moving_at_start: &BTreeSet<EntityId>,
    ) -> (Vec<MovementOutcome>, Vec<Option<(EntityId, Direction)>>) {
        debug_assert!(self.rules.version() >= 9);
        let mut reservations: Vec<_> = self
            .actors
            .iter()
            .filter(|actor| actor.alive)
            .map(|actor| PlayerObstacle::stationary(actor.seat, actor.position))
            .collect();
        let mut outcomes = vec![None; self.actors.len()];
        let mut requests = vec![None; self.actors.len()];
        let priority: Vec<_> = self.assignment.priority_order(self.player_count).collect();
        for seat in priority {
            let index = seat.index();
            if !self.actors[index].alive || frozen[index] {
                outcomes[index] = Some(stationary_outcome(self.actors[index].position));
                continue;
            }
            let mut candidate = self.actors[index].clone();
            let config = movement::AdvanceConfig {
                turn_assist: self.rules.turn_assist(),
                tick,
                slow_foot_enabled: self.rules.enabled_items().contains_slow_foot(),
                wall_pass_enabled: self.rules.enabled_items().contains_wall_pass(),
                kick_enabled: self.rules.enabled_items().contains_kick()
                    && candidate.inventory().can_kick(),
                moving_at_start,
                player_obstacles: Some(&reservations),
            };
            let outcome = movement::advance_actor(&self.world, &mut candidate, &config);
            requests[index] = outcome.kick;
            reservations.extend(
                outcome
                    .trace
                    .segments()
                    .map(|segment| PlayerObstacle { seat, segment }),
            );
            self.actors[index] = candidate;
            outcomes[index] = Some(outcome);
        }
        let outcomes = outcomes
            .into_iter()
            .enumerate()
            .map(|(index, outcome)| {
                outcome.unwrap_or_else(|| stationary_outcome(self.actors[index].position))
            })
            .collect();
        (outcomes, requests)
    }
}

fn stationary_outcome(position: super::Position) -> MovementOutcome {
    MovementOutcome {
        trace: MovementTrace::new(position),
        kick: None,
        original_budget: 0,
        forward_segment: None,
        blocking_player: None,
        punch_contact: None,
    }
}

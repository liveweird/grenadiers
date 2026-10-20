use super::{Actor, ActorInit, HeldDirections, Inventory, RulesConfig, State, TimedEffects};
use crate::game::{Equipment, SeatId, movement};
use std::collections::BTreeSet;

impl Actor {
    pub(super) fn new(init: ActorInit, rules: RulesConfig) -> Self {
        Self {
            id: init.id,
            seat: init.seat,
            position: init.position,
            heading: init.heading,
            speed: init.speed,
            held: HeldDirections::default(),
            residual: 0,
            pending_turn: None,
            bomb_grace: BTreeSet::new(),
            soft_wall_grace: BTreeSet::new(),
            capacity: rules.bomb_capacity(),
            range: rules.bomb_range(),
            alive: true,
            inventory: Inventory::default(),
            effects: TimedEffects::default(),
            damage_grace: None,
        }
    }
}

impl State {
    pub(super) fn replace_actor_equipment(&mut self, seat: SeatId, equipment: Equipment) {
        let actor = &mut self.actors[seat.index()];
        let previous = actor.inventory.equipment();
        let had_wall_pass = self.rules.enabled_items().contains_wall_pass()
            && previous == Some(Equipment::WallPass);
        if had_wall_pass && equipment != Equipment::WallPass {
            actor.soft_wall_grace = movement::overlapping_soft_blocks(&self.world, actor.position);
        } else if equipment == Equipment::WallPass {
            actor.soft_wall_grace.clear();
        } else {
            movement::prune_soft_wall_grace(&self.world, actor);
        }
        actor.inventory.replace_equipment(equipment);
    }
}

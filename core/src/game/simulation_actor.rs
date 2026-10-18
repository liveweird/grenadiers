use super::{Actor, ActorInit, HeldDirections, Inventory, RulesConfig, TimedEffects};
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
            capacity: rules.bomb_capacity(),
            range: rules.bomb_range(),
            alive: true,
            inventory: Inventory::default(),
            effects: TimedEffects::default(),
            damage_grace: None,
        }
    }
}

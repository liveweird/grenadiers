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
            unlit_oil_grace: BTreeSet::new(),
            capacity: rules.bomb_capacity(),
            range: rules.bomb_range(),
            alive: true,
            inventory: Inventory::default(),
            effects: TimedEffects::default(),
            damage_grace: None,
            last_successful_punch_tick: None,
            last_successful_shove_tick: None,
            last_teleport_tick: None,
            storm_slow_until: None,
        }
    }

    #[must_use]
    pub const fn last_teleport_tick(&self) -> Option<crate::game::Tick> {
        self.last_teleport_tick
    }
}

impl Actor {
    pub(super) fn kill(&mut self) {
        self.alive = false;
        self.held.clear();
        self.residual = 0;
        self.pending_turn = None;
        self.inventory.clear();
        self.soft_wall_grace.clear();
        self.unlit_oil_grace.clear();
        self.effects.clear();
        self.damage_grace = None;
        self.storm_slow_until = None;
    }
}

impl State {
    pub(super) fn expire_actor_effects(
        &mut self,
        previous_tick: crate::game::Tick,
        tick: crate::game::Tick,
    ) {
        for actor in &mut self.actors {
            let reversed_before = actor.effects.reverses_controls(previous_tick);
            let reversed_after = actor.effects.reverses_controls(tick);
            actor.effects.expire(tick);
            if reversed_before != reversed_after {
                actor.pending_turn = None;
            }
            if actor
                .damage_grace
                .is_some_and(|grace| !grace.is_active(tick))
            {
                actor.damage_grace = None;
            }
        }
    }

    pub(super) fn replace_actor_equipment(&mut self, seat: SeatId, equipment: Equipment) {
        let actor = &mut self.actors[seat.index()];
        let previous = actor.inventory.equipment();
        let had_wall_pass = self.rules.enabled_items().contains_wall_pass()
            && previous == Some(Equipment::WallPass);
        if had_wall_pass && equipment != Equipment::WallPass {
            actor.soft_wall_grace = movement::overlapping_soft_blocks(&self.world, actor.position);
            actor.unlit_oil_grace = movement::overlapping_unlit_oil(&self.world, actor.position);
        } else if equipment == Equipment::WallPass {
            actor.soft_wall_grace.clear();
            actor.unlit_oil_grace.clear();
        } else {
            movement::prune_soft_wall_grace(&self.world, actor);
            movement::prune_unlit_oil_grace(&self.world, actor);
        }
        actor.inventory.replace_equipment(equipment);
    }
}

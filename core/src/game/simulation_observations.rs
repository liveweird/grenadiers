use super::{Actor, RoundResult, State};
use crate::game::assignment::RoundAssignment;
use crate::game::damage::DamageGrace;
use crate::game::effects::EffectKind;
use crate::game::events::EventAudience;
use crate::game::rules::RulesConfig;
use crate::game::types::{EntityId, SeatId, Tick};
use crate::game::world::World;

impl Actor {
    pub(crate) fn effect_observations(&self) -> impl Iterator<Item = (EffectKind, Tick)> + '_ {
        self.effects.observations()
    }

    pub(crate) const fn damage_grace_observation(&self) -> Option<DamageGrace> {
        self.damage_grace
    }
}

impl State {
    pub(super) fn event_audience_for(&self, id: EntityId, at: Tick) -> EventAudience {
        self.actors
            .iter()
            .find(|actor| actor.id == id && actor.effects.hidden_from_opponents(at))
            .map_or(EventAudience::Both, |actor| EventAudience::only(actor.seat))
    }

    #[must_use]
    pub const fn rules(&self) -> &RulesConfig {
        &self.rules
    }

    #[must_use]
    pub const fn world(&self) -> &World {
        &self.world
    }

    #[must_use]
    pub const fn actors(&self) -> &[Actor; 2] {
        &self.actors
    }

    #[must_use]
    pub const fn tick(&self) -> Tick {
        self.tick
    }

    #[must_use]
    pub const fn running_updates(&self) -> u64 {
        self.running_updates
    }

    #[must_use]
    pub const fn result(&self) -> Option<RoundResult> {
        self.result
    }

    #[must_use]
    pub const fn round_assignment(&self) -> RoundAssignment {
        self.assignment
    }

    #[must_use]
    pub fn voluntary_actions_frozen(&self) -> [bool; 2] {
        self.stopwatch_context(self.tick).frozen
    }

    #[must_use]
    pub fn timed_bombs_paused(&self) -> bool {
        self.stopwatch_context(self.tick).pause_timed
    }

    #[must_use]
    pub fn live_bomb_count(&self, seat: SeatId) -> usize {
        let owner = self.actors[seat.index()].id;
        self.world
            .bombs
            .values()
            .filter(|bomb| bomb.owner() == owner)
            .count()
    }
}

use super::State;
use crate::game::effects::{EffectApplication, MAX_SLOW_FOOT_STACKS};
use crate::game::events::{EventAudience, StepEvent, StepEventKind};
use crate::game::pickups::{Pickup, PickupKind};
use crate::game::simulation::StepError;
use crate::game::types::{CellId, Position, SeatId, Tick};

impl State {
    pub(super) fn collect_pickups(
        &mut self,
        tick: Tick,
        events: &mut Vec<StepEvent>,
    ) -> Result<(), StepError> {
        if !self.rules.pickup_collection() {
            return Ok(());
        }
        let cells: Vec<_> = self.world.pickups.exposed.keys().copied().collect();
        for cell in cells {
            let kind = self.world.pickups.exposed[&cell].kind();
            let Some(seat) = self.collector_for(cell, kind, tick) else {
                continue;
            };
            let pickup = self.world.pickups.exposed[&cell];
            let actor = &mut self.actors[seat.index()];
            if !actor.apply_pickup(pickup, self.rules, tick)? {
                continue;
            }
            self.world.pickups.exposed.remove(&cell);
            events.push(StepEvent::new(
                tick,
                EventAudience::Both,
                StepEventKind::PickupCollected {
                    id: actor.id,
                    seat,
                    cell,
                    kind: pickup.kind(),
                },
            ));
        }
        Ok(())
    }

    fn collector_for(&self, cell: CellId, kind: PickupKind, tick: Tick) -> Option<SeatId> {
        let center = Position::center(cell);
        let distance = |seat: SeatId| {
            let actor = &self.actors[seat.index()];
            let eligible = kind != PickupKind::SlowFoot
                || actor.effects.slow_foot_stacks(tick) < MAX_SLOW_FOOT_STACKS;
            (actor.alive && eligible && actor.position.cell() == Some(cell)).then(|| {
                let dx = i64::from(actor.position.x - center.x);
                let dy = i64::from(actor.position.y - center.y);
                dx * dx + dy * dy
            })
        };
        let one = distance(SeatId::One);
        let two = distance(SeatId::Two);
        match (one, two) {
            (Some(left), Some(right)) if left < right => Some(SeatId::One),
            (Some(left), Some(right)) if right < left => Some(SeatId::Two),
            (Some(_), Some(_)) => Some(self.assignment.priority_seat()),
            (Some(_), None) => Some(SeatId::One),
            (None, Some(_)) => Some(SeatId::Two),
            (None, None) => None,
        }
    }
}

impl super::Actor {
    fn apply_pickup(
        &mut self,
        pickup: Pickup,
        rules: crate::game::RulesConfig,
        tick: Tick,
    ) -> Result<bool, StepError> {
        match pickup.kind() {
            PickupKind::BombCapacity => {
                self.capacity = self
                    .capacity
                    .saturating_add(1)
                    .min(rules.bomb_capacity_cap());
            }
            PickupKind::BlastRange => {
                self.range = self.range.saturating_add(1).min(rules.bomb_range_cap());
            }
            PickupKind::MovementSpeed => self.speed = self.speed.next(),
            PickupKind::Heart => {
                self.inventory
                    .replace_equipment(crate::game::Equipment::Heart);
            }
            PickupKind::RemoteBomb => {
                self.inventory
                    .replace_weapon(crate::game::SpecialWeapon::Remote);
            }
            PickupKind::PiercingBomb => {
                self.inventory
                    .replace_weapon(crate::game::SpecialWeapon::Piercing);
            }
            PickupKind::PowerBomb => {
                self.inventory
                    .replace_weapon(crate::game::SpecialWeapon::Power);
            }
            PickupKind::Landmine => {
                self.inventory
                    .replace_weapon(crate::game::SpecialWeapon::Landmine);
            }
            PickupKind::Shield
            | PickupKind::GoldenFlame
            | PickupKind::ShortFuse
            | PickupKind::Diarrhea
            | PickupKind::SlowFoot
            | PickupKind::ShortFlame
            | PickupKind::LongFuse
            | PickupKind::ReverseControls => {
                let effect = pickup
                    .kind()
                    .timed_effect()
                    .expect("timed pickups map to their effect");
                let reversed_before = self.effects.reverses_controls(tick);
                let application = self
                    .effects
                    .apply(effect, tick)
                    .map_err(|_| StepError::EffectExpiryOverflow(effect))?;
                if application == EffectApplication::Ineligible {
                    return Ok(false);
                }
                if self.effects.reverses_controls(tick) != reversed_before {
                    self.pending_turn = None;
                }
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
#[path = "diarrhea_tests.rs"]
mod diarrhea_tests;
#[cfg(test)]
#[path = "short_flame_tests.rs"]
mod short_flame_tests;
#[cfg(test)]
#[path = "slow_foot_tests.rs"]
mod slow_foot_tests;

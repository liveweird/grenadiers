use super::State;
use crate::game::events::{StepEvent, StepEventKind};
use crate::game::pickups::{Pickup, PickupKind};
use crate::game::types::{CellId, Position, SeatId, Tick};

impl State {
    pub(super) fn collect_pickups(&mut self, tick: Tick, events: &mut Vec<StepEvent>) {
        if !self.rules.pickup_collection() {
            return;
        }
        let cells: Vec<_> = self.world.pickups.exposed.keys().copied().collect();
        for cell in cells {
            let Some(seat) = self.collector_for(cell) else {
                continue;
            };
            let pickup = self
                .world
                .pickups
                .exposed
                .remove(&cell)
                .expect("collection candidates came from the exposed layer");
            let actor = &mut self.actors[seat.index()];
            actor.apply_pickup(pickup, self.rules);
            events.push(StepEvent::new(
                tick,
                StepEventKind::PickupCollected {
                    id: actor.id,
                    seat,
                    cell,
                    kind: pickup.kind(),
                },
            ));
        }
    }

    fn collector_for(&self, cell: CellId) -> Option<SeatId> {
        let center = Position::center(cell);
        let distance = |seat: SeatId| {
            let actor = &self.actors[seat.index()];
            (actor.alive && actor.position.cell() == Some(cell)).then(|| {
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
    fn apply_pickup(&mut self, pickup: Pickup, rules: crate::game::RulesConfig) {
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
        }
    }
}

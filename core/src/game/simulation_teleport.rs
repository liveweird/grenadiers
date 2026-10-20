use std::collections::BTreeSet;

use super::State;
use crate::game::{BombTrigger, EntityId, Position, SeatId, Tick, movement};

impl State {
    pub(super) fn commit_teleport(&mut self, requests: [bool; 2], tick: Tick) {
        let requester = match requests {
            [true, true] => self.assignment.priority_seat(),
            [true, false] => SeatId::One,
            [false, true] => SeatId::Two,
            [false, false] => return,
        };
        let other = if requester == SeatId::One {
            SeatId::Two
        } else {
            SeatId::One
        };
        let from = self.actors[requester.index()].position;
        let to = self.actors[other.index()].position;
        if !self.actors[requester.index()].alive || !self.actors[other.index()].alive || from == to
        {
            return;
        }

        let requester_grace = self.retained_grace(requester, to);
        let other_grace = self.retained_grace(other, from);
        if !self.teleport_destination_is_safe(to, &requester_grace)
            || !self.teleport_destination_is_safe(from, &other_grace)
        {
            return;
        }

        self.actors[requester.index()].position = to;
        self.actors[requester.index()].residual = 0;
        self.actors[requester.index()].pending_turn = None;
        self.actors[requester.index()].bomb_grace = requester_grace;
        self.actors[other.index()].position = from;
        self.actors[other.index()].residual = 0;
        self.actors[other.index()].pending_turn = None;
        self.actors[other.index()].bomb_grace = other_grace;
        self.last_teleport_tick = Some(tick);
    }

    fn retained_grace(&self, seat: SeatId, destination: Position) -> BTreeSet<EntityId> {
        self.actors[seat.index()]
            .bomb_grace
            .iter()
            .filter(|id| {
                self.world
                    .bombs
                    .get(id)
                    .is_some_and(|bomb| movement::position_overlaps_cell(destination, bomb.cell()))
            })
            .copied()
            .collect()
    }

    fn teleport_destination_is_safe(
        &self,
        position: Position,
        retained_grace: &BTreeSet<EntityId>,
    ) -> bool {
        if !movement::position_is_free(&self.world, position, retained_grace) {
            return false;
        }
        if self
            .world
            .hazards
            .keys()
            .any(|cell| movement::position_overlaps_cell(position, *cell))
        {
            return false;
        }
        self.world
            .bombs
            .values()
            .filter(|bomb| matches!(bomb.trigger(), BombTrigger::Contact))
            .all(|bomb| {
                !movement::position_overlaps_cell(position, bomb.cell())
                    || retained_grace.contains(&bomb.id())
            })
    }
}

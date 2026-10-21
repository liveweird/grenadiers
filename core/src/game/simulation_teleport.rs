use std::collections::BTreeSet;

use super::State;
use crate::game::{BombTrigger, CellId, EntityId, Position, SeatId, Tick, movement};

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
        let requester_soft_grace = self.retained_soft_grace(requester, to);
        let other_soft_grace = self.retained_soft_grace(other, from);
        let requester_oil_grace = self.retained_oil_grace(requester, to);
        let other_oil_grace = self.retained_oil_grace(other, from);
        if !self.teleport_destination_is_safe(
            requester,
            to,
            &requester_grace,
            &requester_soft_grace,
            &requester_oil_grace,
        ) || !self.teleport_destination_is_safe(
            other,
            from,
            &other_grace,
            &other_soft_grace,
            &other_oil_grace,
        ) {
            return;
        }

        self.actors[requester.index()].position = to;
        self.actors[requester.index()].residual = 0;
        self.actors[requester.index()].pending_turn = None;
        self.actors[requester.index()].bomb_grace = requester_grace;
        self.actors[requester.index()].soft_wall_grace = requester_soft_grace;
        self.actors[requester.index()].unlit_oil_grace = requester_oil_grace;
        self.actors[other.index()].position = from;
        self.actors[other.index()].residual = 0;
        self.actors[other.index()].pending_turn = None;
        self.actors[other.index()].bomb_grace = other_grace;
        self.actors[other.index()].soft_wall_grace = other_soft_grace;
        self.actors[other.index()].unlit_oil_grace = other_oil_grace;
        self.last_teleport_tick = Some(tick);
    }

    fn retained_soft_grace(&self, seat: SeatId, destination: Position) -> BTreeSet<CellId> {
        let actor = &self.actors[seat.index()];
        actor
            .soft_wall_grace
            .iter()
            .filter(|cell| {
                self.world.arena.blocks().contains(cell)
                    && movement::position_overlaps_cell(actor.position, **cell)
                    && movement::position_overlaps_cell(destination, **cell)
            })
            .copied()
            .collect()
    }

    fn retained_oil_grace(&self, seat: SeatId, destination: Position) -> BTreeSet<CellId> {
        let actor = &self.actors[seat.index()];
        actor
            .unlit_oil_grace
            .iter()
            .filter(|cell| {
                self.world.oil.unlit(**cell)
                    && movement::position_overlaps_cell(actor.position, **cell)
                    && movement::position_overlaps_cell(destination, **cell)
            })
            .copied()
            .collect()
    }

    fn retained_grace(&self, seat: SeatId, destination: Position) -> BTreeSet<EntityId> {
        let actor = &self.actors[seat.index()];
        actor
            .bomb_grace
            .iter()
            .filter(|id| {
                self.world.bombs.get(id).is_some_and(|bomb| {
                    movement::position_overlaps_cell(actor.position, bomb.cell())
                        && movement::position_overlaps_cell(destination, bomb.cell())
                })
            })
            .copied()
            .collect()
    }

    fn teleport_destination_is_safe(
        &self,
        seat: SeatId,
        position: Position,
        retained_grace: &BTreeSet<EntityId>,
        retained_soft_grace: &BTreeSet<CellId>,
        retained_oil_grace: &BTreeSet<CellId>,
    ) -> bool {
        let actor = &self.actors[seat.index()];
        let can_pass = self.rules.enabled_items().contains_wall_pass()
            && actor.inventory.can_pass_soft_walls();
        if !movement::position_is_free(
            &self.world,
            position,
            retained_grace,
            can_pass,
            retained_soft_grace,
            retained_oil_grace,
        ) {
            return false;
        }
        if self
            .world
            .committed_fire_cells()
            .any(|cell| movement::position_overlaps_cell(position, cell))
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

use super::{InputAction, MAX_INPUTS_PER_TICK, SeatInput, State};
use crate::game::arena::Terrain;
use crate::game::bombs::{Bomb, BombTrigger};
use crate::game::events::{EventAudience, StepEvent, StepEventKind};
use crate::game::movement;
use crate::game::types::{EntityId, SeatId, Tick};
use crate::game::view::BombView;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ManualAction {
    PlaceBomb,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ManualActionQueue {
    actions: [ManualAction; MAX_INPUTS_PER_TICK],
    len: usize,
}

impl Default for ManualActionQueue {
    fn default() -> Self {
        Self {
            actions: [ManualAction::PlaceBomb; MAX_INPUTS_PER_TICK],
            len: 0,
        }
    }
}

impl ManualActionQueue {
    fn push(&mut self, action: ManualAction) {
        let slot = self
            .actions
            .get_mut(self.len)
            .expect("the validated logical-input bound guarantees queue capacity");
        *slot = action;
        self.len += 1;
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    pub(super) fn as_slice(&self) -> &[ManualAction] {
        &self.actions[..self.len]
    }

    fn first(&self) -> Option<ManualAction> {
        self.as_slice().first().copied()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ManualActionQueues {
    seats: [ManualActionQueue; 2],
}

impl ManualActionQueues {
    pub(super) fn for_seat(&self, seat: SeatId) -> &ManualActionQueue {
        &self.seats[seat.index()]
    }
}

impl State {
    pub(super) fn apply_inputs(&mut self, inputs: &[SeatInput]) -> ManualActionQueues {
        let mut manual_actions = ManualActionQueues::default();
        for input in inputs {
            let actor = &mut self.actors[input.seat.index()];
            if !actor.alive {
                continue;
            }
            let previous = actor.held.active();
            match input.action {
                InputAction::Press(direction) => actor.held.press(direction),
                InputAction::Release(direction) => actor.held.release(direction),
                InputAction::Clear => {
                    actor.held.clear();
                    actor.pending_turn = None;
                    actor.residual = 0;
                    manual_actions.seats[input.seat.index()].clear();
                }
                InputAction::PlaceBomb => {
                    manual_actions.seats[input.seat.index()].push(ManualAction::PlaceBomb);
                }
            }
            if actor.held.active() != previous {
                actor.pending_turn = None;
            }
            if actor.held.active().is_none() {
                actor.residual = 0;
            }
        }
        manual_actions
    }

    pub(super) fn resolve_classic_manual_actions(
        &mut self,
        actions: &ManualActionQueues,
        tick: Tick,
        events: &mut Vec<StepEvent>,
    ) {
        let requests =
            SeatId::ALL.map(|seat| actions.for_seat(seat).first() == Some(ManualAction::PlaceBomb));
        self.resolve_classic_placements(requests, tick, events);
    }

    fn resolve_classic_placements(
        &mut self,
        requests: [bool; 2],
        tick: Tick,
        events: &mut Vec<StepEvent>,
    ) {
        let mut targets = [None; 2];
        for seat in SeatId::ALL {
            let actor = &self.actors[seat.index()];
            let Some(cell) = actor.position.cell() else {
                continue;
            };
            let has_capacity = self.live_bomb_count(seat) < usize::from(actor.capacity);
            let cell_is_free = self.world.arena.terrain(cell) == Some(Terrain::Floor)
                && !self.world.arena.blocks().contains(&cell)
                && !self.world.bombs.values().any(|bomb| bomb.cell() == cell);
            if requests[seat.index()] && actor.alive && has_capacity && cell_is_free {
                targets[seat.index()] = Some(cell);
            }
        }
        if targets[0].is_some() && targets[0] == targets[1] {
            return;
        }
        for seat in SeatId::ALL {
            let Some(cell) = targets[seat.index()] else {
                continue;
            };
            let actor = &self.actors[seat.index()];
            let Some(fuse_ticks) = actor
                .effects
                .bomb_fuse_ticks(self.rules.bomb_fuse_ticks(), tick)
            else {
                continue;
            };
            let Some(detonates_at) = tick.0.checked_add(fuse_ticks).map(Tick) else {
                continue;
            };
            let Some(id) = self.next_bomb_id() else {
                continue;
            };
            let Ok(bomb) = Bomb::new(
                id,
                cell,
                actor.id,
                actor.effective_bomb_range(tick),
                tick,
                BombTrigger::Timed { detonates_at },
            ) else {
                continue;
            };
            self.insert_bomb(bomb);
            events.push(StepEvent::new(
                tick,
                EventAudience::Both,
                StepEventKind::BombPlaced(BombView::from(bomb)),
            ));
        }
    }

    fn next_bomb_id(&self) -> Option<EntityId> {
        let mut candidate = self.next_entity_id;
        while let Ok(raw) = u32::try_from(candidate) {
            let id = EntityId(raw);
            if !self.actors.iter().any(|actor| actor.id == id)
                && !self.world.bombs.contains_key(&id)
            {
                return Some(id);
            }
            candidate += 1;
        }
        None
    }

    pub(super) fn insert_bomb(&mut self, bomb: Bomb) {
        let id = bomb.id();
        let cell = bomb.cell();
        self.world.bombs.insert(id, bomb);
        self.next_entity_id = self.next_entity_id.max(u64::from(id.0) + 1);
        for actor in &mut self.actors {
            if movement::position_overlaps_cell(actor.position, cell) {
                actor.bomb_grace.insert(id);
            }
        }
    }
}

#[cfg(test)]
#[path = "simulation_actions_tests.rs"]
mod tests;

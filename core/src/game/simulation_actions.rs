use super::{InputAction, MAX_INPUTS_PER_TICK, SeatInput, State};
use crate::game::arena::Terrain;
use crate::game::bombs::{BlastKind, Bomb, BombTrigger};
use crate::game::events::{EventAudience, StepEvent, StepEventKind};
use crate::game::inventory::SpecialWeapon;
use crate::game::movement;
use crate::game::types::{EntityId, SeatId, Tick};
use crate::game::view::BombView;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ManualAction {
    PlaceBomb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlacementOrigin {
    Manual,
    Automatic,
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
    ) -> [Option<EntityId>; 2] {
        let mut remote_targets = [None; 2];
        let requests = SeatId::ALL.map(|seat| {
            let actor = &self.actors[seat.index()];
            if actions.for_seat(seat).first() == Some(ManualAction::PlaceBomb) && actor.alive {
                if let Some(id) = self.world.bombs.values().find_map(|bomb| {
                    (bomb.owner() == actor.id && matches!(bomb.trigger(), BombTrigger::Remote))
                        .then_some(bomb.id())
                }) {
                    remote_targets[seat.index()] = Some(id);
                    return None;
                }
                Some(PlacementOrigin::Manual)
            } else if actor.effects.forces_bomb_drops(tick) {
                Some(PlacementOrigin::Automatic)
            } else {
                None
            }
        });
        self.resolve_classic_placements(requests, tick, events);
        remote_targets
    }

    fn resolve_classic_placements(
        &mut self,
        requests: [Option<PlacementOrigin>; 2],
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
            if requests[seat.index()].is_some() && actor.alive && has_capacity && cell_is_free {
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
            let weapon = if requests[seat.index()] == Some(PlacementOrigin::Manual) {
                actor.inventory.weapon().filter(|weapon| match weapon {
                    SpecialWeapon::Piercing => self.rules.enabled_items().contains_piercing_bomb(),
                    SpecialWeapon::Power => self.rules.enabled_items().contains_power_bomb(),
                    SpecialWeapon::Remote => self.rules.enabled_items().contains_remote_bomb(),
                    SpecialWeapon::Landmine => self.rules.enabled_items().contains_landmine(),
                })
            } else {
                None
            };
            let blast = match weapon {
                Some(SpecialWeapon::Piercing) => BlastKind::Piercing,
                Some(SpecialWeapon::Power) => BlastKind::Power,
                Some(SpecialWeapon::Remote | SpecialWeapon::Landmine) | None => BlastKind::Standard,
            };
            let Some(id) = self.next_bomb_id() else {
                continue;
            };
            let bomb = if weapon == Some(SpecialWeapon::Remote) {
                Bomb::new(
                    id,
                    cell,
                    actor.id,
                    actor.effective_bomb_range(tick),
                    tick,
                    BombTrigger::Remote,
                )
            } else if weapon == Some(SpecialWeapon::Landmine) {
                Bomb::new(
                    id,
                    cell,
                    actor.id,
                    actor.effective_bomb_range(tick),
                    tick,
                    BombTrigger::Contact,
                )
            } else {
                let Some(fuse_ticks) = actor
                    .effects
                    .bomb_fuse_ticks(self.rules.bomb_fuse_ticks(), tick)
                else {
                    continue;
                };
                let Some(detonates_at) = tick.0.checked_add(fuse_ticks).map(Tick) else {
                    continue;
                };
                if blast != BlastKind::Standard {
                    Bomb::new_with_blast(
                        id,
                        cell,
                        actor.id,
                        actor.effective_bomb_range(tick),
                        tick,
                        BombTrigger::Timed { detonates_at },
                        blast,
                    )
                } else {
                    Bomb::new(
                        id,
                        cell,
                        actor.id,
                        actor.effective_bomb_range(tick),
                        tick,
                        BombTrigger::Timed { detonates_at },
                    )
                }
            };
            let Ok(bomb) = bomb else {
                continue;
            };
            self.insert_bomb(bomb);
            if let Some(weapon) = weapon {
                let consumed = self.actors[seat.index()].inventory.consume_weapon();
                debug_assert_eq!(consumed, Some(weapon));
            }
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
#[path = "piercing_bomb_tests.rs"]
mod piercing_bomb_tests;

#[cfg(test)]
#[path = "power_bomb_tests.rs"]
mod power_bomb_tests;

#[cfg(test)]
#[path = "remote_bomb_tests.rs"]
mod remote_bomb_tests;

#[cfg(test)]
#[path = "simulation_actions_tests.rs"]
mod tests;

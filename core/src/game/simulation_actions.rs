use super::manual_actions::ManualActionQueues;
use super::stopwatch::StopwatchContext;
use super::{InputAction, SeatInput, State};
use crate::game::arena::Terrain;
use crate::game::bombs::{BlastKind, Bomb, BombTrigger};
use crate::game::events::{StepEvent, StepEventKind};
use crate::game::inventory::SpecialWeapon;
use crate::game::movement;
use crate::game::types::{EntityId, Tick};
use crate::game::view::BombView;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ManualAction {
    PlaceBomb,
    UseEquipment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlacementOrigin {
    Manual,
    Automatic,
}

impl State {
    pub(super) fn apply_inputs(&mut self, inputs: &[SeatInput]) -> ManualActionQueues {
        let mut manual_actions = ManualActionQueues::new(self.actors.len());
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
                InputAction::UseEquipment => {
                    manual_actions.seats[input.seat.index()].push(ManualAction::UseEquipment);
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

    pub(super) fn resolve_manual_actions(
        &mut self,
        actions: &ManualActionQueues,
        tick: Tick,
        stopwatch: &StopwatchContext,
        events: &mut Vec<StepEvent>,
    ) -> Vec<Option<EntityId>> {
        let seats = self.player_count.seats();
        let mut remote_targets = vec![None; self.actors.len()];
        let first_bombs: Vec<_> = seats
            .iter()
            .map(|&seat| {
                if stopwatch.frozen[seat.index()] {
                    return None;
                }
                actions
                    .for_seat(seat)
                    .as_slice()
                    .iter()
                    .position(|action| *action == ManualAction::PlaceBomb)
            })
            .collect();
        let eventual_requests: Vec<_> = seats
            .iter()
            .map(|&seat| {
                let actor = &self.actors[seat.index()];
                if actor.alive && first_bombs[seat.index()].is_some() {
                    if self.world.bombs.values().any(|bomb| {
                        bomb.owner() == actor.id && matches!(bomb.trigger(), BombTrigger::Remote)
                    }) {
                        return None;
                    }
                    Some(PlacementOrigin::Manual)
                } else if actor.effects.forces_bomb_drops(tick) {
                    Some(PlacementOrigin::Automatic)
                } else {
                    None
                }
            })
            .collect();
        let placement_veto = self.shared_placement_veto(eventual_requests);
        let wave_count = actions
            .seats
            .iter()
            .map(|queue| queue.as_slice().len())
            .max()
            .unwrap_or(0);
        let mut remote_reservations = BTreeSet::new();
        for ordinal in 0..wave_count {
            let wave: Vec<_> = seats
                .iter()
                .map(|&seat| {
                    if stopwatch.frozen[seat.index()] {
                        None
                    } else {
                        actions.for_seat(seat).as_slice().get(ordinal).copied()
                    }
                })
                .collect();
            for &seat in seats {
                if wave[seat.index()] != Some(ManualAction::PlaceBomb)
                    || first_bombs[seat.index()] != Some(ordinal)
                    || !self.actors[seat.index()].alive
                {
                    continue;
                }
                let owner = self.actors[seat.index()].id;
                if let Some(id) = self.world.bombs.values().find_map(|bomb| {
                    (bomb.owner() == owner && matches!(bomb.trigger(), BombTrigger::Remote))
                        .then_some(bomb.id())
                }) {
                    remote_targets[seat.index()] = Some(id);
                    remote_reservations.insert(id);
                }
            }
            let punches: Vec<_> = seats
                .iter()
                .map(|&seat| {
                    (wave[seat.index()] == Some(ManualAction::UseEquipment)).then_some(seat)
                })
                .collect();
            self.resolve_punch_requests(&punches, tick, &remote_reservations);
            let placements: Vec<_> = seats
                .iter()
                .map(|&seat| {
                    (wave[seat.index()] == Some(ManualAction::PlaceBomb)
                        && first_bombs[seat.index()] == Some(ordinal)
                        && remote_targets[seat.index()].is_none())
                    .then_some(PlacementOrigin::Manual)
                })
                .collect();
            self.resolve_placements(
                &placements,
                &placement_veto,
                tick,
                stopwatch.pause_timed,
                events,
            );
        }
        let automatic: Vec<_> = seats
            .iter()
            .map(|&seat| {
                (first_bombs[seat.index()].is_none()
                    && self.actors[seat.index()].alive
                    && self.actors[seat.index()].effects.forces_bomb_drops(tick))
                .then_some(PlacementOrigin::Automatic)
            })
            .collect();
        self.resolve_placements(
            &automatic,
            &placement_veto,
            tick,
            stopwatch.pause_timed,
            events,
        );
        remote_targets
    }

    fn shared_placement_veto(&self, requests: impl AsRef<[Option<PlacementOrigin>]>) -> Vec<bool> {
        let requests = requests.as_ref();
        let targets: Vec<_> = self
            .player_count
            .seats()
            .iter()
            .map(|&seat| {
                let actor = &self.actors[seat.index()];
                let cell = actor.position.cell()?;
                let eligible = requests[seat.index()].is_some()
                    && actor.alive
                    && self.live_bomb_count(seat) < usize::from(actor.capacity)
                    && self.world.arena.terrain(cell) == Some(Terrain::Floor)
                    && !self.world.arena.blocks().contains(&cell);
                let eligible = eligible && !self.world.oil.unlit(cell);
                eligible.then_some(cell)
            })
            .collect();
        targets
            .iter()
            .map(|target| {
                target.is_some()
                    && targets
                        .iter()
                        .filter(|candidate| *candidate == target)
                        .count()
                        > 1
            })
            .collect()
    }

    fn resolve_placements(
        &mut self,
        requests: &[Option<PlacementOrigin>],
        placement_veto: &[bool],
        tick: Tick,
        timed_paused: bool,
        events: &mut Vec<StepEvent>,
    ) {
        let mut targets = vec![None; self.actors.len()];
        for &seat in self.player_count.seats() {
            let actor = &self.actors[seat.index()];
            let Some(cell) = actor.position.cell() else {
                continue;
            };
            let has_capacity = self.live_bomb_count(seat) < usize::from(actor.capacity);
            let cell_is_free = self.world.arena.terrain(cell) == Some(Terrain::Floor)
                && !self.world.arena.blocks().contains(&cell)
                && !self.world.oil.unlit(cell)
                && !self.world.bombs.values().any(|bomb| bomb.cell() == cell);
            if requests[seat.index()].is_some()
                && !placement_veto[seat.index()]
                && actor.alive
                && has_capacity
                && cell_is_free
            {
                targets[seat.index()] = Some(cell);
            }
        }
        for &seat in self.player_count.seats() {
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
            events.push(StepEvent::bomb(
                tick,
                self.bomb_event_audience_for(bomb.owner(), tick),
                StepEventKind::BombPlaced(BombView::from(bomb)),
                bomb,
                timed_paused,
            ));
        }
    }

    fn next_bomb_id(&self) -> Option<EntityId> {
        let mut candidate = self.next_entity_id;
        while let Ok(raw) = u32::try_from(candidate) {
            let id = EntityId(raw);
            if !self.actors.iter().any(|actor| actor.id == id)
                && !self.world.bombs.contains_key(&id)
                && !self
                    .lairs
                    .as_ref()
                    .is_some_and(|lairs| lairs.contains_id(id))
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

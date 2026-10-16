use std::collections::BTreeMap;

use super::{InputAction, RoundResult, SeatInput, State, StepError, StepReport};
use crate::game::arena::Terrain;
use crate::game::bombs::Bomb;
use crate::game::events::{StepEvent, StepEventKind};
use crate::game::geometry::MovementTrace;
use crate::game::movement;
use crate::game::simulation::DrawReason;
use crate::game::types::{CellId, EntityId, SeatId, Tick};
use crate::game::view::BombView;

impl State {
    pub fn step(&mut self, inputs: &[SeatInput]) -> Result<StepReport, StepError> {
        if inputs.len() > super::MAX_INPUTS_PER_TICK {
            return Err(StepError::TooManyInputs {
                count: inputs.len(),
                maximum: super::MAX_INPUTS_PER_TICK,
            });
        }
        let next_tick = Tick(self.tick.0.checked_add(1).ok_or(StepError::TickOverflow)?);
        if self.result.is_some() {
            self.tick = next_tick;
            return Ok(StepReport {
                tick: next_tick,
                traces: self.stationary_traces(),
                events: Vec::new(),
            });
        }

        let mut events = Vec::new();
        self.expire_flames(next_tick);
        let placement_requests = self.apply_inputs(inputs);
        let traces = self.move_living();
        self.kill_on_existing_flames(&traces, next_tick, &mut events);
        self.resolve_placements(placement_requests, next_tick, &mut events);
        self.resolve_explosions(next_tick, &mut events);
        self.kill_on_final_flames(next_tick, &mut events);
        self.running_updates += 1;
        if let Some(result) = self.decide_result() {
            events.push(StepEvent::new(next_tick, StepEventKind::RoundEnded(result)));
        }
        self.tick = next_tick;
        Ok(StepReport {
            tick: next_tick,
            traces,
            events,
        })
    }

    fn apply_inputs(&mut self, inputs: &[SeatInput]) -> [bool; 2] {
        let mut placements = [false; 2];
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
                    placements[input.seat.index()] = false;
                }
                InputAction::PlaceBomb => placements[input.seat.index()] = true,
            }
            if actor.held.active() != previous {
                actor.pending_turn = None;
            }
            if actor.held.active().is_none() {
                actor.residual = 0;
            }
        }
        placements
    }

    fn move_living(&mut self) -> [MovementTrace; 2] {
        self.actors.each_mut().map(|actor| {
            if actor.alive {
                movement::advance_actor(&self.world, actor)
            } else {
                MovementTrace::new(actor.position)
            }
        })
    }

    fn kill_on_existing_flames(
        &mut self,
        traces: &[MovementTrace; 2],
        tick: Tick,
        events: &mut Vec<StepEvent>,
    ) {
        for (actor, trace) in self.actors.iter_mut().zip(traces) {
            if actor.alive && trace_hits_hazards(trace, &self.world.hazards) {
                actor.kill();
                events.push(StepEvent::new(
                    tick,
                    StepEventKind::ActorEliminated {
                        id: actor.id,
                        seat: actor.seat,
                    },
                ));
            }
        }
    }

    fn kill_on_final_flames(&mut self, tick: Tick, events: &mut Vec<StepEvent>) {
        for actor in &mut self.actors {
            if actor.alive
                && self
                    .world
                    .hazards
                    .keys()
                    .any(|cell| movement::position_overlaps_cell(actor.position, *cell))
            {
                actor.kill();
                events.push(StepEvent::new(
                    tick,
                    StepEventKind::ActorEliminated {
                        id: actor.id,
                        seat: actor.seat,
                    },
                ));
            }
        }
    }

    fn resolve_placements(&mut self, requests: [bool; 2], tick: Tick, events: &mut Vec<StepEvent>) {
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
            let Some(detonates_at) = tick.0.checked_add(self.rules.bomb_fuse_ticks()).map(Tick)
            else {
                continue;
            };
            let Some(id) = self.next_bomb_id() else {
                continue;
            };
            let actor = &self.actors[seat.index()];
            let bomb = Bomb {
                id,
                cell,
                owner: actor.id,
                range: actor.range,
                placed_at: tick,
                detonates_at,
            };
            self.insert_bomb(bomb);
            events.push(StepEvent::new(
                tick,
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

    fn decide_result(&mut self) -> Option<RoundResult> {
        self.result = match [self.actors[0].alive, self.actors[1].alive] {
            [true, false] => Some(RoundResult::Winner(SeatId::One)),
            [false, true] => Some(RoundResult::Winner(SeatId::Two)),
            [false, false] => Some(RoundResult::Draw(DrawReason::MutualElimination)),
            [true, true] if self.running_updates >= self.rules.round_ticks() => {
                Some(RoundResult::Draw(DrawReason::Timeout))
            }
            [true, true] => None,
        };
        self.result
    }

    fn stationary_traces(&self) -> [MovementTrace; 2] {
        self.actors
            .each_ref()
            .map(|actor| MovementTrace::new(actor.position))
    }
}

pub(crate) fn trace_hits_hazards(trace: &MovementTrace, hazards: &BTreeMap<CellId, Tick>) -> bool {
    hazards.keys().any(|cell| trace.overlaps_cell(*cell))
}

impl super::Actor {
    fn kill(&mut self) {
        self.alive = false;
        self.held.clear();
        self.residual = 0;
        self.pending_turn = None;
    }
}

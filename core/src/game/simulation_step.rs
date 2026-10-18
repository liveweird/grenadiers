use std::collections::BTreeMap;

use super::{RoundResult, SeatInput, State, StepError, StepReport};
use crate::game::bombs::{BlastKind, BombTrigger};
use crate::game::damage::{DamageOutcome, DamagePreventionSource, apply_damage};
use crate::game::events::{EventAudience, StepEvent, StepEventKind};
use crate::game::geometry::MovementTrace;
use crate::game::movement;
use crate::game::simulation::DrawReason;
use crate::game::types::{CellId, SeatId, Tick};

impl State {
    pub fn step(&mut self, inputs: &[SeatInput]) -> Result<StepReport, StepError> {
        if inputs.len() > super::MAX_INPUTS_PER_TICK {
            return Err(StepError::TooManyInputs {
                count: inputs.len(),
                maximum: super::MAX_INPUTS_PER_TICK,
            });
        }
        let next_tick = Tick(self.tick.0.checked_add(1).ok_or(StepError::TickOverflow)?);
        if let Some(id) = self.world.bombs.values().find_map(|bomb| {
            (!matches!(bomb.trigger(), BombTrigger::Timed { .. })).then_some(bomb.id())
        }) {
            return Err(StepError::UnsupportedBombTrigger(id));
        }
        if let Some(id) = self
            .world
            .bombs
            .values()
            .find_map(|bomb| (bomb.blast() != BlastKind::Standard).then_some(bomb.id()))
        {
            return Err(StepError::UnsupportedBombBlast(id));
        }
        if self.result.is_some() {
            self.tick = next_tick;
            return Ok(StepReport {
                tick: next_tick,
                traces: self.stationary_traces(),
                events: Vec::new(),
            });
        }

        // Timed pickup collection is the only fallible mutation in a running step. Near the
        // absolute tick horizon, retain a transaction snapshot only when that failure is
        // possible, rather than cloning every deterministic update.
        let rollback = self
            .world
            .pickups
            .iter()
            .filter_map(|(_, kind)| kind.timed_effect())
            .any(|effect| next_tick.0.checked_add(effect.duration_ticks()).is_none())
            .then(|| self.clone());
        let mut events = Vec::new();
        let mut shield_prevented = [false; 2];
        self.expire_flames(next_tick);
        self.expire_actor_effects(self.tick, next_tick);
        self.world.expose_flame_free_pickups();
        let manual_actions = self.apply_inputs(inputs);
        let traces = self.move_living(next_tick);
        self.kill_on_existing_flames(&traces, next_tick, &mut events, &mut shield_prevented);
        self.resolve_classic_manual_actions(&manual_actions, next_tick, &mut events);
        self.resolve_explosions(next_tick, &mut events)?;
        self.kill_on_final_flames(next_tick, &mut events, &mut shield_prevented);
        if let Err(error) = self.collect_pickups(next_tick, &mut events) {
            debug_assert!(matches!(error, StepError::EffectExpiryOverflow(_)));
            if let Some(snapshot) = rollback {
                *self = snapshot;
            }
            return Err(error);
        }
        self.running_updates += 1;
        if let Some(result) = self.decide_result() {
            events.push(StepEvent::new(
                next_tick,
                EventAudience::Both,
                StepEventKind::RoundEnded(result),
            ));
        }
        self.tick = next_tick;
        Ok(StepReport {
            tick: next_tick,
            traces,
            events,
        })
    }

    fn move_living(&mut self, tick: Tick) -> [MovementTrace; 2] {
        let turn_assist = self.rules.turn_assist();
        self.actors.each_mut().map(|actor| {
            if actor.alive {
                movement::advance_actor(&self.world, actor, turn_assist, tick)
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
        shield_prevented: &mut [bool; 2],
    ) {
        for (index, trace) in traces.iter().enumerate() {
            if self.actors[index].alive && trace_hits_hazards(trace, &self.world.hazards) {
                self.damage_actor(index, tick, events, shield_prevented);
            }
        }
    }

    fn kill_on_final_flames(
        &mut self,
        tick: Tick,
        events: &mut Vec<StepEvent>,
        shield_prevented: &mut [bool; 2],
    ) {
        for index in 0..self.actors.len() {
            if self.actors[index].alive
                && self.world.hazards.keys().any(|cell| {
                    movement::position_overlaps_cell(self.actors[index].position, *cell)
                })
            {
                self.damage_actor(index, tick, events, shield_prevented);
            }
        }
    }

    fn expire_actor_effects(&mut self, previous_tick: Tick, tick: Tick) {
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

    fn damage_actor(
        &mut self,
        index: usize,
        tick: Tick,
        events: &mut Vec<StepEvent>,
        shield_prevented: &mut [bool; 2],
    ) {
        let actor = &mut self.actors[index];
        let outcome = apply_damage(
            &mut actor.inventory,
            &actor.effects,
            &mut actor.damage_grace,
            tick,
        );
        match outcome {
            DamageOutcome::IgnoredGrace => return,
            DamageOutcome::AbsorbedShield => {
                if !shield_prevented[index] {
                    events.push(StepEvent::new(
                        tick,
                        EventAudience::Both,
                        StepEventKind::DamagePrevented {
                            id: actor.id,
                            seat: actor.seat,
                            source: DamagePreventionSource::Shield,
                        },
                    ));
                    shield_prevented[index] = true;
                }
                return;
            }
            DamageOutcome::AbsorbedHeart { .. } => {
                events.push(StepEvent::new(
                    tick,
                    EventAudience::Both,
                    StepEventKind::DamagePrevented {
                        id: actor.id,
                        seat: actor.seat,
                        source: DamagePreventionSource::Heart,
                    },
                ));
                return;
            }
            DamageOutcome::Eliminated => {}
        }
        actor.kill();
        events.push(StepEvent::new(
            tick,
            EventAudience::Both,
            StepEventKind::ActorEliminated {
                id: actor.id,
                seat: actor.seat,
            },
        ));
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
        self.inventory.clear();
        self.effects.clear();
        self.damage_grace = None;
    }
}

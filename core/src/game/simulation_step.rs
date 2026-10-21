use std::collections::BTreeMap;

use super::{RoundResult, SeatInput, State, StepError, StepReport};
use crate::game::damage::{DamageOutcome, DamagePreventionSource, apply_damage};
use crate::game::events::{EventAudience, StepEvent, StepEventKind};
use crate::game::geometry::MovementTrace;
use crate::game::movement;
use crate::game::simulation::DrawReason;
use crate::game::types::{CellId, SeatId, Tick};
use std::collections::BTreeSet;

impl State {
    pub fn step(&mut self, inputs: &[SeatInput]) -> Result<StepReport, StepError> {
        if inputs.len() > super::MAX_INPUTS_PER_TICK {
            return Err(StepError::TooManyInputs {
                count: inputs.len(),
                maximum: super::MAX_INPUTS_PER_TICK,
            });
        }
        let next_tick = Tick(self.tick.0.checked_add(1).ok_or(StepError::TickOverflow)?);
        if let Some(id) = super::validation::unsupported_trigger(self.rules, &self.world) {
            return Err(StepError::UnsupportedBombTrigger(id));
        }
        if let Some(id) = super::validation::unsupported_blast(self.rules, &self.world) {
            return Err(StepError::UnsupportedBombBlast(id));
        }
        if let Some(owner) = super::validation::duplicate_remote_owner(&self.world) {
            return Err(StepError::DuplicateRemoteOwner(owner));
        }
        if let Some(id) = super::validation::unsupported_kick_history(self.rules, &self.world) {
            return Err(StepError::UnsupportedBombKickHistory(id));
        }
        if let Some(id) = super::validation::unsupported_punch_history(self.rules, &self.world) {
            return Err(StepError::UnsupportedBombPunchHistory(id));
        }
        if self.result.is_some() {
            self.tick = next_tick;
            return Ok(StepReport {
                tick: next_tick,
                traces: self.stationary_traces(),
                events: Vec::new(),
            });
        }

        let stopwatch = self.stopwatch_context(next_tick);
        self.preflight_timed_pause(stopwatch)?;

        // Timed-pause overflow is preflight before mutation. Timed pickup collection remains
        // the only later fallible mutation, so retain a transaction snapshot only when that
        // failure is possible rather than cloning every deterministic update.
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
        self.apply_timed_pause(stopwatch);
        self.world.expose_flame_free_pickups();
        let motion = crate::game::bomb_motion::advance(&mut self.world, &self.actors, next_tick);
        for actor in &mut self.actors {
            actor.bomb_grace.retain(|id| !motion.moved.contains(id));
        }
        let manual_actions = self.apply_inputs(inputs);
        self.clear_frozen_movement(stopwatch.frozen);
        let alive_at_movement_start = self.actors.each_ref().map(|actor| actor.alive);
        let grace_at_movement_start = self.actors.each_ref().map(|actor| actor.bomb_grace.clone());
        let (traces, kick_requests) =
            self.move_living(next_tick, stopwatch.frozen, &motion.moving_at_start);
        let contact_targets = super::contacts::accepted_contact_ids(
            &self.world,
            &traces,
            alive_at_movement_start,
            &grace_at_movement_start,
        );
        self.resolve_kick_requests(kick_requests);
        self.kill_on_existing_flames(&traces, next_tick, &mut events, &mut shield_prevented);
        let remote_targets =
            self.resolve_manual_actions(&manual_actions, next_tick, stopwatch, &mut events);
        self.resolve_explosions(
            next_tick,
            remote_targets,
            contact_targets,
            stopwatch.pause_timed,
            &mut events,
        )?;
        self.kill_on_final_flames(next_tick, &mut events, &mut shield_prevented);
        let teleport_requests = match self.collect_pickups(next_tick, &mut events) {
            Ok(requests) => requests,
            Err(error) => {
                debug_assert!(matches!(error, StepError::EffectExpiryOverflow(_)));
                if let Some(snapshot) = rollback {
                    *self = snapshot;
                }
                return Err(error);
            }
        };
        self.commit_teleport(teleport_requests, next_tick);
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

    fn move_living(
        &mut self,
        tick: Tick,
        frozen: [bool; 2],
        moving_at_start: &BTreeSet<crate::game::EntityId>,
    ) -> (
        [MovementTrace; 2],
        [Option<(crate::game::EntityId, crate::game::Direction)>; 2],
    ) {
        let turn_assist = self.rules.turn_assist();
        let slow_foot_enabled = self.rules.enabled_items().contains_slow_foot();
        let wall_pass_enabled = self.rules.enabled_items().contains_wall_pass();
        let kick_rule_enabled = self.rules.enabled_items().contains_kick();
        let mut requests = [None; 2];
        let mut index = 0;
        let traces = self.actors.each_mut().map(|actor| {
            let current = index;
            index += 1;
            if actor.alive && !frozen[current] {
                let can_kick = kick_rule_enabled && actor.inventory().can_kick();
                let config = movement::AdvanceConfig {
                    turn_assist,
                    tick,
                    slow_foot_enabled,
                    wall_pass_enabled,
                    kick_enabled: can_kick,
                    moving_at_start,
                };
                let outcome = movement::advance_actor(&self.world, actor, &config);
                requests[current] = outcome.kick;
                outcome.trace
            } else {
                MovementTrace::new(actor.position)
            }
        });
        (traces, requests)
    }

    fn resolve_kick_requests(
        &mut self,
        requests: [Option<(crate::game::EntityId, crate::game::Direction)>; 2],
    ) {
        let contested = requests[0]
            .zip(requests[1])
            .filter(|(left, right)| left.0 == right.0)
            .map(|(left, _)| left.0);
        for seat in SeatId::ALL {
            let Some((id, direction)) = requests[seat.index()] else {
                continue;
            };
            if contested == Some(id) && seat != self.assignment.priority_seat() {
                continue;
            }
            let Some(bomb) = self.world.bombs.get_mut(&id) else {
                continue;
            };
            if bomb.motion().is_none()
                && !matches!(bomb.trigger(), crate::game::BombTrigger::Contact)
            {
                bomb.arm_kick(direction);
            }
        }
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
                    let audience = if actor.effects.hidden_from_opponents(tick) {
                        EventAudience::only(actor.seat)
                    } else {
                        EventAudience::Both
                    };
                    events.push(StepEvent::new(
                        tick,
                        audience,
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
                let audience = if actor.effects.hidden_from_opponents(tick) {
                    EventAudience::only(actor.seat)
                } else {
                    EventAudience::Both
                };
                events.push(StepEvent::new(
                    tick,
                    audience,
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
        self.soft_wall_grace.clear();
        self.effects.clear();
        self.damage_grace = None;
    }
}

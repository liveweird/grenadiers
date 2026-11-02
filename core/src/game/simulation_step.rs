use super::{SeatInput, State, StepError, StepReport};
use crate::game::damage::{DamageOutcome, DamagePreventionSource, apply_damage};
use crate::game::events::{EventAudience, StepEvent, StepEventKind};
use crate::game::geometry::MovementTrace;
use crate::game::movement;
#[cfg(test)]
use crate::game::types::CellId;
use crate::game::types::Tick;
#[cfg(test)]
use std::collections::BTreeMap;

impl State {
    pub fn step(&mut self, inputs: &[SeatInput]) -> Result<StepReport, StepError> {
        if self.rules.version() >= 10 {
            let mut staged = self.clone();
            let report = staged.step_tick(inputs)?;
            *self = staged;
            return Ok(report);
        }
        self.step_tick(inputs)
    }

    fn step_tick(&mut self, inputs: &[SeatInput]) -> Result<StepReport, StepError> {
        if inputs.len() > super::MAX_INPUTS_PER_TICK {
            return Err(StepError::TooManyInputs {
                count: inputs.len(),
                maximum: super::MAX_INPUTS_PER_TICK,
            });
        }
        if let Some(input) = inputs
            .iter()
            .find(|input| !self.player_count.contains(input.seat))
        {
            return Err(StepError::SeatNotParticipating(input.seat));
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
        if let Some(id) = super::validation::unsupported_shove_history(self) {
            return Err(StepError::UnsupportedShoveHistory(id));
        }
        if !self.rules.enabled_items().contains_punch()
            && inputs
                .iter()
                .any(|input| input.action == super::InputAction::UseEquipment)
        {
            return Err(StepError::UnsupportedEquipmentAction);
        }
        if self.result.is_some() {
            self.tick = next_tick;
            return Ok(StepReport {
                tick: next_tick,
                traces: self.stationary_traces(),
                events: Vec::new(),
                forced_movements: Vec::new(),
            });
        }

        let stopwatch = self.stopwatch_context(next_tick);
        self.preflight_timed_pause(&stopwatch)?;
        let storm_expiry = self.preflight_storm_slow(next_tick)?;

        let lair_due = self.preflight_lair_attempt(next_tick)?;

        // Rules 8 adds a fallible post-settlement entity allocation, so it retains a transaction
        // snapshot. Closed profiles keep the narrower timed-pickup rollback condition.
        let rollback = (self.rules.version() >= 8 && self.rules.version() < 10)
            .then(|| self.clone())
            .or_else(|| {
                (self.rules.version() < 10
                    && self
                        .world
                        .pickups
                        .iter()
                        .filter_map(|(_, kind)| kind.timed_effect())
                        .any(|effect| next_tick.0.checked_add(effect.duration_ticks()).is_none()))
                .then(|| self.clone())
            });
        let cloud_movement = self.advance_storm_cloud();
        let mut storm_contacts = self.storm_contacts_at_start(cloud_movement);
        let mut events = Vec::new();
        let mut shield_prevented = vec![false; self.actors.len()];
        self.expire_flames(next_tick);
        self.expire_actor_effects(self.tick, next_tick);
        self.expire_storm_slow(next_tick);
        self.apply_timed_pause(&stopwatch);
        self.world.expose_flame_free_pickups(next_tick);
        self.reveal_safe_oil(next_tick);
        self.advance_spike_pits_at_start(next_tick, &mut events, &mut shield_prevented)?;
        let motion = crate::game::bomb_motion::advance(&mut self.world, &self.actors, next_tick);
        for actor in &mut self.actors {
            actor.bomb_grace.retain(|id| !motion.moved.contains(id));
        }
        self.reconcile_contact_punch_latches();
        let manual_actions = self.apply_inputs(inputs);
        self.clear_frozen_movement(&stopwatch.frozen);
        let (mob_contact_targets, mut contact_damaged) = match self.advance_lair_phase(
            next_tick,
            stopwatch.pause_timed,
            &mut events,
            &mut shield_prevented,
        ) {
            Ok(phase) => phase,
            Err(error) => {
                if let Some(snapshot) = rollback {
                    *self = snapshot;
                }
                return Err(error);
            }
        };
        let alive_at_movement_start: Vec<_> = self.actors.iter().map(|actor| actor.alive).collect();
        let grace_at_movement_start: Vec<_> = self
            .actors
            .iter()
            .map(|actor| actor.bomb_grace.clone())
            .collect();
        let (outcomes, kick_requests) =
            self.move_living(next_tick, &stopwatch.frozen, &motion.moving_at_start);
        let contact_punches = self.latch_contact_punches(&outcomes);
        let traces: Vec<_> = outcomes
            .iter()
            .map(|outcome| outcome.trace.clone())
            .collect();
        self.resolve_kick_requests(kick_requests);
        let forced_movements = self.resolve_shove(&outcomes, next_tick);
        self.advance_spike_pits_for_player_traces(
            &traces,
            &forced_movements,
            next_tick,
            &mut events,
            &mut shield_prevented,
        )?;
        self.include_storm_movement_contacts(&mut storm_contacts, &traces, &forced_movements);
        let mut contact_targets = super::contacts::accepted_contact_ids(
            &self.world,
            &traces,
            &forced_movements,
            &alive_at_movement_start,
            &grace_at_movement_start,
        );
        contact_targets.extend(mob_contact_targets);
        self.apply_lair_trace_contacts(
            &traces,
            &forced_movements,
            &mut contact_damaged,
            next_tick,
            &mut events,
            &mut shield_prevented,
        );
        self.kill_on_existing_flames(
            &traces,
            &forced_movements,
            next_tick,
            &mut events,
            &mut shield_prevented,
        );
        let remote_targets =
            self.resolve_manual_actions(&manual_actions, next_tick, &stopwatch, &mut events);
        self.resolve_contact_punches(&contact_punches, next_tick, &remote_targets);
        self.resolve_explosions(
            next_tick,
            &remote_targets,
            contact_targets,
            stopwatch.pause_timed,
            &mut events,
        )?;
        self.reconcile_contact_punch_latches();
        if let Err(error) = self.kill_lairs_on_fire(next_tick) {
            if let Some(snapshot) = rollback {
                *self = snapshot;
            }
            return Err(error);
        }
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
        let teleported = self
            .actors
            .iter()
            .enumerate()
            .filter(|(_, actor)| actor.last_teleport_tick == Some(next_tick))
            .map(|(index, actor)| (index, actor.position))
            .collect::<Vec<_>>();
        self.advance_spike_pits_for_positions(
            &teleported,
            next_tick,
            &mut events,
            &mut shield_prevented,
        )?;
        self.apply_lair_endpoint_contacts(
            &mut contact_damaged,
            next_tick,
            &mut events,
            &mut shield_prevented,
        );
        self.include_storm_endpoint_contacts(&mut storm_contacts);
        self.apply_storm_contacts(&storm_contacts, storm_expiry);
        self.running_updates += 1;
        if let Some(result) = self.decide_result() {
            events.push(StepEvent::new(
                next_tick,
                EventAudience::all(self.player_count),
                StepEventKind::RoundEnded(result),
            ));
        }
        if lair_due && self.result.is_none() {
            let allocator_before = self.next_entity_id;
            let ids = self.available_lair_entity_ids(crate::game::MAX_LAIRS);
            let spawned = match self.spawn_from_lairs(next_tick, &ids) {
                Ok(spawned) => spawned,
                Err(error) => {
                    if let Some(snapshot) = rollback {
                        *self = snapshot;
                    }
                    return Err(error);
                }
            };
            self.advance_spike_pits_for_newborns(&ids[..spawned], next_tick)?;
            self.next_entity_id = if spawned == 0 {
                allocator_before
            } else {
                u64::from(ids[spawned - 1].0) + 1
            };
        }
        self.tick = next_tick;
        Ok(StepReport {
            tick: next_tick,
            traces,
            events,
            forced_movements,
        })
    }

    pub(super) fn kill_on_existing_flames(
        &mut self,
        traces: &[MovementTrace],
        forced_movements: &[super::contract::ForcedMovement],
        tick: Tick,
        events: &mut Vec<StepEvent>,
        shield_prevented: &mut [bool],
    ) {
        for (index, trace) in traces.iter().enumerate() {
            let forced_hit = forced_movements.iter().any(|forced| {
                forced.seat.index() == index
                    && self
                        .world
                        .fire_cells(tick)
                        .any(|cell| forced.segment.overlaps_cell(cell))
            });
            if self.actors[index].alive
                && (self
                    .world
                    .fire_cells(tick)
                    .any(|cell| trace.overlaps_cell(cell))
                    || forced_hit)
            {
                self.damage_actor(index, tick, events, shield_prevented);
            }
        }
    }

    fn kill_on_final_flames(
        &mut self,
        tick: Tick,
        events: &mut Vec<StepEvent>,
        shield_prevented: &mut [bool],
    ) {
        for index in 0..self.actors.len() {
            if self.actors[index].alive
                && self
                    .world
                    .committed_fire_cells()
                    .any(|cell| movement::position_overlaps_cell(self.actors[index].position, cell))
            {
                self.damage_actor(index, tick, events, shield_prevented);
            }
        }
    }

    pub(super) fn damage_actor(
        &mut self,
        index: usize,
        tick: Tick,
        events: &mut Vec<StepEvent>,
        shield_prevented: &mut [bool],
    ) {
        let broadcast = EventAudience::all(self.player_count);
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
                        broadcast
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
                    broadcast
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
            broadcast,
            StepEventKind::ActorEliminated {
                id: actor.id,
                seat: actor.seat,
            },
        ));
    }

    pub(super) fn stationary_traces(&self) -> Vec<MovementTrace> {
        self.actors
            .iter()
            .map(|actor| MovementTrace::new(actor.position))
            .collect()
    }
}

#[cfg(test)]
pub(crate) fn trace_hits_hazards(trace: &MovementTrace, hazards: &BTreeMap<CellId, Tick>) -> bool {
    hazards.keys().any(|cell| trace.overlaps_cell(*cell))
}

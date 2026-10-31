use std::collections::BTreeSet;

use super::{State, StepError};
use crate::game::events::StepEvent;
use crate::game::geometry::MovementTrace;
use crate::game::simulation::contract::ForcedMovement;
use crate::game::types::{EntityId, Tick};

impl State {
    pub(super) fn preflight_lair_attempt(&self, tick: Tick) -> Result<bool, StepError> {
        let due = self.lairs.as_ref().is_some_and(|lairs| lairs.due(tick));
        if !due {
            return Ok(false);
        }
        if self.rules.version() == 8 {
            let interval = match self.rules.lair_config() {
                crate::room::LairConfig::Enabled(settings) => settings.spawn_ticks(),
                crate::room::LairConfig::Disabled => 0,
            };
            tick.0
                .checked_add(interval)
                .ok_or(StepError::LairTimerOverflow)?;
        }
        Ok(true)
    }

    pub(super) fn available_lair_entity_ids(&self, count: usize) -> Vec<EntityId> {
        let mut candidate = self.next_entity_id;
        let mut ids = Vec::with_capacity(count);
        while ids.len() < count {
            let Ok(raw) = u32::try_from(candidate) else {
                break;
            };
            let id = EntityId(raw);
            candidate += 1;
            if self.actors.iter().any(|actor| actor.id == id)
                || self.world.bombs.contains_key(&id)
                || self
                    .lairs
                    .as_ref()
                    .is_some_and(|lairs| lairs.contains_id(id))
            {
                continue;
            }
            ids.push(id);
        }
        ids
    }

    pub(super) fn advance_lair_phase(
        &mut self,
        tick: Tick,
        frozen: bool,
        events: &mut Vec<StepEvent>,
        shield_prevented: &mut [bool],
    ) -> Result<(BTreeSet<EntityId>, Vec<bool>), StepError> {
        let mut damaged = vec![false; self.actors.len()];
        if self.lairs.is_none() {
            return Ok((BTreeSet::new(), damaged));
        }
        self.kill_on_existing_flames(
            &self.stationary_traces(),
            &[],
            tick,
            events,
            shield_prevented,
        );
        let target_order: Vec<_> = self
            .assignment
            .priority_order(self.player_count)
            .map(|seat| seat.index())
            .collect();
        let lairs = self.lairs.as_mut().expect("checked above");
        lairs
            .kill_on_fire(&self.world, tick)
            .map_err(|()| StepError::LairTimerOverflow)?;
        let traces = lairs.advance(&self.world, &self.actors, &target_order, frozen, tick);
        let mine_targets = lairs.mine_contacts(&self.world, &traces);
        let contacts: Vec<_> = (0..self.actors.len())
            .map(|index| {
                self.actors[index].alive
                    && (crate::game::lairs::traces_contact_actor(
                        &traces,
                        self.actors[index].position,
                    ) || crate::game::lairs::actor_position_contacts_mobs(
                        self.actors[index].position,
                        lairs.mobs(),
                    ))
            })
            .collect();
        for (index, contact) in contacts.into_iter().enumerate() {
            if contact {
                self.damage_actor(index, tick, events, shield_prevented);
                damaged[index] = true;
            }
        }
        Ok((mine_targets, damaged))
    }

    pub(super) fn apply_lair_trace_contacts(
        &mut self,
        traces: &[MovementTrace],
        forced_movements: &[ForcedMovement],
        damaged: &mut [bool],
        tick: Tick,
        events: &mut Vec<StepEvent>,
        shield_prevented: &mut [bool],
    ) {
        let Some(lairs) = &self.lairs else { return };
        let contacts: Vec<_> = (0..self.actors.len())
            .map(|index| {
                !damaged[index]
                    && self.actors[index].alive
                    && (crate::game::lairs::actor_trace_contacts_mobs(&traces[index], lairs.mobs())
                        || forced_movements.iter().any(|forced| {
                            forced.seat.index() == index
                                && crate::game::lairs::actor_segment_contacts_mobs(
                                    forced.segment,
                                    lairs.mobs(),
                                )
                        }))
            })
            .collect();
        self.apply_lair_damage(contacts, damaged, tick, events, shield_prevented);
    }

    pub(super) fn apply_lair_endpoint_contacts(
        &mut self,
        damaged: &mut [bool],
        tick: Tick,
        events: &mut Vec<StepEvent>,
        shield_prevented: &mut [bool],
    ) {
        let Some(lairs) = &self.lairs else { return };
        let contacts: Vec<_> = (0..self.actors.len())
            .map(|index| {
                !damaged[index]
                    && self.actors[index].alive
                    && crate::game::lairs::actor_position_contacts_mobs(
                        self.actors[index].position,
                        lairs.mobs(),
                    )
            })
            .collect();
        self.apply_lair_damage(contacts, damaged, tick, events, shield_prevented);
    }

    fn apply_lair_damage(
        &mut self,
        contacts: Vec<bool>,
        damaged: &mut [bool],
        tick: Tick,
        events: &mut Vec<StepEvent>,
        shield_prevented: &mut [bool],
    ) {
        for (index, contact) in contacts.into_iter().enumerate() {
            if contact {
                self.damage_actor(index, tick, events, shield_prevented);
                damaged[index] = true;
            }
        }
    }

    pub(super) fn kill_lairs_on_fire(&mut self, tick: Tick) -> Result<(), StepError> {
        if let Some(lairs) = &mut self.lairs {
            lairs
                .kill_on_fire(&self.world, tick)
                .map_err(|()| StepError::LairTimerOverflow)?;
        }
        Ok(())
    }

    pub(super) fn spawn_from_lairs(
        &mut self,
        tick: Tick,
        ids: &[EntityId],
    ) -> Result<usize, StepError> {
        if self.result.is_some() {
            return Ok(0);
        }
        if let Some(lairs) = &mut self.lairs {
            lairs
                .spawn(&self.world, &self.actors, tick, &mut ids.iter().copied())
                .map_err(|()| StepError::EntityIdOverflow)
        } else {
            Ok(0)
        }
    }
}

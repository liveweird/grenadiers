use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::{BombKnowledge, Observation, ObservedActor, ObservedBombTrigger, WorkBudget};
use crate::game::{EffectKind, EntityId, Equipment, Tick};

pub(super) const REACTIVE_GUARD_ALLOWANCE: u32 = 8_192;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReactivePlantAssumptions {
    pub(super) eligible_bombs: Vec<EntityId>,
    actors: Vec<ActorGuard>,
    pub(super) contact: Option<crate::cpu::MobContactLease>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ActorGuard {
    id: EntityId,
    equipment: Option<Equipment>,
    weapon: Option<crate::game::SpecialWeapon>,
    effects: Vec<(EffectKind, Tick)>,
    damage_grace_expires: Tick,
}

impl ReactivePlantAssumptions {
    pub(super) fn certify(
        observation: &Observation,
        history: &ObservedHazardMemory,
        settled_stop_tick: Tick,
        work: &mut WorkBudget,
    ) -> Option<Self> {
        Self::certify_inner(observation, history, settled_stop_tick, false, false, work)
    }

    pub(super) fn certify_escape(
        observation: &Observation,
        history: &ObservedHazardMemory,
        settled_stop_tick: Tick,
        work: &mut WorkBudget,
    ) -> Option<Self> {
        Self::certify_inner(observation, history, settled_stop_tick, true, true, work)
    }

    fn certify_inner(
        observation: &Observation,
        history: &ObservedHazardMemory,
        settled_stop_tick: Tick,
        allow_flames: bool,
        require_own_bomb: bool,
        work: &mut WorkBudget,
    ) -> Option<Self> {
        work.try_take(1).ok()?;
        let parts = observation.parts();
        if parts.rules.version != 10
            || parts.rules.punch_activation_policy != crate::game::PunchActivationPolicy::ContactV2
            || settled_stop_tick != parts.tick
            || (!allow_flames && !parts.flames.is_empty())
            || !parts.oil.is_empty()
            || !parts.pickups.is_empty()
        {
            return None;
        }
        let actors = actor_guards(observation, work)?;
        let recipient = actors
            .iter()
            .find(|guard| {
                parts
                    .actors
                    .iter()
                    .any(|actor| actor.seat() == parts.recipient && actor.id() == guard.id)
            })?
            .id;
        let BombKnowledge::Visible(bombs) = &parts.bombs else {
            return None;
        };
        let mut eligible_bombs = Vec::new();
        let mut own_bombs = 0_u16;
        for bomb in bombs {
            work.try_take(1).ok()?;
            if !matches!(
                bomb.trigger,
                ObservedBombTrigger::Timed { paused: false, .. }
            ) || !history.is_stationary_candidate(bomb.id, parts.tick, bomb.cell)
            {
                return None;
            }
            if bomb.owner != recipient {
                eligible_bombs.push(bomb.id);
            } else {
                own_bombs = own_bombs.checked_add(1)?;
                if require_own_bomb {
                    eligible_bombs.push(bomb.id);
                }
            }
        }
        (!eligible_bombs.is_empty() && (!require_own_bomb || own_bombs > 0)).then_some(Self {
            eligible_bombs,
            actors,
            contact: None,
        })
    }

    pub(super) fn with_contact(mut self, contact: crate::cpu::MobContactLease) -> Self {
        self.contact = Some(contact);
        self
    }

    pub(super) fn matches(&self, current: &Observation, work: &mut WorkBudget) -> bool {
        if work.try_take(1).is_err()
            || !current.parts().pickups.is_empty()
            || current.parts().actors.len() != self.actors.len()
        {
            return false;
        }
        let Some(current_guards) = actor_guards(current, work) else {
            return false;
        };
        actor_guards_match(&self.actors, &current_guards, current.parts().tick)
            && self
                .contact
                .as_ref()
                .is_some_and(|contact| contact.observation_fits(current, work))
    }
}

fn actor_guards(observation: &Observation, work: &mut WorkBudget) -> Option<Vec<ActorGuard>> {
    let mut guards = Vec::with_capacity(observation.parts().actors.len());
    for actor in &observation.parts().actors {
        work.try_take(1).ok()?;
        let ObservedActor::Visible(actor) = actor else {
            return None;
        };
        if actor.alive
            && (matches!(
                actor.equipment,
                Some(Equipment::Kick | Equipment::Punch | Equipment::Bracelet)
            ) || actor.weapon.is_some()
                || actor.effects.iter().any(|effect| {
                    matches!(effect.kind, EffectKind::Stopwatch | EffectKind::Diarrhea)
                })
                || actor.voluntary_actions_frozen)
        {
            return None;
        }
        guards.push(ActorGuard {
            id: actor.id,
            equipment: actor.equipment,
            weapon: actor.weapon,
            effects: actor
                .effects
                .iter()
                .map(|effect| {
                    observation
                        .parts()
                        .tick
                        .0
                        .checked_add(u64::from(effect.remaining_ticks))
                        .map(Tick)
                        .map(|expiry| (effect.kind, expiry))
                })
                .collect::<Option<Vec<_>>>()?,
            damage_grace_expires: if actor.damage_grace_remaining_ticks == 0 {
                Tick(0)
            } else {
                Tick(
                    observation
                        .parts()
                        .tick
                        .0
                        .checked_add(u64::from(actor.damage_grace_remaining_ticks))?,
                )
            },
        });
    }
    guards.sort_unstable_by_key(|guard| guard.id);
    Some(guards)
}

fn actor_guards_match(expected: &[ActorGuard], actual: &[ActorGuard], tick: Tick) -> bool {
    expected.iter().zip(actual).all(|(before, after)| {
        before.id == after.id
            && before.equipment == after.equipment
            && before.weapon == after.weapon
            && (if before.damage_grace_expires <= tick {
                Tick(0)
            } else {
                before.damage_grace_expires
            }) == after.damage_grace_expires
            && before
                .effects
                .iter()
                .filter(|(_, expiry)| *expiry > tick)
                .eq(after.effects.iter())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guard(expiry: Tick) -> ActorGuard {
        ActorGuard {
            id: EntityId(1),
            equipment: None,
            weapon: None,
            effects: Vec::new(),
            damage_grace_expires: expiry,
        }
    }

    #[test]
    fn damage_grace_normal_expiry_is_allowed_but_extension_is_rejected() {
        let sealed = guard(Tick(10));
        assert!(actor_guards_match(
            std::slice::from_ref(&sealed),
            &[guard(Tick(0))],
            Tick(10)
        ));
        assert!(!actor_guards_match(
            std::slice::from_ref(&sealed),
            &[guard(Tick(11))],
            Tick(10)
        ));
        assert!(!actor_guards_match(&[sealed], &[guard(Tick(12))], Tick(9)));
    }
}

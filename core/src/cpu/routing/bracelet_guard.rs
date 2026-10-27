use crate::cpu::{BombKnowledge, Observation, ObservedActor};
use crate::game::{EntityId, Equipment};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BraceletGuard {
    rival: EntityId,
}

impl BraceletGuard {
    pub(crate) const fn new(rival: EntityId) -> Self {
        Self { rival }
    }

    pub(crate) fn allows(self, observation: &Observation) -> bool {
        let parts = observation.parts();
        if !parts.rules.settings.enabled_items().contains_bracelet()
            || !matches!(&parts.bombs, BombKnowledge::Visible(bombs) if bombs.is_empty())
            || !parts.pickups.is_empty()
        {
            return false;
        }
        let recipient_has_bracelet = parts.actors.iter().any(|actor| {
            matches!(actor, ObservedActor::Visible(actor)
                if actor.seat == parts.recipient
                    && actor.alive
                    && actor.equipment == Some(Equipment::Bracelet))
        });
        let rival_is_shoveable = parts.actors.iter().any(|actor| {
            matches!(actor, ObservedActor::Visible(actor)
                if actor.id == self.rival
                    && actor.seat != parts.recipient
                    && actor.alive
                    && actor.equipment != Some(Equipment::Bracelet))
        });
        recipient_has_bracelet && rival_is_shoveable
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::policy::tests::observation;
    use crate::cpu::{
        BombKnowledge, Observation, ObservedActor, ObservedBomb, ObservedBombTrigger,
        ObservedPickup, UnknownActorFacts, UnknownBombFacts, VisibleActor,
    };
    use crate::game::{
        BlastKind, CellId, Direction, EnabledItems, Equipment, PickupKind, Position, SeatId,
        SpeedLevel, Tick,
    };

    use super::*;

    fn guarded_observation() -> Observation {
        let mut parts = observation(CellId::new(7, 7)).into_parts();
        parts.rules.settings = parts
            .rules
            .settings
            .with_enabled_items(EnabledItems::BRACELET);
        parts.actors[0] = ObservedActor::Visible(VisibleActor {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(CellId::new(8, 7)),
            heading: Direction::West,
            speed: SpeedLevel::Base,
            alive: true,
            bomb_capacity: 1,
            bomb_range: 2,
            live_bomb_count: 0,
            equipment: None,
            weapon: None,
            effects: Vec::new(),
            damage_grace_remaining_ticks: 0,
            voluntary_actions_frozen: false,
            last_teleport_tick: None,
            unknown: UnknownActorFacts::ALL,
        });
        let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
            unreachable!()
        };
        cpu.equipment = Some(Equipment::Bracelet);
        Observation::new(parts).unwrap()
    }

    #[test]
    fn guard_requires_bracelet_holder_and_enabled_rule() {
        let valid = guarded_observation();
        let guard = BraceletGuard::new(EntityId(1));
        assert!(guard.allows(&valid));

        let mut no_holder = valid.clone().into_parts();
        let ObservedActor::Visible(cpu) = &mut no_holder.actors[1] else {
            unreachable!()
        };
        cpu.equipment = None;
        assert!(!guard.allows(&Observation::new(no_holder).unwrap()));

        let mut disabled = valid.into_parts();
        disabled.rules.settings = disabled
            .rules
            .settings
            .with_enabled_items(EnabledItems::EMPTY);
        assert!(!guard.allows(&Observation::new(disabled).unwrap()));
    }

    #[test]
    fn guard_rejects_new_bombs_and_exposed_pickups() {
        let valid = guarded_observation();
        let guard = BraceletGuard::new(EntityId(1));
        let mut bomb = valid.clone().into_parts();
        bomb.bombs = BombKnowledge::Visible(vec![ObservedBomb {
            id: EntityId(9),
            placed_at: Tick(10),
            cell: CellId::new(5, 5),
            owner: EntityId(1),
            range: 1,
            trigger: ObservedBombTrigger::Timed {
                remaining_ticks: 30,
                paused: false,
            },
            blast: BlastKind::Standard,
            unknown: UnknownBombFacts::ALL,
        }]);
        assert!(!guard.allows(&Observation::new(bomb).unwrap()));

        let mut pickup = valid.into_parts();
        pickup.pickups.push(ObservedPickup {
            cell: CellId::new(5, 5),
            kind: PickupKind::BombCapacity,
        });
        assert!(!guard.allows(&Observation::new(pickup).unwrap()));
    }
}

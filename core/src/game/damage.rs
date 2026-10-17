use super::effects::TimedEffects;
use super::inventory::Inventory;
use super::types::Tick;

pub const HEART_GRACE_TICKS: u64 = 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamagePreventionSource {
    Heart,
    Shield,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DamageGrace {
    received_at: Tick,
}

impl DamageGrace {
    #[must_use]
    pub(crate) const fn received_at(self) -> Tick {
        self.received_at
    }

    const fn received(at: Tick) -> Self {
        Self { received_at: at }
    }

    #[must_use]
    pub fn is_active(self, at: Tick) -> bool {
        at.0.checked_sub(self.received_at.0)
            .is_some_and(|elapsed| elapsed < HEART_GRACE_TICKS)
    }

    #[must_use]
    pub fn remaining_ticks(self, at: Tick) -> u64 {
        at.0.checked_sub(self.received_at.0)
            .map_or(0, |elapsed| HEART_GRACE_TICKS.saturating_sub(elapsed))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageOutcome {
    IgnoredGrace,
    AbsorbedShield,
    AbsorbedHeart { grace: DamageGrace },
    Eliminated,
}

pub(crate) fn apply_damage(
    inventory: &mut Inventory,
    effects: &TimedEffects,
    grace: &mut Option<DamageGrace>,
    at: Tick,
) -> DamageOutcome {
    if grace.is_some_and(|active| active.is_active(at)) {
        return DamageOutcome::IgnoredGrace;
    }
    *grace = None;
    if effects.damage_immune(at) {
        return DamageOutcome::AbsorbedShield;
    }
    if inventory.consume_heart() {
        let active = DamageGrace::received(at);
        *grace = Some(active);
        return DamageOutcome::AbsorbedHeart { grace: active };
    }
    DamageOutcome::Eliminated
}

#[cfg(test)]
mod tests {
    use super::{DamageGrace, DamageOutcome, HEART_GRACE_TICKS, apply_damage};
    use crate::game::Tick;
    use crate::game::effects::{EffectKind, TimedEffects};
    use crate::game::inventory::{Equipment, Inventory};

    #[test]
    fn shield_precedes_heart_and_heart_grace_has_an_exact_boundary() {
        let mut inventory = Inventory::default();
        inventory.replace_equipment(Equipment::Heart);
        let mut effects = TimedEffects::default();
        effects.apply(EffectKind::Shield, Tick(10)).unwrap();
        let mut grace = None;
        assert_eq!(
            apply_damage(&mut inventory, &effects, &mut grace, Tick(10)),
            DamageOutcome::AbsorbedShield
        );
        assert_eq!(inventory.equipment(), Some(Equipment::Heart));
        effects.clear();
        assert_eq!(
            apply_damage(&mut inventory, &effects, &mut grace, Tick(10)),
            DamageOutcome::AbsorbedHeart {
                grace: DamageGrace::received(Tick(10))
            }
        );
        assert_eq!(inventory.equipment(), None);
        assert_eq!(
            apply_damage(&mut inventory, &effects, &mut grace, Tick(69)),
            DamageOutcome::IgnoredGrace
        );
        assert_eq!(
            apply_damage(&mut inventory, &effects, &mut grace, Tick(70)),
            DamageOutcome::Eliminated
        );
    }

    #[test]
    fn grace_remains_active_for_repeated_damage_at_the_tick_horizon() {
        let mut inventory = Inventory::default();
        inventory.replace_equipment(Equipment::Heart);
        let effects = TimedEffects::default();
        let mut grace = None;

        let outcome = apply_damage(&mut inventory, &effects, &mut grace, Tick(u64::MAX));
        let active = DamageGrace::received(Tick(u64::MAX));
        assert_eq!(outcome, DamageOutcome::AbsorbedHeart { grace: active });
        assert!(active.is_active(Tick(u64::MAX)));
        assert_eq!(active.remaining_ticks(Tick(u64::MAX)), HEART_GRACE_TICKS);
        assert_eq!(
            apply_damage(&mut inventory, &effects, &mut grace, Tick(u64::MAX)),
            DamageOutcome::IgnoredGrace
        );
    }
}

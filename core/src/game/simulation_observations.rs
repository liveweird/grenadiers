use super::Actor;
use crate::game::damage::DamageGrace;
use crate::game::effects::EffectKind;
use crate::game::types::Tick;

impl Actor {
    pub(crate) fn effect_observations(&self) -> impl Iterator<Item = (EffectKind, Tick)> + '_ {
        self.effects.observations()
    }

    pub(crate) const fn damage_grace_observation(&self) -> Option<DamageGrace> {
        self.damage_grace
    }
}

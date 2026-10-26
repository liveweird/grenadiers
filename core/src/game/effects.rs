use std::collections::BTreeMap;

use super::types::{SpeedLevel, Tick};

pub const SHIELD_TICKS: u64 = 300;
pub const STOPWATCH_TICKS: u64 = 180;
pub const ORDINARY_EFFECT_TICKS: u64 = 600;
pub const MAX_SLOW_FOOT_STACKS: usize = 6;
pub const GOLDEN_FLAME_RANGE: u16 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffectKind {
    Shield,
    Stopwatch,
    InvisibilityCloak,
    GoldenFlame,
    ShortFuse,
    Diarrhea,
    SlowFoot,
    ShortFlame,
    LongFuse,
    ReverseControls,
    InvisibleBombs,
}

impl EffectKind {
    #[must_use]
    pub const fn duration_ticks(self) -> u64 {
        match self {
            Self::Shield => SHIELD_TICKS,
            Self::Stopwatch => STOPWATCH_TICKS,
            _ => ORDINARY_EFFECT_TICKS,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectApplication {
    Added { expires_at: Tick },
    Refreshed { expires_at: Tick },
    Stacked { expires_at: Tick, stacks: usize },
    Ineligible,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectError {
    ExpiryOverflow,
}

/// Exact fixed-point speed ratio. Consumers retain the numerator until division.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectiveSpeed {
    numerator: u16,
    denominator: u16,
}

impl EffectiveSpeed {
    #[must_use]
    pub const fn from_slow_foot_stacks(base: SpeedLevel, stacks: usize) -> Self {
        let bounded = if stacks > MAX_SLOW_FOOT_STACKS {
            MAX_SLOW_FOOT_STACKS
        } else {
            stacks
        };
        Self {
            numerator: base.units_per_second(),
            denominator: 1_u16 << bounded,
        }
    }

    #[must_use]
    pub const fn numerator(self) -> u16 {
        self.numerator
    }

    #[must_use]
    pub const fn denominator(self) -> u16 {
        self.denominator
    }

    #[must_use]
    pub const fn units_per_second(self) -> u16 {
        self.numerator / self.denominator
    }

    /// Advances the authoritative coordinate-unit remainder by one 60 Hz update.
    #[must_use]
    pub const fn advance(self, residual: u16) -> (i32, u16) {
        let total = residual as u32 + self.units_per_second() as u32;
        (
            (total / super::types::TICKS_PER_SECOND as u32) as i32,
            (total % super::types::TICKS_PER_SECOND as u32) as u16,
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TimedEffects {
    ordinary: BTreeMap<EffectKind, Tick>,
    slow_foot: Vec<Tick>,
}

impl TimedEffects {
    pub(crate) fn observations(&self) -> impl Iterator<Item = (EffectKind, Tick)> + '_ {
        self.ordinary
            .iter()
            .map(|(&kind, &expires_at)| (kind, expires_at))
            .chain(
                self.slow_foot
                    .iter()
                    .map(|&expires_at| (EffectKind::SlowFoot, expires_at)),
            )
    }

    pub fn apply(&mut self, kind: EffectKind, at: Tick) -> Result<EffectApplication, EffectError> {
        let expires_at = Tick(
            at.0.checked_add(kind.duration_ticks())
                .ok_or(EffectError::ExpiryOverflow)?,
        );
        self.expire(at);
        if kind == EffectKind::SlowFoot {
            if self.slow_foot.len() == MAX_SLOW_FOOT_STACKS {
                return Ok(EffectApplication::Ineligible);
            }
            self.slow_foot.push(expires_at);
            self.slow_foot.sort_unstable();
            return Ok(EffectApplication::Stacked {
                expires_at,
                stacks: self.slow_foot.len(),
            });
        }
        let refreshed = self.ordinary.insert(kind, expires_at).is_some();
        Ok(if refreshed {
            EffectApplication::Refreshed { expires_at }
        } else {
            EffectApplication::Added { expires_at }
        })
    }

    pub(crate) fn expire(&mut self, at: Tick) {
        self.ordinary.retain(|_, expiry| *expiry > at);
        self.slow_foot.retain(|expiry| *expiry > at);
    }

    pub(crate) fn clear(&mut self) {
        self.ordinary.clear();
        self.slow_foot.clear();
    }

    #[must_use]
    pub fn active(&self, kind: EffectKind, at: Tick) -> bool {
        if kind == EffectKind::SlowFoot {
            return self.slow_foot.iter().any(|expiry| *expiry > at);
        }
        self.ordinary.get(&kind).is_some_and(|expiry| *expiry > at)
    }

    #[must_use]
    pub fn slow_foot_stacks(&self, at: Tick) -> usize {
        self.slow_foot.iter().filter(|expiry| **expiry > at).count()
    }

    #[must_use]
    pub fn effective_speed(&self, base: SpeedLevel, at: Tick) -> EffectiveSpeed {
        EffectiveSpeed::from_slow_foot_stacks(base, self.slow_foot_stacks(at))
    }

    #[must_use]
    pub fn bomb_range(&self, base: u16, at: Tick) -> u16 {
        if self.active(EffectKind::ShortFlame, at) {
            1
        } else if self.active(EffectKind::GoldenFlame, at) {
            GOLDEN_FLAME_RANGE
        } else {
            base
        }
    }

    #[must_use]
    pub fn bomb_fuse_ticks(&self, base: u64, at: Tick) -> Option<u64> {
        match (
            self.active(EffectKind::ShortFuse, at),
            self.active(EffectKind::LongFuse, at),
        ) {
            (true, false) => Some(base.div_ceil(2).max(1)),
            (false, true) => base.checked_mul(2),
            _ => Some(base),
        }
    }

    #[must_use]
    pub fn damage_immune(&self, at: Tick) -> bool {
        self.active(EffectKind::Shield, at)
    }

    #[must_use]
    pub fn stopwatch_active(&self, at: Tick) -> bool {
        self.active(EffectKind::Stopwatch, at)
    }

    #[must_use]
    pub fn hidden_from_opponents(&self, at: Tick) -> bool {
        self.active(EffectKind::InvisibilityCloak, at)
    }

    #[must_use]
    pub fn forces_bomb_drops(&self, at: Tick) -> bool {
        self.active(EffectKind::Diarrhea, at)
    }

    #[must_use]
    pub fn reverses_controls(&self, at: Tick) -> bool {
        self.active(EffectKind::ReverseControls, at)
    }

    #[must_use]
    pub fn hides_bombs(&self, at: Tick) -> bool {
        self.active(EffectKind::InvisibleBombs, at)
    }
}

#[cfg(test)]
mod tests {
    use super::{EffectApplication, EffectError, EffectKind, MAX_SLOW_FOOT_STACKS, TimedEffects};
    use crate::game::{SpeedLevel, Tick};

    #[test]
    fn ordinary_effects_refresh_to_exact_expiry_and_expire_at_the_boundary() {
        let mut effects = TimedEffects::default();
        assert_eq!(
            effects.apply(EffectKind::Shield, Tick(10)),
            Ok(EffectApplication::Added {
                expires_at: Tick(310)
            })
        );
        assert_eq!(
            effects.apply(EffectKind::Shield, Tick(20)),
            Ok(EffectApplication::Refreshed {
                expires_at: Tick(320)
            })
        );
        assert!(effects.active(EffectKind::Shield, Tick(319)));
        effects.expire(Tick(320));
        assert!(!effects.active(EffectKind::Shield, Tick(320)));
        assert_eq!(
            effects.apply(EffectKind::Stopwatch, Tick(u64::MAX)),
            Err(EffectError::ExpiryOverflow)
        );
    }

    #[test]
    fn slow_stacks_expire_independently_and_seventh_is_ineligible_without_refresh() {
        let mut effects = TimedEffects::default();
        for offset in 0..MAX_SLOW_FOOT_STACKS {
            let at = Tick(offset as u64);
            assert_eq!(
                effects.apply(EffectKind::SlowFoot, at),
                Ok(EffectApplication::Stacked {
                    expires_at: Tick(600 + offset as u64),
                    stacks: offset + 1,
                })
            );
        }
        let at_capacity = effects.clone();
        assert_eq!(
            effects.apply(EffectKind::SlowFoot, Tick(10)),
            Ok(EffectApplication::Ineligible)
        );
        assert_eq!(effects, at_capacity);
        effects.expire(Tick(600));
        assert_eq!(effects.slow_foot_stacks(Tick(600)), 5);
        assert!(matches!(
            effects.apply(EffectKind::SlowFoot, Tick(600)),
            Ok(EffectApplication::Stacked { stacks: 6, .. })
        ));
    }

    #[test]
    fn derived_modifiers_compose_without_mutating_base_upgrades() {
        let mut effects = TimedEffects::default();
        let at = Tick(5);
        effects.apply(EffectKind::GoldenFlame, at).unwrap();
        effects.apply(EffectKind::ShortFlame, at).unwrap();
        effects.apply(EffectKind::ShortFuse, at).unwrap();
        effects.apply(EffectKind::LongFuse, at).unwrap();
        for _ in 0..6 {
            effects.apply(EffectKind::SlowFoot, at).unwrap();
        }
        assert_eq!(effects.bomb_range(6, at), 1);
        assert_eq!(effects.bomb_fuse_ticks(181, at), Some(181));
        let speed = effects.effective_speed(SpeedLevel::Base, at);
        assert_eq!((speed.numerator(), speed.denominator()), (3_072, 64));
        assert_eq!(SpeedLevel::Base.units_per_second(), 3_072);
    }

    #[test]
    fn range_and_fuse_modifiers_cover_individual_rounding_overflow_and_expiry() {
        let mut range = TimedEffects::default();
        range.apply(EffectKind::GoldenFlame, Tick(10)).unwrap();
        assert_eq!(range.bomb_range(6, Tick(10)), 8);
        range.apply(EffectKind::ShortFlame, Tick(10)).unwrap();
        assert_eq!(range.bomb_range(6, Tick(10)), 1);
        assert_eq!(range.bomb_range(6, Tick(610)), 6);

        let mut short = TimedEffects::default();
        short.apply(EffectKind::ShortFuse, Tick(0)).unwrap();
        assert_eq!(short.bomb_fuse_ticks(180, Tick(0)), Some(90));
        assert_eq!(short.bomb_fuse_ticks(181, Tick(0)), Some(91));

        let mut long = TimedEffects::default();
        long.apply(EffectKind::LongFuse, Tick(0)).unwrap();
        assert_eq!(long.bomb_fuse_ticks(180, Tick(0)), Some(360));
        assert_eq!(long.bomb_fuse_ticks(u64::MAX, Tick(0)), None);

        short.apply(EffectKind::LongFuse, Tick(0)).unwrap();
        assert_eq!(short.bomb_fuse_ticks(181, Tick(0)), Some(181));
        assert_eq!(short.bomb_fuse_ticks(181, Tick(600)), Some(181));
    }

    #[test]
    fn slow_rate_uses_the_upgraded_base_and_each_expiry_independently() {
        let mut effects = TimedEffects::default();
        effects.apply(EffectKind::SlowFoot, Tick(0)).unwrap();
        let one_stack = effects.effective_speed(SpeedLevel::Four, Tick(0));
        assert_eq!((one_stack.numerator(), one_stack.denominator()), (5_120, 2));
        effects.apply(EffectKind::SlowFoot, Tick(100)).unwrap();
        assert_eq!(
            effects
                .effective_speed(SpeedLevel::Four, Tick(599))
                .denominator(),
            4
        );
        assert_eq!(
            effects
                .effective_speed(SpeedLevel::Four, Tick(600))
                .denominator(),
            2
        );
        assert_eq!(
            effects
                .effective_speed(SpeedLevel::Four, Tick(700))
                .denominator(),
            1
        );
    }

    #[test]
    fn effective_speed_advances_the_authoritative_remainder_exactly() {
        let speed = super::EffectiveSpeed::from_slow_foot_stacks(SpeedLevel::Base, 0);
        assert_eq!(speed.advance(0), (51, 12));
        assert_eq!(speed.advance(59), (52, 11));
        let slow = super::EffectiveSpeed::from_slow_foot_stacks(SpeedLevel::Four, 2);
        assert_eq!(slow.units_per_second(), 1_280);
        assert_eq!(slow.advance(0), (21, 20));
    }

    #[test]
    fn failed_expiry_overflow_is_atomic() {
        let mut effects = TimedEffects::default();
        effects.apply(EffectKind::Shield, Tick(0)).unwrap();
        let before = effects.clone();
        assert_eq!(
            effects.apply(EffectKind::InvisibilityCloak, Tick(u64::MAX)),
            Err(EffectError::ExpiryOverflow)
        );
        assert_eq!(effects, before);
    }

    #[test]
    fn non_stat_capabilities_follow_exact_active_windows() {
        let mut effects = TimedEffects::default();
        for kind in [
            EffectKind::Stopwatch,
            EffectKind::InvisibilityCloak,
            EffectKind::Diarrhea,
            EffectKind::ReverseControls,
            EffectKind::InvisibleBombs,
        ] {
            effects.apply(kind, Tick(10)).unwrap();
        }
        assert!(effects.stopwatch_active(Tick(189)));
        assert!(!effects.stopwatch_active(Tick(190)));
        assert!(effects.hidden_from_opponents(Tick(609)));
        assert!(effects.forces_bomb_drops(Tick(609)));
        assert!(effects.reverses_controls(Tick(609)));
        assert!(effects.hides_bombs(Tick(609)));
    }
}

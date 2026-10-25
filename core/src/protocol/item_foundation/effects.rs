use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawEffectView", into = "RawEffectView")]
pub struct EffectView {
    kind: EffectKind,
    remaining_ticks: u16,
}

impl EffectView {
    pub const fn new(
        kind: EffectKind,
        remaining_ticks: u16,
    ) -> Result<Self, super::super::InvalidScalar> {
        let maximum = match kind {
            EffectKind::Shield => 300,
            EffectKind::Stopwatch => 180,
            _ => 600,
        };
        if remaining_ticks >= 1 && remaining_ticks <= maximum {
            Ok(Self {
                kind,
                remaining_ticks,
            })
        } else {
            Err(super::super::InvalidScalar)
        }
    }

    #[must_use]
    pub const fn kind(self) -> EffectKind {
        self.kind
    }
    #[must_use]
    pub const fn remaining_ticks(self) -> u16 {
        self.remaining_ticks
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawEffectView {
    kind: EffectKind,
    #[serde(deserialize_with = "remaining_ticks")]
    remaining_ticks: u16,
}

fn remaining_ticks<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<u16, D::Error> {
    super::super::scalar_input::deserialize_bounded(deserializer, 600, "invalid effect duration")
        .map(|value| value as u16)
}

impl TryFrom<RawEffectView> for EffectView {
    type Error = super::super::InvalidScalar;
    fn try_from(raw: RawEffectView) -> Result<Self, Self::Error> {
        Self::new(raw.kind, raw.remaining_ticks)
    }
}

impl From<EffectView> for RawEffectView {
    fn from(effect: EffectView) -> Self {
        Self {
            kind: effect.kind,
            remaining_ticks: effect.remaining_ticks,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<EffectView>", into = "Vec<EffectView>")]
pub struct EffectViews(Vec<EffectView>);

impl EffectViews {
    pub fn new(effects: Vec<EffectView>) -> Result<Self, super::super::InvalidScalar> {
        if effects.len() <= 16 {
            Ok(Self(effects))
        } else {
            Err(super::super::InvalidScalar)
        }
    }
    #[must_use]
    pub fn as_slice(&self) -> &[EffectView] {
        &self.0
    }
}

impl TryFrom<Vec<EffectView>> for EffectViews {
    type Error = super::super::InvalidScalar;
    fn try_from(effects: Vec<EffectView>) -> Result<Self, Self::Error> {
        Self::new(effects)
    }
}
impl From<EffectViews> for Vec<EffectView> {
    fn from(effects: EffectViews) -> Self {
        effects.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectValidationError {
    DuplicateEffect,
    SlowFootLimit,
}

/// Applies gameplay semantics beyond the structural JSON contract.
pub fn validate_effects(effects: &EffectViews) -> Result<(), EffectValidationError> {
    let mut ordinary = [false; 11];
    let mut slow_foot = 0;
    for effect in effects.as_slice() {
        if effect.kind == EffectKind::SlowFoot {
            slow_foot += 1;
            if slow_foot > 6 {
                return Err(EffectValidationError::SlowFootLimit);
            }
        } else {
            let index = effect.kind as usize;
            if ordinary[index] {
                return Err(EffectValidationError::DuplicateEffect);
            }
            ordinary[index] = true;
        }
    }
    Ok(())
}

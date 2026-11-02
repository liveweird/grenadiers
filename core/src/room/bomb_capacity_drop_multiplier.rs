pub const MIN_BOMB_CAPACITY_DROP_MULTIPLIER: u64 = 1;
pub const MAX_BOMB_CAPACITY_DROP_MULTIPLIER: u64 = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BombCapacityDropMultiplierError {
    InvalidValue(u64),
}

impl std::fmt::Display for BombCapacityDropMultiplierError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for BombCapacityDropMultiplierError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BombCapacityDropMultiplier(u8);

impl BombCapacityDropMultiplier {
    pub const LEGACY: Self = Self(MIN_BOMB_CAPACITY_DROP_MULTIPLIER as u8);
    pub const DEFAULT: Self = Self(5);

    pub const fn new(value: u64) -> Result<Self, BombCapacityDropMultiplierError> {
        if value < MIN_BOMB_CAPACITY_DROP_MULTIPLIER || value > MAX_BOMB_CAPACITY_DROP_MULTIPLIER {
            return Err(BombCapacityDropMultiplierError::InvalidValue(value));
        }
        Ok(Self(value as u8))
    }

    #[must_use]
    pub const fn value(self) -> u64 {
        self.0 as u64
    }
}

impl Default for BombCapacityDropMultiplier {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
#[path = "bomb_capacity_drop_multiplier_tests.rs"]
mod tests;

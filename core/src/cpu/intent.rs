use crate::game::Direction;

/// Desired movement for one admitted CPU decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MovementIntent {
    Direction(Direction),
    Stop,
}

/// At most one ordinary manual action accompanying movement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManualIntent {
    Bomb,
    Equipment,
}

/// A transport-neutral request which still passes through ordinary validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Intent {
    pub movement: MovementIntent,
    pub manual: Option<ManualIntent>,
}

impl Intent {
    pub const STOP: Self = Self {
        movement: MovementIntent::Stop,
        manual: None,
    };

    #[must_use]
    pub const fn moving(direction: Direction) -> Self {
        Self {
            movement: MovementIntent::Direction(direction),
            manual: None,
        }
    }

    #[must_use]
    pub const fn with_manual(mut self, manual: ManualIntent) -> Self {
        self.manual = Some(manual);
        self
    }
}

impl Default for Intent {
    fn default() -> Self {
        Self::STOP
    }
}

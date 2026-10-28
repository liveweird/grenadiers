use crate::cpu::MemoryKey;
use crate::cpu::forecast::TickWindow;
use crate::game::{Direction, EntityId, Position, Tick};

use super::super::MovementIntent;
use super::BraceletGuard;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteMode {
    CertifiedOnly,
    AllowStationaryAssumptions,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteAssurance {
    Certified,
    ConditionalStationary { bombs: Vec<EntityId> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PulseRequest {
    pub movement: MovementIntent,
    pub hold_ticks: u8,
    pub start_window: TickWindow,
    pub stopped_safe_until: Tick,
    pub mode: RouteMode,
    pub settled_stop_tick: Tick,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoseBounds {
    min: Position,
    max: Position,
    heading_mask: u8,
}

impl PoseBounds {
    pub(super) fn new(min: Position, max: Position, heading_mask: u8) -> Option<Self> {
        (min.x <= max.x && min.y <= max.y && heading_mask != 0).then_some(Self {
            min,
            max,
            heading_mask,
        })
    }

    #[must_use]
    pub const fn min(self) -> Position {
        self.min
    }
    #[must_use]
    pub const fn max(self) -> Position {
        self.max
    }
    #[must_use]
    pub const fn heading_mask(self) -> u8 {
        self.heading_mask
    }
    #[must_use]
    pub fn contains(self, position: Position, heading: Direction) -> bool {
        (self.min.x..=self.max.x).contains(&position.x)
            && (self.min.y..=self.max.y).contains(&position.y)
            && self.heading_mask & direction_bit(heading) != 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PulseLease {
    key: MemoryKey,
    basis_tick: Tick,
    start_window: TickWindow,
    movement: MovementIntent,
    hold_ticks: u8,
    safe_until: Tick,
    assurance: RouteAssurance,
    initial_position: Position,
    initial_heading: Direction,
    poses: Vec<PoseBounds>,
    bracelet_guard: Option<BraceletGuard>,
}

impl PulseLease {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        key: MemoryKey,
        basis_tick: Tick,
        request: PulseRequest,
        assurance: RouteAssurance,
        initial_position: Position,
        initial_heading: Direction,
        poses: Vec<PoseBounds>,
    ) -> Result<Self, RouteFailure> {
        if poses.is_empty()
            || poses.len() != usize::from(request.hold_ticks) + 1
            || poses.len() > 13
        {
            return Err(RouteFailure::Capacity);
        }
        Ok(Self {
            key,
            basis_tick,
            start_window: request.start_window,
            movement: request.movement,
            hold_ticks: request.hold_ticks,
            safe_until: request.stopped_safe_until,
            assurance,
            initial_position,
            initial_heading,
            poses,
            bracelet_guard: None,
        })
    }

    /// Restricts this lease to the public facts used by the Bracelet tactic.
    #[must_use]
    pub(crate) fn with_bracelet_guard(mut self, rival: EntityId, direction: Direction) -> Self {
        self.bracelet_guard = Some(BraceletGuard::new(rival, direction));
        self
    }

    #[must_use]
    pub(crate) fn bracelet_guard_allows(&self, observation: &crate::cpu::Observation) -> bool {
        self.bracelet_guard
            .is_none_or(|guard| guard.allows(observation))
    }

    #[must_use]
    pub const fn key(&self) -> MemoryKey {
        self.key
    }
    #[must_use]
    pub const fn basis_tick(&self) -> Tick {
        self.basis_tick
    }
    #[must_use]
    pub const fn start_window(&self) -> TickWindow {
        self.start_window
    }
    #[must_use]
    pub const fn movement(&self) -> MovementIntent {
        self.movement
    }
    #[must_use]
    pub const fn hold_ticks(&self) -> u8 {
        self.hold_ticks
    }
    #[must_use]
    pub const fn safe_until(&self) -> Tick {
        self.safe_until
    }
    #[must_use]
    pub const fn assurance(&self) -> &RouteAssurance {
        &self.assurance
    }
    #[must_use]
    pub fn matches_initial_pose(&self, position: Position, heading: Direction) -> bool {
        self.initial_position == position && self.initial_heading == heading
    }
    #[must_use]
    pub fn pose_after(&self, completed: u8) -> PoseBounds {
        self.poses[usize::from(completed.min(self.hold_ticks))]
    }
}

const fn direction_bit(direction: Direction) -> u8 {
    match direction {
        Direction::North => 1,
        Direction::East => 2,
        Direction::South => 4,
        Direction::West => 8,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteFailure {
    IdentityMismatch,
    InvalidRequest,
    Unsupported,
    NoRoute,
    WorkExhausted,
    Capacity,
}

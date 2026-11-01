use crate::cpu::{Intent, MemoryKey};
use crate::game::{CellId, PickupKind, Position, SeatId, Tick};

pub const MAX_HOLD_TICKS: u8 = 12;
pub const MAX_ROUTE_CELLS: usize = 4;
pub const MAX_RECENT_CELLS: usize = 8;
pub const STRATEGIC_COMMITMENT_TICKS: u64 = 120;
pub const STRATEGIC_PROGRESS_TICKS: u64 = 180;
pub const FAILED_TARGET_DEFER_TICKS: u64 = 120;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrategicObjectiveKind {
    Collect,
    Farm,
    Attack,
    Explore,
    Survive,
    Wait,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrategicTarget {
    Pickup { cell: CellId, kind: PickupKind },
    Block { cell: CellId },
    Opponent { seat: SeatId },
    Tile { cell: CellId },
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrategicPhase {
    Approach,
    Plant,
    Escape,
    Reengage,
    Evade,
    Waiting,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrategicSwitchReason {
    Initial,
    Completed,
    TargetInvalid,
    VisibilityLost,
    BetterOpportunity,
    Stalled,
    NoTargets,
    SafetyInterrupt,
    Resumed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StrategicIntent {
    pub objective: StrategicObjectiveKind,
    pub target: StrategicTarget,
    pub approach_cell: Option<CellId>,
    pub phase: StrategicPhase,
    pub last_switch_reason: StrategicSwitchReason,
    pub observation_tick: Tick,
    pub score: u8,
    pub committed_until: Tick,
    pub progress_deadline: Tick,
    pub last_distance: u16,
    pub last_cpu_position: Position,
    pub deferred_until: Option<Tick>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntentDebugSummary {
    pub cpu_seat: SeatId,
    pub objective: StrategicObjectiveKind,
    pub target: StrategicTarget,
    pub approach_cell: Option<CellId>,
    pub phase: StrategicPhase,
    pub last_switch_reason: StrategicSwitchReason,
    pub observation_tick: Tick,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GoalKind {
    Pickup,
    Farming,
    Combat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GoalMemory {
    pub kind: GoalKind,
    pub cell: CellId,
    pub priority: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyError {
    IdentityMismatch,
    InvalidObservation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UtilityMemory {
    key: MemoryKey,
    route: [Option<CellId>; MAX_ROUTE_CELLS],
    route_len: u8,
    route_index: u8,
    planted_origin: Option<CellId>,
    refuge: Option<CellId>,
    goal: Option<GoalMemory>,
    previous_cell: Option<CellId>,
    recent_cells: [Option<CellId>; MAX_RECENT_CELLS],
    recent_len: u8,
    recent_next: u8,
    deferred_goal: Option<GoalMemory>,
    strategic_intent: Option<StrategicIntent>,
    failed_target: Option<(StrategicTarget, Tick)>,
    failed_farm_approach: Option<(CellId, Tick)>,
}

impl UtilityMemory {
    #[must_use]
    pub const fn new(key: MemoryKey) -> Self {
        Self {
            key,
            route: [None; MAX_ROUTE_CELLS],
            route_len: 0,
            route_index: 0,
            planted_origin: None,
            refuge: None,
            goal: None,
            previous_cell: None,
            recent_cells: [None; MAX_RECENT_CELLS],
            recent_len: 0,
            recent_next: 0,
            deferred_goal: None,
            strategic_intent: None,
            failed_target: None,
            failed_farm_approach: None,
        }
    }

    #[must_use]
    pub const fn key(self) -> MemoryKey {
        self.key
    }

    #[must_use]
    pub const fn route(&self) -> &[Option<CellId>; MAX_ROUTE_CELLS] {
        &self.route
    }

    #[must_use]
    pub const fn planted_origin(self) -> Option<CellId> {
        self.planted_origin
    }

    #[must_use]
    pub const fn refuge(self) -> Option<CellId> {
        self.refuge
    }

    #[must_use]
    pub const fn goal(self) -> Option<GoalMemory> {
        self.goal
    }

    #[must_use]
    pub const fn previous_cell(self) -> Option<CellId> {
        self.previous_cell
    }

    #[must_use]
    pub const fn recent_cells(&self) -> &[Option<CellId>; MAX_RECENT_CELLS] {
        &self.recent_cells
    }

    #[must_use]
    pub const fn deferred_goal(self) -> Option<GoalMemory> {
        self.deferred_goal
    }

    #[must_use]
    pub const fn strategic_intent(self) -> Option<StrategicIntent> {
        self.strategic_intent
    }

    #[must_use]
    pub(crate) const fn failed_target(self) -> Option<(StrategicTarget, Tick)> {
        self.failed_target
    }

    #[must_use]
    pub(crate) const fn failed_farm_approach(self) -> Option<(CellId, Tick)> {
        self.failed_farm_approach
    }

    #[must_use]
    pub fn with_strategic_intent(mut self, intent: StrategicIntent) -> Self {
        self.strategic_intent = Some(intent);
        self
    }

    pub(crate) fn with_failed_target(mut self, failed: Option<(StrategicTarget, Tick)>) -> Self {
        self.failed_target = failed;
        self
    }

    pub(crate) fn defer_strategic_target(mut self, tick: Tick) -> Self {
        if let Some(intent) = self.strategic_intent.as_mut() {
            self.failed_target = Some((
                intent.target,
                Tick(tick.0.saturating_add(FAILED_TARGET_DEFER_TICKS)),
            ));
            intent.last_switch_reason = StrategicSwitchReason::Stalled;
            intent.observation_tick = tick;
        }
        self
    }

    pub(crate) fn defer_farm_approach(mut self, cell: CellId, tick: Tick) -> Self {
        self.failed_farm_approach =
            Some((cell, Tick(tick.0.saturating_add(FAILED_TARGET_DEFER_TICKS))));
        self
    }

    pub fn mark_strategic_phase(
        mut self,
        phase: StrategicPhase,
        reason: StrategicSwitchReason,
        tick: Tick,
    ) -> Self {
        if let Some(intent) = self.strategic_intent.as_mut() {
            intent.phase = phase;
            intent.last_switch_reason = reason;
            intent.observation_tick = tick;
            if matches!(phase, StrategicPhase::Escape | StrategicPhase::Reengage) {
                intent.progress_deadline = Tick(tick.0.saturating_add(STRATEGIC_PROGRESS_TICKS));
            }
        }
        self
    }

    pub fn suspend_strategic_deadline(mut self, tick: Tick) -> Self {
        if let Some(intent) = self.strategic_intent.as_mut() {
            intent.observation_tick = tick;
            intent.progress_deadline = Tick(tick.0.saturating_add(STRATEGIC_PROGRESS_TICKS));
        }
        self
    }

    /// Drops every executable route/refuge fact while retaining only bounded,
    /// non-authoritative strategic metadata.
    #[must_use]
    pub fn without_execution_authority(self) -> Self {
        Self {
            strategic_intent: self.strategic_intent,
            failed_target: self.failed_target,
            failed_farm_approach: self.failed_farm_approach,
            ..Self::new(self.key)
        }
    }

    pub(crate) fn use_survival_fallback(mut self, tick: Tick) -> Self {
        if let Some(intent) = self.strategic_intent.as_mut() {
            intent.objective = StrategicObjectiveKind::Survive;
            intent.target = StrategicTarget::None;
            intent.approach_cell = None;
            intent.phase = StrategicPhase::Evade;
            intent.last_switch_reason = StrategicSwitchReason::SafetyInterrupt;
            intent.observation_tick = tick;
        }
        self
    }

    pub(crate) fn with_progress(mut self, current: CellId, goal: GoalMemory) -> Self {
        self.observe_cell(current);
        self.goal = Some(goal);
        self
    }

    pub(crate) fn without_guidance(mut self, current: CellId) -> Self {
        self.observe_cell(current);
        if let Some(goal) = self.goal
            && goal.cell == current
            && goal.kind != GoalKind::Pickup
        {
            self.deferred_goal = Some(goal);
        }
        self.goal = None;
        self
    }

    pub(crate) fn carry_progress(mut self, previous: Self) -> Self {
        self.goal = previous.goal;
        self.previous_cell = previous.previous_cell;
        self.recent_cells = previous.recent_cells;
        self.recent_len = previous.recent_len;
        self.recent_next = previous.recent_next;
        self.deferred_goal = previous.deferred_goal;
        self.strategic_intent = previous.strategic_intent;
        self.failed_target = previous.failed_target;
        self.failed_farm_approach = previous.failed_farm_approach;
        self
    }

    fn observe_cell(&mut self, current: CellId) {
        if self.previous_cell == Some(current) {
            return;
        }
        let index = usize::from(self.recent_next);
        self.recent_cells[index] = Some(current);
        self.recent_len = self
            .recent_len
            .saturating_add(1)
            .min(MAX_RECENT_CELLS as u8);
        self.recent_next = (self.recent_next + 1) % MAX_RECENT_CELLS as u8;
        self.previous_cell = Some(current);
    }

    pub(crate) fn from_route(key: MemoryKey, cells: &[CellId], planted: bool) -> Self {
        let mut memory = Self::new(key);
        let len = cells.len().min(MAX_ROUTE_CELLS);
        let mut index = 0;
        while index < len {
            memory.route[index] = Some(cells[index]);
            index += 1;
        }
        memory.route_len = len as u8;
        memory.refuge = cells.get(len.saturating_sub(1)).copied();
        if planted {
            memory.planted_origin = cells.first().copied();
            memory.route_index = 1;
        }
        memory
    }

    pub(crate) fn next(mut self, current: CellId, position: Position) -> Option<(CellId, Self)> {
        while self.route_index < self.route_len {
            let index = usize::from(self.route_index);
            let cell = self.route[index]?;
            if cell != current || !near_center(position, cell) {
                return Some((cell, self));
            }
            self.route_index += 1;
        }
        None
    }
}

fn near_center(position: Position, cell: CellId) -> bool {
    let center = Position::center(cell);
    position.x.abs_diff(center.x) <= 64 && position.y.abs_diff(center.y) <= 64
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlPlan {
    pub intent: Intent,
    pub hold_ticks: u8,
    pub next_memory: UtilityMemory,
}

impl ControlPlan {
    #[must_use]
    pub const fn stop(memory: UtilityMemory) -> Self {
        Self {
            intent: Intent::STOP,
            hold_ticks: 1,
            next_memory: memory,
        }
    }
}

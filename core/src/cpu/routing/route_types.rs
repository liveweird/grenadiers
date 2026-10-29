use crate::cpu::MemoryKey;
use crate::cpu::forecast::{ForecastEvolution, TickWindow};
use crate::game::{Direction, Position, Tick};

use super::{RouteAssurance, RouteFailure, RouteMode};

pub const MAX_ROUTE_STEPS: u8 = 8;
pub const MAX_ROUTE_EXPANSIONS: u16 = 2_048;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RouteLimits {
    pub max_steps: u8,
    pub max_expansions: u16,
}

impl Default for RouteLimits {
    fn default() -> Self {
        Self {
            max_steps: MAX_ROUTE_STEPS,
            max_expansions: MAX_ROUTE_EXPANSIONS,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RouteRequest {
    pub start_window: TickWindow,
    pub settled_stop_tick: Tick,
    pub survive_until: Tick,
    pub mode: RouteMode,
    pub limits: RouteLimits,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteStep {
    Move { direction: Direction, updates: u8 },
    Wait { updates: u16 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StepTiming {
    pub(super) start: Tick,
    pub(super) stop: Tick,
    pub(super) ready: Tick,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteLane {
    first_update: Tick,
    timings: Vec<StepTiming>,
    poses: Vec<(
        Tick,
        Position,
        Direction,
        Option<super::storm::DeadlineBounds>,
    )>,
}

impl RouteLane {
    pub(super) fn new(
        first_update: Tick,
        timings: Vec<StepTiming>,
        poses: Vec<(
            Tick,
            Position,
            Direction,
            Option<super::storm::DeadlineBounds>,
        )>,
    ) -> Result<Self, RouteFailure> {
        if timings.len() > usize::from(MAX_ROUTE_STEPS)
            || poses.is_empty()
            || poses.windows(2).any(|pair| pair[0].0 >= pair[1].0)
        {
            return Err(RouteFailure::Capacity);
        }
        Ok(Self {
            first_update,
            timings,
            poses,
        })
    }

    #[must_use]
    pub fn step_start(&self, index: usize) -> Option<Tick> {
        self.timings.get(index).map(|timing| timing.start)
    }

    #[must_use]
    pub fn stop_update(&self, index: usize) -> Option<Tick> {
        self.timings.get(index).map(|timing| timing.stop)
    }

    #[must_use]
    pub fn ready_update(&self, index: usize) -> Option<Tick> {
        self.timings.get(index).map(|timing| timing.ready)
    }

    #[must_use]
    pub fn pose_at(&self, snapshot_tick: Tick) -> Option<(Position, Direction)> {
        self.poses
            .binary_search_by_key(&snapshot_tick, |pose| pose.0)
            .ok()
            .map(|index| (self.poses[index].1, self.poses[index].2))
    }

    pub(crate) fn storm_deadlines_at(
        &self,
        snapshot_tick: Tick,
    ) -> Option<super::storm::DeadlineBounds> {
        self.poses
            .binary_search_by_key(&snapshot_tick, |pose| pose.0)
            .ok()
            .and_then(|index| self.poses[index].3)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteCertificate {
    key: MemoryKey,
    basis_tick: Tick,
    start_window: TickWindow,
    assurance: RouteAssurance,
    survive_until: Tick,
    steps: Vec<RouteStep>,
    lanes: Vec<RouteLane>,
    evolution: ForecastEvolution,
}

impl RouteCertificate {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        key: MemoryKey,
        basis_tick: Tick,
        start_window: TickWindow,
        assurance: RouteAssurance,
        survive_until: Tick,
        steps: Vec<RouteStep>,
        lanes: Vec<RouteLane>,
        evolution: ForecastEvolution,
    ) -> Result<Self, RouteFailure> {
        let expected_lanes = start_window
            .end
            .0
            .checked_sub(start_window.start.0)
            .and_then(|width| usize::try_from(width).ok())
            .ok_or(RouteFailure::Capacity)?;
        let final_tick = Tick(
            survive_until
                .0
                .checked_sub(1)
                .ok_or(RouteFailure::Capacity)?,
        );
        if steps.is_empty()
            || steps.len() > usize::from(MAX_ROUTE_STEPS)
            || lanes.len() != expected_lanes
            || lanes.len() > 7
            || lanes
                .iter()
                .zip(start_window.start.0..start_window.end.0)
                .any(|(lane, tick)| lane.first_update != Tick(tick))
            || lanes.iter().any(|lane| {
                lane.timings.len() != steps.len()
                    || lane.pose_at(basis_tick).is_none()
                    || lane.pose_at(final_tick).is_none()
            })
        {
            return Err(RouteFailure::Capacity);
        }
        Ok(Self {
            key,
            basis_tick,
            start_window,
            assurance,
            survive_until,
            steps,
            lanes,
            evolution,
        })
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
    pub const fn assurance(&self) -> &RouteAssurance {
        &self.assurance
    }

    #[must_use]
    pub const fn survive_until(&self) -> Tick {
        self.survive_until
    }

    #[must_use]
    pub fn steps(&self) -> &[RouteStep] {
        &self.steps
    }

    #[must_use]
    pub fn lane(&self, first_update: Tick) -> Option<&RouteLane> {
        let index = first_update.0.checked_sub(self.start_window.start.0)?;
        self.lanes.get(usize::try_from(index).ok()?).filter(|lane| {
            lane.first_update == first_update && self.start_window.contains(first_update)
        })
    }

    #[must_use]
    pub const fn evolution(&self) -> &ForecastEvolution {
        &self.evolution
    }
}

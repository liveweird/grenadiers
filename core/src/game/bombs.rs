use super::input::Direction;
use super::types::{CellId, EntityId, Tick};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BombMotion {
    pub(crate) direction: Direction,
    pub(crate) countdown: u8,
}

impl BombMotion {
    #[must_use]
    pub const fn direction(self) -> Direction {
        self.direction
    }

    #[must_use]
    pub const fn countdown(self) -> u8 {
        self.countdown
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BombTrigger {
    Timed { detonates_at: Tick },
    Remote,
    Contact,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlastKind {
    #[default]
    Standard,
    Piercing,
    Power,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BombError {
    DeadlineBeforePlacement,
    DeadlineOverflow,
    PauseOverflow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimedBombHistory {
    pub(crate) captured_fuse_ticks: u64,
    pub(crate) paused_ticks: u64,
}

/// An owned bomb with placement-time properties captured for later detonation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bomb {
    pub(crate) id: EntityId,
    pub(crate) cell: CellId,
    pub(crate) owner: EntityId,
    pub(crate) range: u16,
    pub(crate) placed_at: Tick,
    pub(crate) trigger: BombTrigger,
    blast: BlastKind,
    timed_history: Option<TimedBombHistory>,
    motion: Option<BombMotion>,
    has_been_kicked: bool,
    has_been_punched: bool,
}

impl Bomb {
    pub(crate) const fn new(
        id: EntityId,
        cell: CellId,
        owner: EntityId,
        range: u16,
        placed_at: Tick,
        trigger: BombTrigger,
    ) -> Result<Self, BombError> {
        Self::new_with_blast(
            id,
            cell,
            owner,
            range,
            placed_at,
            trigger,
            BlastKind::Standard,
        )
    }

    pub(crate) const fn new_with_blast(
        id: EntityId,
        cell: CellId,
        owner: EntityId,
        range: u16,
        placed_at: Tick,
        trigger: BombTrigger,
        blast: BlastKind,
    ) -> Result<Self, BombError> {
        let timed_history = match trigger {
            BombTrigger::Timed { detonates_at } => {
                let Some(captured_fuse_ticks) = detonates_at.0.checked_sub(placed_at.0) else {
                    return Err(BombError::DeadlineBeforePlacement);
                };
                Some(TimedBombHistory {
                    captured_fuse_ticks,
                    paused_ticks: 0,
                })
            }
            BombTrigger::Remote | BombTrigger::Contact => None,
        };
        Ok(Self {
            id,
            cell,
            owner,
            range,
            placed_at,
            trigger,
            blast,
            timed_history,
            motion: None,
            has_been_kicked: false,
            has_been_punched: false,
        })
    }

    #[must_use]
    pub const fn id(self) -> EntityId {
        self.id
    }

    #[must_use]
    pub const fn cell(self) -> CellId {
        self.cell
    }

    #[must_use]
    pub const fn owner(self) -> EntityId {
        self.owner
    }

    #[must_use]
    pub const fn range(self) -> u16 {
        self.range
    }

    #[must_use]
    pub const fn placed_at(self) -> Tick {
        self.placed_at
    }

    #[must_use]
    pub const fn trigger(self) -> BombTrigger {
        self.trigger
    }

    #[must_use]
    pub const fn blast(self) -> BlastKind {
        self.blast
    }

    #[must_use]
    pub const fn detonates_at(self) -> Option<Tick> {
        match self.trigger {
            BombTrigger::Timed { detonates_at } => Some(detonates_at),
            BombTrigger::Remote | BombTrigger::Contact => None,
        }
    }

    #[must_use]
    pub const fn timed_history(self) -> Option<TimedBombHistory> {
        self.timed_history
    }

    #[must_use]
    pub const fn motion(self) -> Option<BombMotion> {
        self.motion
    }

    #[must_use]
    pub const fn has_been_kicked(self) -> bool {
        self.has_been_kicked
    }

    #[must_use]
    pub const fn has_been_punched(self) -> bool {
        self.has_been_punched
    }

    pub(crate) fn arm_kick(&mut self, direction: Direction) {
        self.motion = Some(BombMotion {
            direction,
            countdown: 0,
        });
        self.has_been_kicked = true;
    }

    pub(crate) fn set_motion(&mut self, motion: Option<BombMotion>) {
        self.motion = motion;
    }

    // Used by the paired trusted-capture binding once the v10 protocol slice is integrated.
    #[allow(dead_code)]
    pub(crate) fn restore_kick_history(
        &mut self,
        motion: Option<BombMotion>,
        has_been_kicked: bool,
    ) {
        self.motion = motion;
        self.has_been_kicked = has_been_kicked;
    }

    pub(crate) fn set_cell(&mut self, cell: CellId) {
        self.cell = cell;
    }

    pub(crate) fn apply_punch(&mut self, cell: CellId) {
        self.cell = cell;
        self.motion = None;
        self.has_been_punched = true;
    }

    // Used by the paired trusted-capture binding once the v10 protocol slice is integrated.
    #[allow(dead_code)]
    pub(crate) fn restore_punch_history(&mut self, has_been_punched: bool) {
        self.has_been_punched = has_been_punched;
    }

    pub(crate) fn pause_one_tick(&mut self) -> Result<(), BombError> {
        let BombTrigger::Timed { detonates_at } = self.trigger else {
            return Ok(());
        };
        let Some(history) = self.timed_history.as_mut() else {
            return Err(BombError::PauseOverflow);
        };
        let paused_ticks = history
            .paused_ticks
            .checked_add(1)
            .ok_or(BombError::PauseOverflow)?;
        let deadline = detonates_at
            .0
            .checked_add(1)
            .ok_or(BombError::DeadlineOverflow)?;
        history.paused_ticks = paused_ticks;
        self.trigger = BombTrigger::Timed {
            detonates_at: Tick(deadline),
        };
        Ok(())
    }
}

#[cfg(test)]
#[path = "bombs_tests.rs"]
mod tests;

use std::{fmt, mem::size_of};

use grenadiers_core::{
    game::{Direction, GenerationRecord, RoundAssignment, SeatId, SpeedLevel, Tick},
    room::{DepartureReason, InterruptReason, RoomSettings},
};

pub(super) const FORMAT_VERSION: u16 = 1;
pub(in crate::game::rooms) const M5_LIFECYCLE_PROFILE: u16 = 1;
pub(super) const M6_LIFECYCLE_PROFILE: u16 = 2;
pub(super) const CLASSIC_V2_RULES: u16 = 2;
pub(super) const CLASSIC_V3_RULES: u16 = 3;
pub(super) const CONFIGURABLE_V4_RULES: u16 = 4;
pub(super) const GENERATED_V1: u16 = 1;
pub(super) const MAX_RECORDS: usize = 60_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) struct Header {
    pub(super) format_version: u16,
    pub(super) lifecycle_profile: u16,
    pub(super) rules_version: u16,
    pub(super) generator_version: u16,
    pub(super) initial_seats: [bool; 2],
    pub(super) initial_tick: Tick,
    pub(super) initial_settings: RoomSettings,
    pub(super) initial_settings_revision: u32,
    pub(super) initial_settings_editor: Option<SeatId>,
}

impl Header {
    pub(in crate::game::rooms) fn m5(initial_seats: [bool; 2]) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            lifecycle_profile: M5_LIFECYCLE_PROFILE,
            rules_version: CLASSIC_V2_RULES,
            generator_version: GENERATED_V1,
            initial_seats,
            initial_tick: Tick(0),
            initial_settings: RoomSettings::new(SpeedLevel::Base, 128, 3, 180)
                .expect("valid classic v2 settings"),
            initial_settings_revision: 1,
            initial_settings_editor: None,
        }
    }

    pub(in crate::game::rooms) fn m6(initial_seats: [bool; 2]) -> Self {
        Self {
            lifecycle_profile: M6_LIFECYCLE_PROFILE,
            ..Self::m5(initial_seats)
        }
    }

    pub(in crate::game::rooms) fn m6_v3(initial_seats: [bool; 2]) -> Self {
        Self {
            rules_version: CLASSIC_V3_RULES,
            initial_settings: RoomSettings::DEFAULT,
            ..Self::m6(initial_seats)
        }
    }

    pub(in crate::game::rooms) fn configurable_v4() -> Self {
        Self {
            rules_version: CONFIGURABLE_V4_RULES,
            initial_settings: RoomSettings::DEFAULT,
            initial_settings_editor: Some(SeatId::One),
            ..Self::m6([false; 2])
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) enum ReplayAction {
    Ready(bool),
    ReadyChecked {
        ready: bool,
        settings_revision: u32,
    },
    UpdateSettings {
        settings_revision: u32,
        settings: RoomSettings,
    },
    Movement(Option<Direction>),
    Bomb(u32),
    Rematch(bool),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::game::rooms) enum Operation {
    Occupy(SeatId),
    Action {
        seat: SeatId,
        epoch: u64,
        action: ReplayAction,
    },
    InputFence(SeatId),
    Tick,
    #[cfg(test)]
    Depart([Option<DepartureReason>; 2]),
    GraceDepart([Option<DepartureReason>; 2]),
    Resume {
        seat: SeatId,
        kind: crate::game::rooms::match_runtime::ResumeKind,
    },
    Expiry,
    ReportFlush,
    Interrupt(InterruptReason),
    InterruptAndDepart {
        reason: InterruptReason,
        departures: [Option<DepartureReason>; 2],
    },
    RoundStarted {
        epoch: u64,
        assignment: RoundAssignment,
        generation: GenerationRecord,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct Record {
    pub(super) tick: Tick,
    pub(super) order: u32,
    pub(super) operation: Operation,
}

const _: () = assert!(size_of::<Record>() <= 64);
const _: () = assert!(MAX_RECORDS * size_of::<Record>() < 4 * 1024 * 1024);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReplayStatus {
    Complete,
    Truncated {
        last_complete_index: u32,
    },
    Incomplete {
        last_complete_index: Option<u32>,
        reason: IncompleteReason,
    },
    #[cfg(test)]
    UnsupportedProfile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum IncompleteReason {
    RoundConstruction,
}

pub(in crate::game::rooms) struct Journal {
    pub(super) header: Header,
    pub(super) records: Vec<Record>,
    pub(super) status: ReplayStatus,
}

impl Journal {
    pub(in crate::game::rooms) fn new(header: Header) -> Self {
        Self {
            header,
            records: Vec::with_capacity(MAX_RECORDS),
            status: ReplayStatus::Complete,
        }
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn unsupported_profile() -> Self {
        Self {
            header: Header::m5([false; 2]),
            records: Vec::new(),
            status: ReplayStatus::UnsupportedProfile,
        }
    }

    pub(in crate::game::rooms) fn record(&mut self, tick: Tick, operation: Operation) -> bool {
        if self.status != ReplayStatus::Complete {
            return false;
        }
        if self.records.len() == MAX_RECORDS {
            self.status = ReplayStatus::Truncated {
                last_complete_index: u32::try_from(self.records.len().saturating_sub(1))
                    .unwrap_or(u32::MAX),
            };
            return false;
        }
        let order = self.records.last().map_or(0, |previous| {
            if previous.tick == tick {
                previous.order + 1
            } else {
                0
            }
        });
        self.records.push(Record {
            tick,
            order,
            operation,
        });
        true
    }

    pub(in crate::game::rooms) fn mark_round_construction_incomplete(&mut self) {
        if self.status == ReplayStatus::Complete {
            self.status = ReplayStatus::Incomplete {
                last_complete_index: self
                    .records
                    .len()
                    .checked_sub(1)
                    .and_then(|index| u32::try_from(index).ok()),
                reason: IncompleteReason::RoundConstruction,
            };
        }
    }

    #[cfg(test)]
    pub(super) const fn status(&self) -> ReplayStatus {
        self.status
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.records.len()
    }

    #[cfg(test)]
    pub(super) fn allocated_bytes(&self) -> usize {
        size_of::<Self>() + self.records.capacity() * size_of::<Record>()
    }
}

impl fmt::Debug for Journal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Journal")
            .field("header", &self.header)
            .field("record_count", &self.records.len())
            .field("status", &self.status)
            .finish()
    }
}

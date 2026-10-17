use std::{fmt, mem::size_of};

use grenadiers_core::{
    game::{Direction, GenerationRecord, RoundAssignment, SeatId, Tick},
    room::{DepartureReason, InterruptReason},
};

pub(super) const FORMAT_VERSION: u16 = 1;
pub(super) const M5_LIFECYCLE_PROFILE: u16 = 1;
pub(super) const CLASSIC_V2_RULES: u16 = 2;
pub(super) const GENERATED_V1: u16 = 1;
pub(super) const MAX_RECORDS: usize = 60_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Header {
    pub(super) format_version: u16,
    pub(super) lifecycle_profile: u16,
    pub(super) rules_version: u16,
    pub(super) generator_version: u16,
    pub(super) initial_seats: [bool; 2],
    pub(super) initial_tick: Tick,
}

impl Header {
    pub(super) const fn m5(initial_seats: [bool; 2]) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            lifecycle_profile: M5_LIFECYCLE_PROFILE,
            rules_version: CLASSIC_V2_RULES,
            generator_version: GENERATED_V1,
            initial_seats,
            initial_tick: Tick(0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReplayAction {
    Ready(bool),
    Movement(Option<Direction>),
    Bomb(u32),
    Rematch(bool),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Operation {
    Occupy(SeatId),
    Action {
        seat: SeatId,
        epoch: u64,
        action: ReplayAction,
    },
    InputFence(SeatId),
    Tick,
    Depart([Option<DepartureReason>; 2]),
    Interrupt(InterruptReason),
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
    Truncated { last_complete_index: u32 },
}

pub(super) struct Journal {
    pub(super) header: Header,
    pub(super) records: Vec<Record>,
    pub(super) status: ReplayStatus,
}

impl Journal {
    pub(super) fn new(header: Header) -> Self {
        Self {
            header,
            records: Vec::with_capacity(MAX_RECORDS),
            status: ReplayStatus::Complete,
        }
    }

    pub(super) fn record(&mut self, tick: Tick, operation: Operation) -> bool {
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

    pub(super) const fn status(&self) -> ReplayStatus {
        self.status
    }

    pub(super) fn len(&self) -> usize {
        self.records.len()
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

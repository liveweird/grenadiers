use std::{fmt, mem::size_of};

use grenadiers_core::{
    game::{Direction, GenerationRecord, RoundAssignment, SeatId, Tick},
    room::{CpuDifficulty, DepartureReason, InterruptReason, RoomConfiguration, RoomSettings},
};

use crate::game::rooms::roster_seed::SimulationRoot;

pub(super) const LIFECYCLE_PROFILE: u16 = 3;
const _: () = assert!(LIFECYCLE_PROFILE == grenadiers_core::room::ROSTER_LIFECYCLE_PROFILE);
pub(super) const RULES_VERSION: u16 = 6;
pub(super) const CPU_POLICY_VERSION: u16 = 17;
pub(super) const SIMULATION_RANDOMNESS_VERSION: u16 = 1;
pub(super) const SIMULATION_SEED_DERIVATION_VERSION: u16 = 1;
pub(super) const MAX_RECORDS: usize = 60_000;

const _: () =
    assert!(SIMULATION_RANDOMNESS_VERSION == grenadiers_core::game::SIMULATION_RANDOMNESS_VERSION);
const _: () = assert!(
    SIMULATION_SEED_DERIVATION_VERSION == grenadiers_core::game::SIMULATION_SEED_DERIVATION_VERSION
);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::game::rooms) struct PrivateCpuSeed([u8; 32]);

impl PrivateCpuSeed {
    pub(in crate::game::rooms) const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl fmt::Debug for PrivateCpuSeed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PrivateCpuSeed(<redacted>)")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) struct CpuProvenance {
    pub(in crate::game::rooms) seat: SeatId,
    pub(in crate::game::rooms) difficulty: CpuDifficulty,
    pub(in crate::game::rooms) policy_version: u16,
    pub(in crate::game::rooms) private_seed: PrivateCpuSeed,
}

impl CpuProvenance {
    pub(in crate::game::rooms) const fn new(
        seat: SeatId,
        difficulty: CpuDifficulty,
        private_seed: [u8; 32],
    ) -> Self {
        Self {
            seat,
            difficulty,
            policy_version: CPU_POLICY_VERSION,
            private_seed: PrivateCpuSeed::new(private_seed),
        }
    }
}

#[derive(Clone, Debug)]
pub(in crate::game::rooms) struct ReplayHeader {
    pub(super) format_version: u16,
    pub(super) lifecycle_profile: u16,
    pub(super) rules_version: u16,
    pub(super) generator_version: u16,
    pub(super) simulation_randomness_version: u16,
    pub(super) simulation_seed_derivation_version: u16,
    pub(super) configuration: RoomConfiguration,
    pub(super) initial_settings: RoomSettings,
    pub(super) initial_settings_revision: u32,
    pub(super) initially_occupied_humans: [bool; 4],
    pub(super) simulation_root: SimulationRoot,
    pub(super) cpus: Box<[CpuProvenance]>,
}

pub(in crate::game::rooms) type ReplayV4Header = ReplayHeader;
pub(in crate::game::rooms) type ReplayV5Header = ReplayHeader;

impl ReplayHeader {
    pub(in crate::game::rooms) fn new(
        configuration: RoomConfiguration,
        settings: RoomSettings,
        initially_occupied_humans: [bool; 4],
        cpus: Vec<CpuProvenance>,
    ) -> Result<Self, ()> {
        Self::new_for_profile(
            super::profile::ReplayProfile::V4,
            configuration,
            settings,
            initially_occupied_humans,
            cpus,
        )
    }

    pub(in crate::game::rooms) fn new_v5(
        configuration: RoomConfiguration,
        settings: RoomSettings,
        initially_occupied_humans: [bool; 4],
        cpus: Vec<CpuProvenance>,
    ) -> Result<Self, ()> {
        Self::new_for_profile(
            super::profile::ReplayProfile::V5,
            configuration,
            settings,
            initially_occupied_humans,
            cpus,
        )
    }

    fn new_for_profile(
        profile: super::profile::ReplayProfile,
        configuration: RoomConfiguration,
        settings: RoomSettings,
        initially_occupied_humans: [bool; 4],
        cpus: Vec<CpuProvenance>,
    ) -> Result<Self, ()> {
        super::validation::validate_configuration(
            profile,
            configuration,
            initially_occupied_humans,
            &cpus,
        )?;
        let header = Self {
            format_version: profile.format(),
            lifecycle_profile: LIFECYCLE_PROFILE,
            rules_version: RULES_VERSION,
            generator_version: profile.generator(),
            simulation_randomness_version: SIMULATION_RANDOMNESS_VERSION,
            simulation_seed_derivation_version: SIMULATION_SEED_DERIVATION_VERSION,
            configuration,
            initial_settings: settings,
            initial_settings_revision: 1,
            initially_occupied_humans,
            // This request is deliberately separate from arena and controller roots.
            simulation_root: SimulationRoot::generate()?,
            cpus: cpus.into_boxed_slice(),
        };
        super::validation::validate_header_for_profile(profile, &header)?;
        Ok(header)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) enum ReplayAction {
    Ready {
        ready: bool,
        settings_revision: u32,
    },
    UpdateSettings {
        settings_revision: u32,
        settings: RoomSettings,
    },
    Movement(Option<Direction>),
    Bomb(u32),
    Equipment(u32),
    Rematch(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) enum ActionSource {
    Human,
    Cpu,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) enum Operation {
    Admit {
        seat: SeatId,
        resume: bool,
    },
    Action {
        seat: SeatId,
        epoch: u64,
        source: ActionSource,
        action: ReplayAction,
    },
    InputFence(SeatId),
    Settle([Option<DepartureReason>; 4]),
    Interrupt(InterruptReason),
    InterruptAndDepart {
        reason: InterruptReason,
        departures: [Option<DepartureReason>; 4],
    },
    PublishFlush,
    RoundStarted {
        epoch: u64,
        assignment: RoundAssignment,
        generation: GenerationRecord,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Record {
    pub(super) tick: Tick,
    pub(super) order: u32,
    pub(super) operation: Operation,
}

const _: () = assert!(size_of::<Record>() <= 64);
const _: () = assert!(MAX_RECORDS * size_of::<Record>() < 4 * 1024 * 1024);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) enum ReplayStatus {
    Complete,
    Truncated,
    IncompleteRoundConstruction,
}

#[derive(Clone)]
pub(in crate::game::rooms) struct Journal {
    pub(super) header: ReplayV4Header,
    pub(super) records: Vec<Record>,
    pub(super) status: ReplayStatus,
}

impl Journal {
    pub(in crate::game::rooms) fn new(header: ReplayV4Header) -> Self {
        Self {
            header,
            records: Vec::with_capacity(MAX_RECORDS),
            status: ReplayStatus::Complete,
        }
    }

    pub(super) fn record(&mut self, tick: Tick, operation: Operation) {
        if self.status != ReplayStatus::Complete {
            return;
        }
        if self.records.len() == MAX_RECORDS {
            self.status = ReplayStatus::Truncated;
            return;
        }
        let order = self
            .records
            .last()
            .filter(|record| record.tick == tick)
            .map_or(0, |record| record.order + 1);
        self.records.push(Record {
            tick,
            order,
            operation,
        });
    }

    pub(in crate::game::rooms) fn status(&self) -> ReplayStatus {
        self.status
    }
    pub(in crate::game::rooms) fn allocated_bytes(&self) -> usize {
        size_of::<Self>()
            + self.records.capacity() * size_of::<Record>()
            + self.header.cpus.len() * size_of::<CpuProvenance>()
    }
}

impl fmt::Debug for Journal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Replay4Journal")
            .field("header", &self.header)
            .field("record_count", &self.records.len())
            .field("status", &self.status)
            .finish()
    }
}

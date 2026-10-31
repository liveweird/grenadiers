use std::{fmt, mem::size_of};

use grenadiers_core::{
    game::{Direction, GenerationRecord, RoundAssignment, SeatId, StormCloudConfig, Tick},
    room::{
        CpuDifficulty, DepartureReason, InterruptReason, LairRoomConfiguration, RespawnLairTuning,
        RoomSettings,
    },
};

use crate::game::rooms::roster_seed::SimulationRoot;

pub(super) const FORMAT_VERSION: u16 = 8;
pub(super) const LIFECYCLE_PROFILE: u16 = 3;
pub(super) const RULES_VERSION: u16 = 9;
pub(super) const CPU_POLICY_VERSION: u16 = 21;
pub(super) const SIMULATION_RANDOMNESS_VERSION: u16 = 1;
pub(super) const SIMULATION_SEED_DERIVATION_VERSION: u16 = 1;
pub(super) const ENVIRONMENT_RANDOMNESS_VERSION: u16 = 1;
pub(super) const LAIR_LAYOUT_VERSION: u16 = 1;
pub(super) const MAX_RECORDS: usize = 60_000;

#[derive(Clone, Copy, PartialEq, Eq)]
struct PrivateCpuSeed([u8; 32]);

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
    private_seed: PrivateCpuSeed,
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
            private_seed: PrivateCpuSeed(private_seed),
        }
    }
}

#[derive(Clone, Debug)]
pub(in crate::game::rooms) struct ReplayHeader {
    pub(super) format_version: u16,
    pub(super) lifecycle_profile: u16,
    pub(super) rules_version: u16,
    pub(super) cpu_policy_version: u16,
    pub(super) generator_version: u16,
    pub(super) simulation_randomness_version: u16,
    pub(super) simulation_seed_derivation_version: u16,
    pub(super) environment_randomness_version: u16,
    pub(super) lair_layout_version: u16,
    pub(super) configuration: LairRoomConfiguration,
    pub(super) initial_settings: RoomSettings,
    pub(super) initial_settings_revision: u32,
    pub(super) initially_occupied_humans: [bool; 4],
    pub(super) simulation_root: SimulationRoot,
    pub(super) cpus: Box<[CpuProvenance]>,
}

impl ReplayHeader {
    pub(in crate::game::rooms) fn new(
        configuration: LairRoomConfiguration,
        settings: RoomSettings,
        initially_occupied_humans: [bool; 4],
        cpus: Vec<CpuProvenance>,
    ) -> Result<Self, ()> {
        let generator_version = if configuration.base().board()
            == grenadiers_core::room::BoardSize::FifteenByThirteen
        {
            grenadiers_core::game::MULTIPLAYER_GENERATOR_VERSION
        } else {
            grenadiers_core::game::VARIABLE_GENERATOR_VERSION
        };
        let header = Self {
            format_version: FORMAT_VERSION,
            lifecycle_profile: LIFECYCLE_PROFILE,
            rules_version: RULES_VERSION,
            cpu_policy_version: CPU_POLICY_VERSION,
            generator_version,
            simulation_randomness_version: SIMULATION_RANDOMNESS_VERSION,
            simulation_seed_derivation_version: SIMULATION_SEED_DERIVATION_VERSION,
            environment_randomness_version: ENVIRONMENT_RANDOMNESS_VERSION,
            lair_layout_version: LAIR_LAYOUT_VERSION,
            configuration,
            initial_settings: settings,
            initial_settings_revision: 1,
            initially_occupied_humans,
            simulation_root: SimulationRoot::generate()?,
            cpus: cpus.into_boxed_slice(),
        };
        super::validation::validate_header(&header)?;
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
        cloud: StormCloudConfig,
        tuning: RespawnLairTuning,
    },
    Movement(Option<Direction>),
    Bomb(u32),
    Equipment(u32),
    Rematch(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ActionSource {
    Human,
    Cpu,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Operation {
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

const _: () = assert!(size_of::<Record>() <= 80);
const _: () = assert!(MAX_RECORDS * size_of::<Record>() < 5 * 1024 * 1024);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) enum ReplayStatus {
    Complete,
    Truncated,
    IncompleteRoundConstruction,
}

#[derive(Clone)]
pub(in crate::game::rooms) struct Journal {
    pub(super) header: ReplayHeader,
    pub(super) records: Vec<Record>,
    pub(super) status: ReplayStatus,
}

impl Journal {
    pub(super) fn new(header: ReplayHeader) -> Self {
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

    pub(in crate::game::rooms) const fn status(&self) -> ReplayStatus {
        self.status
    }
}

impl fmt::Debug for Journal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Replay8Journal")
            .field("header", &self.header)
            .field("record_count", &self.records.len())
            .field("status", &self.status)
            .finish()
    }
}

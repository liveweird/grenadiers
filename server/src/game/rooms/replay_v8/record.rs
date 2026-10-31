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
pub(super) const CPU_POLICY_VERSION: u16 = 25;
pub(super) const INTENT_CPU_POLICY_VERSION: u16 = 24;
pub(super) const STRATEGY_CPU_POLICY_VERSION: u16 = 23;
pub(super) const PICKUP_CPU_POLICY_VERSION: u16 = 22;
pub(super) const ARCHIVED_CPU_POLICY_VERSION: u16 = 21;
pub(in crate::game::rooms) const SIMULATION_RANDOMNESS_VERSION: u16 = 1;
pub(in crate::game::rooms) const SIMULATION_SEED_DERIVATION_VERSION: u16 = 1;
pub(in crate::game::rooms) const ENVIRONMENT_RANDOMNESS_VERSION: u16 = 1;
pub(in crate::game::rooms) const LAIR_LAYOUT_VERSION: u16 = 1;
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

    pub(in crate::game::rooms) const fn new_with_policy(
        seat: SeatId,
        difficulty: CpuDifficulty,
        private_seed: [u8; 32],
        policy_version: u16,
    ) -> Self {
        Self {
            seat,
            difficulty,
            policy_version,
            private_seed: PrivateCpuSeed(private_seed),
        }
    }
}

#[derive(Clone, Debug)]
pub(in crate::game::rooms) struct ReplayHeader {
    pub(in crate::game::rooms) format_version: u16,
    pub(in crate::game::rooms) lifecycle_profile: u16,
    pub(in crate::game::rooms) rules_version: u16,
    pub(in crate::game::rooms) cpu_policy_version: u16,
    pub(in crate::game::rooms) generator_version: u16,
    pub(in crate::game::rooms) simulation_randomness_version: u16,
    pub(in crate::game::rooms) simulation_seed_derivation_version: u16,
    pub(in crate::game::rooms) environment_randomness_version: u16,
    pub(in crate::game::rooms) lair_layout_version: u16,
    pub(in crate::game::rooms) configuration: LairRoomConfiguration,
    pub(in crate::game::rooms) initial_settings: RoomSettings,
    pub(in crate::game::rooms) initial_settings_revision: u32,
    pub(in crate::game::rooms) initially_occupied_humans: [bool; 4],
    pub(in crate::game::rooms) simulation_root: SimulationRoot,
    pub(in crate::game::rooms) cpus: Box<[CpuProvenance]>,
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
    UpdateSpikeSettings {
        settings_revision: u32,
        settings: RoomSettings,
        cloud: StormCloudConfig,
        tuning: RespawnLairTuning,
        spike_pits: grenadiers_core::room::SpikePitTuning,
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
pub(in crate::game::rooms) struct Record {
    pub(in crate::game::rooms) tick: Tick,
    pub(in crate::game::rooms) order: u32,
    pub(in crate::game::rooms) operation: Operation,
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
    pub(in crate::game::rooms) header: ReplayHeader,
    pub(in crate::game::rooms) records: Vec<Record>,
    pub(in crate::game::rooms) status: ReplayStatus,
}

impl Journal {
    pub(in crate::game::rooms) const fn cpu_policy_version(&self) -> u16 {
        self.header.cpu_policy_version
    }

    pub(in crate::game::rooms) fn with_cpu_policy_for_test(mut self, version: u16) -> Self {
        self.header.cpu_policy_version = version;
        for cpu in &mut self.header.cpus {
            cpu.policy_version = version;
        }
        self
    }

    pub(super) fn new(header: ReplayHeader) -> Self {
        Self {
            header,
            records: Vec::with_capacity(MAX_RECORDS),
            status: ReplayStatus::Complete,
        }
    }

    pub(in crate::game::rooms) fn new_unvalidated(
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
        Ok(Self::new(ReplayHeader {
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
        }))
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

use grenadiers_core::{
    game::{ArenaConstructionProfile, EntityId, SpikePitLayoutPolicy, Tick},
    room::{BombCapacityDropMultiplier, CpuProfile},
};

use crate::game::rooms::cpu::AdmissionTrace;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Outcome {
    pub profile: CpuProfile,
    pub construction_profile: ArenaConstructionProfile,
    pub generation_version: u16,
    pub pit_layout: SpikePitLayoutPolicy,
    pub multiplier: BombCapacityDropMultiplier,
    pub moved: [bool; 3],
    pub owned_bombs: [usize; 3],
    pub event_placements: [usize; 3],
    pub destroyed_blocks: usize,
    pub flames_seen: bool,
    pub mob_births: usize,
    pub mob_moved: bool,
    pub first_bombs: [Option<BombHorizon>; 3],
    pub upgrades: [[bool; 3]; 3],
    pub attack_intents: [usize; 3],
    pub attack_bombs: [usize; 3],
    pub completed_switches: [usize; 3],
    pub stalled_switches: [usize; 3],
    pub alive: [bool; 3],
    pub ever_dead: [bool; 3],
    pub dequeued: usize,
    pub delivered: usize,
    pub cancelled: usize,
    pub failed: usize,
    pub dropped: usize,
    pub work_len: usize,
    pub max_work: u32,
    pub admissions: [AdmissionTrace; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct BombHorizon {
    pub id: EntityId,
    pub placed_at: Tick,
    pub detonated_at: Option<Tick>,
    pub finite_flames_clear_at: Option<Tick>,
    pub owner_alive_at_clear: Option<bool>,
}

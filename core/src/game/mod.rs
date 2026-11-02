//! Deterministic game rules, independent of transport, storage, and rendering.
//!
//! ```
//! use grenadiers_core::game::{
//!     ActorInit, Direction, EntityId, InputAction, Position, RulesConfig, SeatId,
//!     SeatInput, SpeedLevel, State, World, fixed_arena,
//! };
//!
//! let arena = fixed_arena();
//! let spawns = arena.spawns().to_vec();
//! let actors = [SeatId::One, SeatId::Two].map(|seat| ActorInit {
//!     id: EntityId(seat.index() as u32 + 1),
//!     seat,
//!     position: Position::center(spawns[seat.index()]),
//!     heading: Direction::East,
//!     speed: SpeedLevel::Base,
//! });
//! let mut state = State::new(RulesConfig::classic_v1(), World::new(arena), actors).unwrap();
//! let report = state.step(&[SeatInput {
//!     seat: SeatId::One,
//!     action: InputAction::Press(Direction::East),
//! }]).unwrap();
//! assert_eq!(report.tick.0, 1);
//! assert!(state.actors()[0].position().x > Position::center(spawns[0]).x);
//! ```

pub mod arena;
pub mod assignment;
pub(crate) mod blast_geometry;
mod bomb_motion;
pub mod bombs;
pub mod damage;
pub mod effects;
pub mod enabled_items;
pub mod events;
mod explosions;
pub mod fixtures;
pub mod generation;
pub mod geometry;
pub mod input;
pub mod inventory;
mod lairs;
mod mob_escape;
mod mob_hazards;
mod movement;
mod movement_collision;
pub mod oil;
pub mod pickups;
pub mod player_collision;
mod punch_geometry;
pub mod rules;
pub mod simulation;
pub mod spike_pits;
mod storm_cloud;
pub(crate) mod storm_geometry;
#[cfg(test)]
pub(crate) mod test_support;
pub mod types;
pub mod view;
pub mod world;

pub use arena::{Arena, ArenaError, Terrain};
pub use assignment::RoundAssignment;
pub use bombs::{BlastKind, Bomb, BombError, BombMotion, BombTrigger, TimedBombHistory};
pub use damage::{DamageGrace, DamageOutcome, DamagePreventionSource};
pub use effects::{EffectApplication, EffectError, EffectKind, EffectiveSpeed, TimedEffects};
pub use enabled_items::EnabledItems;
pub use events::{EventAudience, StepEvent, StepEventKind};
pub use fixtures::fixed_arena;
pub use generation::{
    ArenaGenerationConfig, EXPANDED_GENERATOR_VERSION, GENERATOR_VERSION, GeneratedArena,
    GenerationError, GenerationRecord, MULTIPLAYER_GENERATOR_VERSION, VARIABLE_GENERATOR_VERSION,
    generate_arena, generate_expanded_arena, generate_multiplayer_arena,
    generate_variable_multiplayer_arena,
};
pub use geometry::{MovementSegment, MovementTrace};
pub use input::{Direction, HeldDirections};
pub use inventory::{Equipment, Inventory, SpecialWeapon};
pub use lairs::{
    LAIR_LAYOUT_VERSION, LairStatusView, LairView, MAX_LAIRS, MAX_MOBS, MOB_UNITS_PER_SECOND,
    MobView, lair_layout_v1,
};
pub(crate) use oil::OilState;
pub use oil::{MAX_OIL_BARRELS, OilBarrelState};
pub use pickups::{Pickup, PickupKind, PickupLayer, PickupPlacementError};
pub(crate) use punch_geometry::{
    adjacent_cell as punch_adjacent_cell,
    landing_candidates_for_board as punch_landing_candidates_for_board,
    landing_is_open as punch_landing_is_open,
};
pub use rules::{MobNavigationPolicy, PunchActivationPolicy, RulesConfig, StormCloudConfig};
pub use simulation::{
    Actor, ActorInit, DrawReason, InputAction, RoundActorInit, RoundResult,
    SIMULATION_RANDOMNESS_VERSION, SIMULATION_SEED_DERIVATION_VERSION, STORM_CLOUD_HALF_EXTENT,
    STORM_CLOUD_LEG_TICKS, STORM_CLOUD_RANDOMNESS_VERSION, STORM_CLOUD_SPEED_PER_TICK,
    STORM_SLOW_TICKS, SeatInput, SimulationSeed, State, StateError, StepError, StepReport,
    StormCloudView,
};
pub(crate) use simulation::{stationary_contact_entry, stationary_sweep_contact};
pub use spike_pits::{
    MAX_SPIKE_PITS, SPIKE_PIT_LAYOUT_VERSION, SpikePitClock, SpikePitClockError, SpikePitPhase,
    SpikePitRaisedInterval, SpikePitView, position_contacts_spike_pit, segment_contacts_spike_pit,
    spike_pit_layout_v1, trace_contacts_spike_pit, traces_contact_spike_pit,
};
pub use types::{CellId, EntityId, PlayerCount, Position, SeatId, SpeedLevel, Tick};
pub use view::{ActorView, ArenaView, BombView, FlameView, OilBarrelView, PickupView, PublicState};
pub use world::World;

#[cfg(test)]
mod generated_upgrade_trace_tests;

#[cfg(test)]
mod generated_match_trace_tests;

#[cfg(test)]
mod variable_board_tests;

#[cfg(test)]
mod mob_escape_tests;

#[cfg(test)]
mod mob_hazard_phase_tests;

#[cfg(test)]
mod mob_oil_escape_tests;

#[cfg(test)]
mod invisibility_cloak_tests;

#[cfg(test)]
mod invisible_bombs_tests;
#[cfg(test)]
mod oil_route_tests;

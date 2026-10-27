//! Deterministic game rules, independent of transport, storage, and rendering.
//!
//! ```
//! use grenadiers_core::game::{
//!     ActorInit, Direction, EntityId, InputAction, Position, RulesConfig, SeatId,
//!     SeatInput, SpeedLevel, State, World, fixed_arena,
//! };
//!
//! let arena = fixed_arena();
//! let spawns = arena.spawns();
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
mod movement;
mod movement_collision;
pub mod oil;
pub mod pickups;
mod punch_geometry;
pub mod rules;
pub mod simulation;
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
    EXPANDED_GENERATOR_VERSION, GENERATOR_VERSION, GeneratedArena, GenerationRecord,
    generate_arena, generate_expanded_arena,
};
pub use geometry::{MovementSegment, MovementTrace};
pub use input::{Direction, HeldDirections};
pub use inventory::{Equipment, Inventory, SpecialWeapon};
pub(crate) use oil::OilState;
pub use oil::{MAX_OIL_BARRELS, OilBarrelState};
pub use pickups::{Pickup, PickupKind, PickupLayer, PickupPlacementError};
pub(crate) use punch_geometry::{
    adjacent_cell as punch_adjacent_cell, landing_candidates as punch_landing_candidates,
    landing_is_open as punch_landing_is_open,
};
pub use rules::RulesConfig;
pub(crate) use simulation::stationary_contact_entry;
pub use simulation::{
    Actor, ActorInit, DrawReason, InputAction, RoundResult, SeatInput, State, StateError,
    StepError, StepReport,
};
pub use types::{CellId, EntityId, Position, SeatId, SpeedLevel, Tick};
pub use view::{ActorView, ArenaView, BombView, FlameView, OilBarrelView, PickupView, PublicState};
pub use world::World;

#[cfg(test)]
mod generated_upgrade_trace_tests;

#[cfg(test)]
mod generated_match_trace_tests;

#[cfg(test)]
mod invisibility_cloak_tests;

#[cfg(test)]
mod invisible_bombs_tests;
#[cfg(test)]
mod oil_route_tests;

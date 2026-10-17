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
pub mod bombs;
pub mod events;
mod explosions;
pub mod fixtures;
pub mod generation;
pub mod geometry;
pub mod input;
mod movement;
pub mod pickups;
pub mod rules;
pub mod simulation;
#[cfg(test)]
pub(crate) mod test_support;
pub mod types;
pub mod view;
pub mod world;

pub use arena::{Arena, ArenaError, Terrain};
pub use bombs::Bomb;
pub use events::{StepEvent, StepEventKind};
pub use fixtures::fixed_arena;
pub use generation::{GENERATOR_VERSION, GeneratedArena, GenerationRecord, generate_arena};
pub use geometry::{MovementSegment, MovementTrace};
pub use input::{Direction, HeldDirections};
pub use pickups::PickupKind;
pub use rules::RulesConfig;
pub use simulation::{
    Actor, ActorInit, DrawReason, InputAction, RoundResult, SeatInput, State, StateError,
    StepError, StepReport,
};
pub use types::{CellId, EntityId, Position, SeatId, SpeedLevel, Tick};
pub use view::{ActorView, ArenaView, BombView, FlameView, PublicState};
pub use world::World;

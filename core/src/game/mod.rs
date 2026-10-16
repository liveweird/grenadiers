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
pub mod fixtures;
pub mod geometry;
pub mod input;
mod movement;
pub mod rules;
pub mod simulation;
pub mod types;
pub mod world;

pub use arena::{Arena, ArenaError, Terrain};
pub use fixtures::fixed_arena;
pub use geometry::{MovementSegment, MovementTrace};
pub use input::{Direction, HeldDirections};
pub use rules::RulesConfig;
pub use simulation::{
    Actor, ActorInit, InputAction, SeatInput, State, StateError, StepError, StepReport,
};
pub use types::{CellId, EntityId, Position, SeatId, SpeedLevel, Tick};
pub use world::{BombObstacle, World};

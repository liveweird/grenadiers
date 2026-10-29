use crate::game::arena::ARENA_CELL_COUNT;
use crate::game::{
    BlastKind, CellId, Direction, EffectKind, EntityId, Equipment, MAX_OIL_BARRELS, OilBarrelState,
    PickupKind, Position, SeatId, SpecialWeapon, SpeedLevel, Terrain, Tick,
};
use crate::room::BoardSize;
use crate::room::RoomSettings;

use super::WorldId;
pub use super::observation_environment::ObservedEnvironment;
use super::observation_environment::validate_environment;
use super::observation_validation::{
    validate_actors, validate_arena, validate_bombs, validate_cells, validate_clock,
};

pub const MAX_OBSERVED_BOMBS: usize = 16;
pub const MAX_MULTIPLAYER_OBSERVED_BOMBS: usize = 32;
pub const MAX_OBSERVED_EFFECTS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unknown {
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnknownActorFacts {
    pub movement_residual: Unknown,
    pub action_cooldowns: Unknown,
    pub collision_grace: Unknown,
}

impl UnknownActorFacts {
    pub const ALL: Self = Self {
        movement_residual: Unknown::Unknown,
        action_cooldowns: Unknown::Unknown,
        collision_grace: Unknown::Unknown,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservedRules {
    pub version: u16,
    pub tick_rate: u16,
    pub running_updates: u16,
    pub remaining_ticks: u16,
    pub settings: RoomSettings,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedArena {
    pub width: u16,
    pub height: u16,
    pub terrain: Vec<Terrain>,
    pub blocks: Vec<CellId>,
}

impl ObservedArena {
    #[must_use]
    pub fn board(&self) -> Option<BoardSize> {
        BoardSize::try_from((self.width, self.height)).ok()
    }

    #[must_use]
    pub const fn cell_count(&self) -> usize {
        self.width as usize * self.height as usize
    }

    #[must_use]
    pub const fn contains(&self, cell: CellId) -> bool {
        cell.column < self.width && cell.row < self.height
    }

    #[must_use]
    pub fn index(&self, cell: CellId) -> Option<usize> {
        self.contains(cell)
            .then(|| usize::from(cell.row) * usize::from(self.width) + usize::from(cell.column))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservedEffect {
    pub kind: EffectKind,
    pub remaining_ticks: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisibleActor {
    pub id: EntityId,
    pub seat: SeatId,
    pub position: Position,
    pub heading: Direction,
    pub speed: SpeedLevel,
    pub alive: bool,
    pub bomb_capacity: u16,
    pub bomb_range: u16,
    pub live_bomb_count: u16,
    pub equipment: Option<Equipment>,
    pub weapon: Option<SpecialWeapon>,
    pub effects: Vec<ObservedEffect>,
    pub damage_grace_remaining_ticks: u8,
    pub voluntary_actions_frozen: bool,
    pub last_teleport_tick: Option<Tick>,
    pub unknown: UnknownActorFacts,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservedActor {
    Visible(VisibleActor),
    Hidden {
        id: EntityId,
        seat: SeatId,
        alive: bool,
    },
}

impl ObservedActor {
    #[must_use]
    pub const fn id(&self) -> EntityId {
        match self {
            Self::Visible(actor) => actor.id,
            Self::Hidden { id, .. } => *id,
        }
    }

    #[must_use]
    pub const fn seat(&self) -> SeatId {
        match self {
            Self::Visible(actor) => actor.seat,
            Self::Hidden { seat, .. } => *seat,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservedBombTrigger {
    Timed { remaining_ticks: u16, paused: bool },
    Remote,
    Contact,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservedBomb {
    pub id: EntityId,
    pub placed_at: Tick,
    pub cell: CellId,
    pub owner: EntityId,
    pub range: u16,
    pub trigger: ObservedBombTrigger,
    pub blast: BlastKind,
    pub unknown: UnknownBombFacts,
}

/// Authoritative bomb bookkeeping intentionally absent from recipient views.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnknownBombFacts {
    pub motion: Unknown,
    pub interaction_history: Unknown,
}

impl UnknownBombFacts {
    pub const ALL: Self = Self {
        motion: Unknown::Unknown,
        interaction_history: Unknown::Unknown,
    };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BombKnowledge {
    /// The recipient is not allowed to know whether any bombs exist.
    Unavailable,
    /// Complete recipient-visible bomb information, which may be empty.
    Visible(Vec<ObservedBomb>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservedFlame {
    pub cell: CellId,
    pub remaining_ticks: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservedOil {
    pub cell: CellId,
    pub state: OilBarrelState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservedPickup {
    pub cell: CellId,
    pub kind: PickupKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationParts {
    pub world: WorldId,
    pub tick: Tick,
    pub recipient: SeatId,
    pub rules: ObservedRules,
    pub arena: ObservedArena,
    pub actors: Vec<ObservedActor>,
    pub bombs: BombKnowledge,
    pub flames: Vec<ObservedFlame>,
    pub oil: Vec<ObservedOil>,
    pub pickups: Vec<ObservedPickup>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    parts: ObservationParts,
    environment: Option<ObservedEnvironment>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservationError {
    InvalidClock,
    InvalidArena,
    InvalidActors,
    InvalidEffects,
    InvalidBombs,
    InvalidFlames,
    InvalidOil,
    InvalidPickups,
}

impl Observation {
    pub fn new(parts: ObservationParts) -> Result<Self, ObservationError> {
        if !(1..=6).contains(&parts.rules.version) {
            return Err(ObservationError::InvalidActors);
        }
        validate_clock(parts.tick, parts.rules)?;
        validate_arena(parts.rules.version, &parts.arena)?;
        validate_actors(
            parts.rules.version,
            parts.recipient,
            &parts.arena,
            &parts.actors,
        )?;
        validate_bombs(parts.tick, parts.rules.version, &parts.arena, &parts.bombs)?;
        if parts.flames.len() > ARENA_CELL_COUNT {
            return Err(ObservationError::InvalidFlames);
        }
        validate_cells(&parts.arena, &parts.flames, |value| value.cell)
            .map_err(|()| ObservationError::InvalidFlames)?;
        if parts.oil.len() > MAX_OIL_BARRELS {
            return Err(ObservationError::InvalidOil);
        }
        validate_cells(&parts.arena, &parts.oil, |value| value.cell)
            .map_err(|()| ObservationError::InvalidOil)?;
        if parts.pickups.len() > ARENA_CELL_COUNT {
            return Err(ObservationError::InvalidPickups);
        }
        validate_cells(&parts.arena, &parts.pickups, |value| value.cell)
            .map_err(|()| ObservationError::InvalidPickups)?;
        Ok(Self {
            parts,
            environment: None,
        })
    }

    pub fn new_rules_v7(
        parts: ObservationParts,
        environment: ObservedEnvironment,
    ) -> Result<Self, ObservationError> {
        if parts.rules.version != 7 {
            return Err(ObservationError::InvalidActors);
        }
        validate_clock(parts.tick, parts.rules)?;
        validate_arena(parts.rules.version, &parts.arena)?;
        validate_actors(
            parts.rules.version,
            parts.recipient,
            &parts.arena,
            &parts.actors,
        )?;
        validate_bombs(parts.tick, parts.rules.version, &parts.arena, &parts.bombs)?;
        if parts.flames.len() > ARENA_CELL_COUNT {
            return Err(ObservationError::InvalidFlames);
        }
        validate_cells(&parts.arena, &parts.flames, |value| value.cell)
            .map_err(|()| ObservationError::InvalidFlames)?;
        if parts.oil.len() > MAX_OIL_BARRELS {
            return Err(ObservationError::InvalidOil);
        }
        validate_cells(&parts.arena, &parts.oil, |value| value.cell)
            .map_err(|()| ObservationError::InvalidOil)?;
        if parts.pickups.len() > ARENA_CELL_COUNT {
            return Err(ObservationError::InvalidPickups);
        }
        validate_cells(&parts.arena, &parts.pickups, |value| value.cell)
            .map_err(|()| ObservationError::InvalidPickups)?;
        validate_environment(&parts, &environment)?;
        Ok(Self {
            parts,
            environment: Some(environment),
        })
    }

    #[must_use]
    pub const fn parts(&self) -> &ObservationParts {
        &self.parts
    }

    #[must_use]
    pub const fn environment(&self) -> Option<&ObservedEnvironment> {
        self.environment.as_ref()
    }

    #[must_use]
    pub fn into_parts(self) -> ObservationParts {
        self.parts
    }

    pub fn map_parts(
        self,
        map: impl FnOnce(&mut ObservationParts),
    ) -> Result<Self, ObservationError> {
        let Self {
            mut parts,
            environment,
        } = self;
        map(&mut parts);
        match environment {
            Some(environment) => Self::new_rules_v7(parts, environment),
            None => Self::new(parts),
        }
    }
}

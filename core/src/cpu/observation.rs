use crate::game::arena::ARENA_CELL_COUNT;
use crate::game::{
    CellId, Direction, EffectKind, EntityId, Equipment, MAX_OIL_BARRELS, OilBarrelState,
    PickupKind, Position, PunchActivationPolicy, SeatId, SpecialWeapon, SpeedLevel, Terrain, Tick,
};
use crate::room::BoardSize;
use crate::room::RoomSettings;

use super::WorldId;
pub use super::observation_bombs::{
    BombKnowledge, ObservedBomb, ObservedBombTrigger, UnknownBombFacts,
};
pub use super::observation_environment::ObservedEnvironment;
use super::observation_environment::validate_environment;
pub use super::observation_lairs::ObservedLairs;
use super::observation_lairs::validate_lairs;
pub use super::observation_respawn_lairs::ObservedRespawnLairs;
use super::observation_respawn_lairs::validate_respawn_lairs;
pub use super::observation_spike_pits::ObservedSpikePits;
use super::observation_spike_pits::validate_spike_pits;
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
    pub punch_activation_policy: PunchActivationPolicy,
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
    lairs: Option<ObservedLairs>,
    respawn_lairs: Option<ObservedRespawnLairs>,
    spike_pits: Option<ObservedSpikePits>,
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
        validate_common(&parts)?;
        Ok(Self {
            parts,
            environment: None,
            lairs: None,
            respawn_lairs: None,
            spike_pits: None,
        })
    }

    pub fn new_rules_v7(
        parts: ObservationParts,
        environment: ObservedEnvironment,
    ) -> Result<Self, ObservationError> {
        if parts.rules.version != 7 {
            return Err(ObservationError::InvalidActors);
        }
        validate_common(&parts)?;
        validate_environment(&parts, &environment)?;
        Ok(Self {
            parts,
            environment: Some(environment),
            lairs: None,
            respawn_lairs: None,
            spike_pits: None,
        })
    }

    /// Constructs the rules-v8 recipient view. Overlay identity/count failures
    /// use `InvalidActors` so every older exhaustive transport mapping remains
    /// closed while v8 adds mob entity facts outside `ObservationParts`.
    pub fn new_rules_v8(
        parts: ObservationParts,
        environment: ObservedEnvironment,
        lairs: ObservedLairs,
    ) -> Result<Self, ObservationError> {
        if parts.rules.version != 8 {
            return Err(ObservationError::InvalidActors);
        }
        validate_common(&parts)?;
        validate_environment(&parts, &environment)?;
        validate_lairs(&parts, &lairs)?;
        Ok(Self {
            parts,
            environment: Some(environment),
            lairs: Some(lairs),
            respawn_lairs: None,
            spike_pits: None,
        })
    }

    /// Constructs a rules-v9 recipient view. `None` is the disabled-lair overlay;
    /// an enabled overlay carries independently nullable replacement deadlines.
    pub fn new_rules_v9(
        parts: ObservationParts,
        environment: ObservedEnvironment,
        respawn_lairs: Option<ObservedRespawnLairs>,
    ) -> Result<Self, ObservationError> {
        if parts.rules.version != 9 {
            return Err(ObservationError::InvalidActors);
        }
        validate_common(&parts)?;
        validate_environment(&parts, &environment)?;
        if let Some(lairs) = &respawn_lairs {
            validate_respawn_lairs(&parts, lairs)?;
        }
        Ok(Self {
            parts,
            environment: Some(environment),
            lairs: None,
            respawn_lairs,
            spike_pits: None,
        })
    }

    /// Constructs the dormant rules-v10 recipient view. An absent spike-pit
    /// overlay means disabled; an enabled overlay may contain no placed sites.
    pub fn new_rules_v10(
        parts: ObservationParts,
        environment: ObservedEnvironment,
        respawn_lairs: Option<ObservedRespawnLairs>,
        spike_pits: Option<ObservedSpikePits>,
    ) -> Result<Self, ObservationError> {
        if parts.rules.version != 10 {
            return Err(ObservationError::InvalidActors);
        }
        validate_common(&parts)?;
        validate_environment(&parts, &environment)?;
        if let Some(lairs) = &respawn_lairs {
            validate_respawn_lairs(&parts, lairs)?;
        }
        if let Some(pits) = &spike_pits {
            validate_spike_pits(&parts, respawn_lairs.as_ref(), pits)?;
        }
        Ok(Self {
            parts,
            environment: Some(environment),
            lairs: None,
            respawn_lairs,
            spike_pits,
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
    pub const fn lairs(&self) -> Option<&ObservedLairs> {
        self.lairs.as_ref()
    }

    #[must_use]
    pub const fn respawn_lairs(&self) -> Option<&ObservedRespawnLairs> {
        self.respawn_lairs.as_ref()
    }

    #[must_use]
    pub const fn spike_pits(&self) -> Option<&ObservedSpikePits> {
        self.spike_pits.as_ref()
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
            lairs,
            respawn_lairs,
            spike_pits,
        } = self;
        map(&mut parts);
        match (
            environment,
            lairs,
            respawn_lairs,
            spike_pits,
            parts.rules.version,
        ) {
            (Some(environment), Some(lairs), None, None, 8) => {
                Self::new_rules_v8(parts, environment, lairs)
            }
            (Some(environment), None, respawn_lairs, None, 9) => {
                Self::new_rules_v9(parts, environment, respawn_lairs)
            }
            (Some(environment), None, respawn_lairs, spike_pits, 10) => {
                Self::new_rules_v10(parts, environment, respawn_lairs, spike_pits)
            }
            (Some(environment), None, None, None, 7) => Self::new_rules_v7(parts, environment),
            (None, None, None, None, 1..=6) => Self::new(parts),
            _ => Err(ObservationError::InvalidActors),
        }
    }
}

fn validate_common(parts: &ObservationParts) -> Result<(), ObservationError> {
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
        .map_err(|()| ObservationError::InvalidPickups)
}

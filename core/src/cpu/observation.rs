use std::collections::BTreeSet;

use crate::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH};
use crate::game::{
    BlastKind, CellId, Direction, EffectKind, EntityId, Equipment, MAX_OIL_BARRELS, OilBarrelState,
    PickupKind, Position, SeatId, SpecialWeapon, SpeedLevel, Terrain, Tick,
};
use crate::room::RoomSettings;

use super::WorldId;

pub const MAX_OBSERVED_BOMBS: usize = 16;
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
    pub actors: [ObservedActor; 2],
    pub bombs: BombKnowledge,
    pub flames: Vec<ObservedFlame>,
    pub oil: Vec<ObservedOil>,
    pub pickups: Vec<ObservedPickup>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation(ObservationParts);

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
        validate_clock(parts.tick, parts.rules)?;
        validate_arena(&parts.arena)?;
        validate_actors(parts.recipient, &parts.actors)?;
        validate_bombs(parts.tick, &parts.arena, &parts.bombs)?;
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
        Ok(Self(parts))
    }

    #[must_use]
    pub const fn parts(&self) -> &ObservationParts {
        &self.0
    }

    #[must_use]
    pub fn into_parts(self) -> ObservationParts {
        self.0
    }
}

fn validate_clock(tick: Tick, rules: ObservedRules) -> Result<(), ObservationError> {
    if rules.tick_rate == 0
        || u64::from(rules.running_updates) > tick.0
        || rules
            .running_updates
            .checked_add(rules.remaining_ticks)
            .is_none()
    {
        return Err(ObservationError::InvalidClock);
    }
    Ok(())
}

fn validate_arena(arena: &ObservedArena) -> Result<(), ObservationError> {
    if arena.width != ARENA_WIDTH
        || arena.height != ARENA_HEIGHT
        || arena.terrain.len() != ARENA_CELL_COUNT
        || arena.blocks.len() > ARENA_CELL_COUNT
    {
        return Err(ObservationError::InvalidArena);
    }
    validate_cells(arena, &arena.blocks, |cell| *cell).map_err(|()| ObservationError::InvalidArena)
}

fn validate_actors(recipient: SeatId, actors: &[ObservedActor; 2]) -> Result<(), ObservationError> {
    if actors[0].seat() == actors[1].seat() || actors[0].id() == actors[1].id() {
        return Err(ObservationError::InvalidActors);
    }
    let Some(actor) = actors.iter().find(|actor| actor.seat() == recipient) else {
        return Err(ObservationError::InvalidActors);
    };
    if !matches!(actor, ObservedActor::Visible(_)) {
        return Err(ObservationError::InvalidActors);
    }
    for actor in actors {
        if let ObservedActor::Visible(actor) = actor {
            validate_effects(&actor.effects)?;
            if actor.unknown != UnknownActorFacts::ALL {
                return Err(ObservationError::InvalidActors);
            }
        }
    }
    Ok(())
}

fn validate_effects(effects: &[ObservedEffect]) -> Result<(), ObservationError> {
    if effects.len() > MAX_OBSERVED_EFFECTS
        || effects.iter().any(|effect| effect.remaining_ticks == 0)
    {
        return Err(ObservationError::InvalidEffects);
    }
    let mut ordinary = BTreeSet::new();
    let mut slow_foot = 0;
    for effect in effects {
        if effect.kind == EffectKind::SlowFoot {
            slow_foot += 1;
            if slow_foot > crate::game::effects::MAX_SLOW_FOOT_STACKS {
                return Err(ObservationError::InvalidEffects);
            }
        } else if !ordinary.insert(effect.kind) {
            return Err(ObservationError::InvalidEffects);
        }
    }
    Ok(())
}

fn validate_bombs(
    tick: Tick,
    arena: &ObservedArena,
    bombs: &BombKnowledge,
) -> Result<(), ObservationError> {
    let BombKnowledge::Visible(bombs) = bombs else {
        return Ok(());
    };
    if bombs.len() > MAX_OBSERVED_BOMBS {
        return Err(ObservationError::InvalidBombs);
    }
    let mut ids = BTreeSet::new();
    if bombs.iter().any(|bomb| {
        bomb.unknown != UnknownBombFacts::ALL
            || bomb.placed_at > tick
            || !contains(arena, bomb.cell)
            || !ids.insert(bomb.id)
    }) {
        return Err(ObservationError::InvalidBombs);
    }
    Ok(())
}

fn validate_cells<T>(
    arena: &ObservedArena,
    values: &[T],
    cell: impl Fn(&T) -> CellId,
) -> Result<(), ()> {
    let mut occupied = BTreeSet::new();
    if values
        .iter()
        .map(cell)
        .any(|cell| !contains(arena, cell) || !occupied.insert(cell))
    {
        return Err(());
    }
    Ok(())
}

fn contains(arena: &ObservedArena, cell: CellId) -> bool {
    cell.column < arena.width && cell.row < arena.height
}

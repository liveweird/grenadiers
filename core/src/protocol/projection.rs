use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::game;

use super::scalars::{Cell, Direction, EntityId, Position, RulesVersion, Seat, Tick};
use super::server::{DrawReason, WireRoundResult};
use super::snapshot::{
    ARENA_TERRAIN_CELLS, ActorView, ArenaView, BombView, FlameView, GameState, MAX_BLOCKS,
    MAX_BOMBS, MAX_FLAMES, MAX_PICKUPS, MAX_RUNNING_UPDATES, PickupKind, PickupView, Terrain,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectedRound {
    pub tick: Tick,
    pub rules_version: RulesVersion,
    pub result: Option<WireRoundResult>,
    pub state: GameState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionError {
    UnsupportedRulesVersion,
    InvalidTick,
    InvalidRunningClock,
    InvalidArena,
    InvalidActors,
    InvalidBombs,
    UnsupportedBombTrigger,
    UnsupportedBombBlast,
    InvalidFlames,
    InvalidPickups,
    InvalidResult,
    InvalidReportWindow,
    InvalidEvent,
}

impl fmt::Display for ProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedRulesVersion => "unsupported rules version",
            Self::InvalidTick => "invalid projected tick",
            Self::InvalidRunningClock => "invalid running clock",
            Self::InvalidArena => "invalid arena projection",
            Self::InvalidActors => "invalid actor projection",
            Self::InvalidBombs => "invalid bomb projection",
            Self::UnsupportedBombTrigger => "unsupported bomb trigger",
            Self::UnsupportedBombBlast => "unsupported bomb blast",
            Self::InvalidFlames => "invalid flame projection",
            Self::InvalidPickups => "invalid pickup projection",
            Self::InvalidResult => "invalid round result projection",
            Self::InvalidReportWindow => "invalid event report window",
            Self::InvalidEvent => "invalid projected event",
        })
    }
}

impl std::error::Error for ProjectionError {}

pub fn project_state(view: &game::PublicState) -> Result<ProjectedRound, ProjectionError> {
    let rules_version = RulesVersion::try_from(view.rules_version)
        .map_err(|_| ProjectionError::UnsupportedRulesVersion)?;
    let tick = wire_tick(view.tick)?;
    let running_updates =
        u16::try_from(view.running_updates).map_err(|_| ProjectionError::InvalidRunningClock)?;
    let remaining_ticks =
        u16::try_from(view.remaining_ticks).map_err(|_| ProjectionError::InvalidRunningClock)?;
    if running_updates > MAX_RUNNING_UPDATES
        || remaining_ticks > MAX_RUNNING_UPDATES
        || running_updates.checked_add(remaining_ticks) != Some(MAX_RUNNING_UPDATES)
    {
        return Err(ProjectionError::InvalidRunningClock);
    }

    let arena = project_arena(&view.arena)?;
    let actors = project_actors(view)?;
    let bombs = project_bombs(view, &actors)?;
    let flames = project_flames(view)?;
    let pickups = project_pickups(view)?;
    let result = project_result(view.result);
    validate_result(view, result)?;

    Ok(ProjectedRound {
        tick,
        rules_version,
        result,
        state: GameState {
            running_updates,
            remaining_ticks,
            arena,
            actors,
            bombs,
            flames,
            pickups,
        },
    })
}

fn project_arena(view: &game::ArenaView) -> Result<ArenaView, ProjectionError> {
    if view.width != 15
        || view.height != 13
        || view.terrain.len() != ARENA_TERRAIN_CELLS
        || view.blocks.len() > MAX_BLOCKS
        || !strictly_sorted(&view.blocks)
    {
        return Err(ProjectionError::InvalidArena);
    }
    let terrain = view
        .terrain
        .iter()
        .map(|terrain| match terrain {
            game::Terrain::Floor => Terrain::Floor,
            game::Terrain::Wall => Terrain::Wall,
        })
        .collect();
    let mut blocks = Vec::with_capacity(view.blocks.len());
    for &cell in &view.blocks {
        if terrain_at(view, cell) != Some(game::Terrain::Floor) {
            return Err(ProjectionError::InvalidArena);
        }
        blocks.push(wire_cell(cell).map_err(|_| ProjectionError::InvalidArena)?);
    }
    Ok(ArenaView {
        width: 15,
        height: 13,
        terrain,
        blocks,
    })
}

fn project_actors(view: &game::PublicState) -> Result<[ActorView; 2], ProjectionError> {
    if view.actors[0].seat != game::SeatId::One
        || view.actors[1].seat != game::SeatId::Two
        || view.actors[0].id == view.actors[1].id
    {
        return Err(ProjectionError::InvalidActors);
    }
    Ok([
        project_actor(view.actors[0])?,
        project_actor(view.actors[1])?,
    ])
}

fn project_actor(actor: game::ActorView) -> Result<ActorView, ProjectionError> {
    if !(1..=8).contains(&actor.bomb_capacity)
        || !(1..=8).contains(&actor.bomb_range)
        || actor.live_bomb_count > 8
        || actor.live_bomb_count > usize::from(actor.bomb_capacity)
    {
        return Err(ProjectionError::InvalidActors);
    }
    let x = u16::try_from(actor.position.x).map_err(|_| ProjectionError::InvalidActors)?;
    let y = u16::try_from(actor.position.y).map_err(|_| ProjectionError::InvalidActors)?;
    Ok(ActorView {
        id: EntityId::new(actor.id.0),
        seat: wire_seat(actor.seat),
        position: Position::new(x, y).map_err(|_| ProjectionError::InvalidActors)?,
        heading: wire_direction(actor.heading),
        speed_level: match actor.speed {
            game::SpeedLevel::Base => 0,
            game::SpeedLevel::One => 1,
            game::SpeedLevel::Two => 2,
            game::SpeedLevel::Three => 3,
            game::SpeedLevel::Four => 4,
        },
        alive: actor.alive,
        bomb_capacity: actor.bomb_capacity as u8,
        bomb_range: actor.bomb_range as u8,
        live_bomb_count: actor.live_bomb_count as u8,
    })
}

fn project_bombs(
    view: &game::PublicState,
    actors: &[ActorView; 2],
) -> Result<Vec<BombView>, ProjectionError> {
    if view.bombs.len() > MAX_BOMBS || !view.bombs.windows(2).all(|pair| pair[0].id < pair[1].id) {
        return Err(ProjectionError::InvalidBombs);
    }
    let actor_ids = BTreeSet::from([view.actors[0].id, view.actors[1].id]);
    let block_cells = view.arena.blocks.iter().copied().collect::<BTreeSet<_>>();
    let mut bomb_cells = BTreeSet::new();
    let mut owner_counts = BTreeMap::new();
    let mut bombs = Vec::with_capacity(view.bombs.len());
    for &bomb in &view.bombs {
        let projected = project_bomb_record(bomb)?;
        if actor_ids.contains(&bomb.id)
            || !actor_ids.contains(&bomb.owner)
            || !bomb_cells.insert(bomb.cell)
            || block_cells.contains(&bomb.cell)
            || terrain_at(&view.arena, bomb.cell) != Some(game::Terrain::Floor)
            || bomb.placed_at.0 > view.tick.0
            || (view.result.is_none() && projected.detonates_at.get() <= view.tick.0)
        {
            return Err(ProjectionError::InvalidBombs);
        }
        *owner_counts.entry(bomb.owner).or_insert(0_usize) += 1;
        bombs.push(projected);
    }
    for (domain, projected) in view.actors.iter().zip(actors) {
        let actual = owner_counts.get(&domain.id).copied().unwrap_or(0);
        if actual != domain.live_bomb_count || actual > usize::from(projected.bomb_capacity) {
            return Err(ProjectionError::InvalidBombs);
        }
    }
    Ok(bombs)
}

fn project_flames(view: &game::PublicState) -> Result<Vec<FlameView>, ProjectionError> {
    if view.flames.len() > MAX_FLAMES
        || !view
            .flames
            .windows(2)
            .all(|pair| pair[0].cell < pair[1].cell)
    {
        return Err(ProjectionError::InvalidFlames);
    }
    view.flames
        .iter()
        .map(|flame| {
            if terrain_at(&view.arena, flame.cell) != Some(game::Terrain::Floor)
                || (view.result.is_none() && flame.expires_at.0 <= view.tick.0)
            {
                return Err(ProjectionError::InvalidFlames);
            }
            Ok(FlameView {
                cell: wire_cell(flame.cell).map_err(|_| ProjectionError::InvalidFlames)?,
                expires_at: wire_tick(flame.expires_at)?,
            })
        })
        .collect()
}

fn validate_result(
    view: &game::PublicState,
    result: Option<WireRoundResult>,
) -> Result<(), ProjectionError> {
    let alive = [view.actors[0].alive, view.actors[1].alive];
    let valid = match (result, alive) {
        (None, [true, true]) => view.running_updates < u64::from(MAX_RUNNING_UPDATES),
        (Some(WireRoundResult::Winner { seat: Seat::One }), [true, false])
        | (Some(WireRoundResult::Winner { seat: Seat::Two }), [false, true])
        | (
            Some(WireRoundResult::Draw {
                reason: DrawReason::MutualElimination,
            }),
            [false, false],
        ) => true,
        (
            Some(WireRoundResult::Draw {
                reason: DrawReason::Timeout,
            }),
            [true, true],
        ) => view.running_updates == u64::from(MAX_RUNNING_UPDATES),
        _ => false,
    };
    valid.then_some(()).ok_or(ProjectionError::InvalidResult)
}

pub(super) fn project_bomb_record(bomb: game::BombView) -> Result<BombView, ProjectionError> {
    if bomb.blast != game::BlastKind::Standard {
        return Err(ProjectionError::UnsupportedBombBlast);
    }
    let detonates_at = match bomb.trigger {
        game::BombTrigger::Timed { detonates_at } => detonates_at,
        game::BombTrigger::Remote | game::BombTrigger::Contact => {
            return Err(ProjectionError::UnsupportedBombTrigger);
        }
    };
    if !(1..=8).contains(&bomb.range) || bomb.placed_at.0 > detonates_at.0 {
        return Err(ProjectionError::InvalidBombs);
    }
    Ok(BombView {
        id: EntityId::new(bomb.id.0),
        cell: wire_cell(bomb.cell).map_err(|_| ProjectionError::InvalidBombs)?,
        owner: EntityId::new(bomb.owner.0),
        range: bomb.range as u8,
        placed_at: wire_tick(bomb.placed_at)?,
        detonates_at: wire_tick(detonates_at)?,
    })
}

pub(super) fn project_result(result: Option<game::RoundResult>) -> Option<WireRoundResult> {
    result.map(|result| match result {
        game::RoundResult::Winner(seat) => WireRoundResult::Winner {
            seat: wire_seat(seat),
        },
        game::RoundResult::Draw(game::DrawReason::MutualElimination) => WireRoundResult::Draw {
            reason: DrawReason::MutualElimination,
        },
        game::RoundResult::Draw(game::DrawReason::Timeout) => WireRoundResult::Draw {
            reason: DrawReason::Timeout,
        },
    })
}

pub(super) fn wire_tick(tick: game::Tick) -> Result<Tick, ProjectionError> {
    Tick::new(tick.0).map_err(|_| ProjectionError::InvalidTick)
}

pub(super) fn wire_cell(cell: game::CellId) -> Result<Cell, ProjectionError> {
    let column = u8::try_from(cell.column).map_err(|_| ProjectionError::InvalidArena)?;
    let row = u8::try_from(cell.row).map_err(|_| ProjectionError::InvalidArena)?;
    Cell::new(column, row).map_err(|_| ProjectionError::InvalidArena)
}

pub(super) const fn wire_seat(seat: game::SeatId) -> Seat {
    match seat {
        game::SeatId::One => Seat::One,
        game::SeatId::Two => Seat::Two,
    }
}

fn wire_direction(direction: game::Direction) -> Direction {
    match direction {
        game::Direction::North => Direction::North,
        game::Direction::East => Direction::East,
        game::Direction::South => Direction::South,
        game::Direction::West => Direction::West,
    }
}

fn terrain_at(arena: &game::ArenaView, cell: game::CellId) -> Option<game::Terrain> {
    if cell.column >= arena.width || cell.row >= arena.height {
        return None;
    }
    arena
        .terrain
        .get(usize::from(cell.row) * usize::from(arena.width) + usize::from(cell.column))
        .copied()
}

fn strictly_sorted<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn project_pickups(view: &game::PublicState) -> Result<Vec<PickupView>, ProjectionError> {
    if view.pickups.len() > MAX_PICKUPS
        || !view
            .pickups
            .windows(2)
            .all(|pair| pair[0].cell < pair[1].cell)
    {
        return Err(ProjectionError::InvalidPickups);
    }
    view.pickups
        .iter()
        .map(|pickup| {
            if terrain_at(&view.arena, pickup.cell) != Some(game::Terrain::Floor)
                || view.arena.blocks.binary_search(&pickup.cell).is_ok()
                || view
                    .flames
                    .binary_search_by_key(&pickup.cell, |flame| flame.cell)
                    .is_ok()
            {
                return Err(ProjectionError::InvalidPickups);
            }
            Ok(PickupView {
                cell: wire_cell(pickup.cell).map_err(|_| ProjectionError::InvalidPickups)?,
                kind: wire_pickup_kind(pickup.kind).ok_or(ProjectionError::InvalidPickups)?,
            })
        })
        .collect()
}

pub(super) const fn wire_pickup_kind(kind: game::PickupKind) -> Option<PickupKind> {
    match kind {
        game::PickupKind::BombCapacity => Some(PickupKind::BombCapacity),
        game::PickupKind::BlastRange => Some(PickupKind::BlastRange),
        game::PickupKind::MovementSpeed => Some(PickupKind::MovementSpeed),
        game::PickupKind::WallPass
        | game::PickupKind::Heart
        | game::PickupKind::RemoteBomb
        | game::PickupKind::PiercingBomb
        | game::PickupKind::PowerBomb
        | game::PickupKind::Landmine
        | game::PickupKind::Shield
        | game::PickupKind::Stopwatch
        | game::PickupKind::InvisibilityCloak
        | game::PickupKind::GoldenFlame
        | game::PickupKind::ShortFuse
        | game::PickupKind::Diarrhea
        | game::PickupKind::SlowFoot
        | game::PickupKind::ShortFlame
        | game::PickupKind::LongFuse
        | game::PickupKind::ReverseControls
        | game::PickupKind::InvisibleBombs
        | game::PickupKind::Teleporter => None,
    }
}

use std::collections::BTreeSet;

use super::arena::Terrain;
use super::geometry::{boundary_distance, distance_to_blocking_cell, position_overlaps_cell};
use super::input::Direction;
use super::player_collision::{PlayerObstacle, clip_player_segment, player_contact_distance};
use super::simulation::Actor;
use super::types::{CellId, EntityId, HALF_FOOTPRINT, Position, SeatId, UNITS_PER_CELL};
use super::world::World;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Collision {
    pub(crate) distance: i32,
    pub(crate) bomb: Option<EntityId>,
    pub(crate) bomb_contact: Option<EntityId>,
    pub(crate) ambiguous: bool,
    pub(crate) physical_contact: Option<SeatId>,
}

#[derive(Clone, Copy)]
pub(crate) struct ObstacleAccess<'a> {
    pub(crate) can_pass: bool,
    pub(crate) soft_grace: &'a BTreeSet<CellId>,
    pub(crate) oil_grace: &'a BTreeSet<CellId>,
}

pub(crate) fn position_is_free(
    world: &World,
    position: Position,
    bomb_grace: &BTreeSet<EntityId>,
    can_pass_soft_walls: bool,
    soft_wall_grace: &BTreeSet<CellId>,
    unlit_oil_grace: &BTreeSet<CellId>,
) -> bool {
    let x = i64::from(position.x);
    let y = i64::from(position.y);
    let half = i64::from(HALF_FOOTPRINT);
    if x - half < 0
        || y - half < 0
        || x + half > i64::from(world.arena.width()) * i64::from(UNITS_PER_CELL)
        || y + half > i64::from(world.arena.height()) * i64::from(UNITS_PER_CELL)
    {
        return false;
    }
    blocking_cells(world).all(|(cell, bomb_id, soft, oil)| {
        !position_overlaps_cell(position, cell)
            || bomb_id.is_some_and(|id| bomb_grace.contains(&id))
            || soft && (can_pass_soft_walls || soft_wall_grace.contains(&cell))
            || oil && (can_pass_soft_walls || unlit_oil_grace.contains(&cell))
    })
}

pub(crate) fn allowed_distance(
    world: &World,
    position: Position,
    direction: Direction,
    requested: i32,
    bomb_grace: &BTreeSet<EntityId>,
    access: ObstacleAccess<'_>,
) -> i32 {
    allowed_distance_with_blocker(world, position, direction, requested, bomb_grace, access)
        .distance
}

pub(crate) fn allowed_distance_with_blocker(
    world: &World,
    position: Position,
    direction: Direction,
    requested: i32,
    bomb_grace: &BTreeSet<EntityId>,
    access: ObstacleAccess<'_>,
) -> Collision {
    allowed_distance_with_players(
        world, position, direction, requested, bomb_grace, access, None,
    )
}

pub(crate) fn allowed_distance_with_players(
    world: &World,
    position: Position,
    direction: Direction,
    requested: i32,
    bomb_grace: &BTreeSet<EntityId>,
    access: ObstacleAccess<'_>,
    players: Option<(SeatId, &[PlayerObstacle])>,
) -> Collision {
    let mut allowed = requested.max(0);
    let boundary = boundary_distance(
        position,
        direction,
        allowed,
        world.arena.width(),
        world.arena.height(),
    );
    let mut bomb = None;
    let mut ambiguous = boundary < allowed;
    let mut physical_contact = None;
    let mut contact = (boundary < requested.max(0)).then_some((boundary, None));
    let mut contact_ambiguous = false;
    allowed = allowed.min(boundary);
    for (cell, bomb_id, soft, oil) in blocking_cells(world) {
        if bomb_id.is_some_and(|id| bomb_grace.contains(&id))
            || soft && (access.can_pass || access.soft_grace.contains(&cell))
            || oil && (access.can_pass || access.oil_grace.contains(&cell))
        {
            continue;
        }
        let Some(candidate) = distance_to_blocking_cell(position, direction, cell) else {
            continue;
        };
        if candidate <= requested.max(0) {
            match contact {
                None => contact = Some((candidate, bomb_id)),
                Some((distance, _)) if candidate < distance => {
                    contact = Some((candidate, bomb_id));
                    contact_ambiguous = false;
                }
                Some((distance, _)) if candidate == distance => contact_ambiguous = true,
                Some(_) => {}
            }
        }
        if candidate < allowed {
            allowed = candidate;
            bomb = bomb_id;
            ambiguous = false;
        } else if candidate == allowed && candidate < requested.max(0) {
            ambiguous = true;
        }
    }
    if let Some((seat, obstacles)) = players {
        let player = clip_player_segment(seat, position, direction, requested, obstacles);
        let player_contact =
            player_contact_distance(seat, position, direction, requested, obstacles)
                .into_iter()
                .chain((player.distance < requested.max(0)).then_some(player.distance))
                .min();
        if let Some(player_distance) = player_contact {
            match contact {
                None => contact = Some((player_distance, None)),
                Some((distance, _)) if player_distance < distance => {
                    contact = Some((player_distance, None));
                    contact_ambiguous = false;
                }
                Some((distance, _)) if player_distance == distance => contact_ambiguous = true,
                Some(_) => {}
            }
        }
        if player.distance < allowed {
            allowed = player.distance;
            bomb = None;
            ambiguous = false;
            physical_contact = player.physical_contact;
        } else if player.distance == allowed && player.distance < requested.max(0) {
            bomb = None;
            ambiguous = true;
            physical_contact = player.physical_contact;
        }
    }
    let bomb_contact = if contact_ambiguous {
        None
    } else {
        contact.and_then(|(_, bomb)| bomb)
    };
    Collision {
        distance: allowed.max(0),
        bomb,
        bomb_contact,
        ambiguous,
        physical_contact,
    }
}

pub(crate) fn overlapping_soft_blocks(world: &World, position: Position) -> BTreeSet<CellId> {
    world
        .arena
        .blocks()
        .iter()
        .filter(|cell| position_overlaps_cell(position, **cell))
        .copied()
        .collect()
}

pub(crate) fn prune_soft_wall_grace(world: &World, actor: &mut Actor) {
    actor.soft_wall_grace.retain(|cell| {
        world.arena.blocks().contains(cell) && position_overlaps_cell(actor.position, *cell)
    });
}

pub(crate) fn overlapping_unlit_oil(world: &World, position: Position) -> BTreeSet<CellId> {
    world
        .oil
        .iter()
        .filter_map(|(cell, state)| {
            (state == crate::game::OilState::Unlit && position_overlaps_cell(position, cell))
                .then_some(cell)
        })
        .collect()
}

pub(crate) fn prune_unlit_oil_grace(world: &World, actor: &mut Actor) {
    actor
        .unlit_oil_grace
        .retain(|cell| world.oil.unlit(*cell) && position_overlaps_cell(actor.position, *cell));
}

pub(crate) fn prune_bomb_grace(world: &World, actor: &mut Actor) {
    actor.bomb_grace.retain(|id| {
        world
            .bombs
            .get(id)
            .is_some_and(|bomb| position_overlaps_cell(actor.position, bomb.cell()))
    });
}

fn blocking_cells(
    world: &World,
) -> impl Iterator<Item = (CellId, Option<EntityId>, bool, bool)> + '_ {
    let terrain = (0..world.arena.height()).flat_map(move |row| {
        (0..world.arena.width()).filter_map(move |column| {
            let cell = CellId::new(column, row);
            if world.arena.terrain(cell) == Some(Terrain::Wall) {
                Some((cell, None, false, false))
            } else if world.arena.blocks().contains(&cell) {
                Some((cell, None, true, false))
            } else {
                None
            }
        })
    });
    terrain
        .chain(world.oil.iter().filter_map(|(cell, state)| {
            (state == crate::game::OilState::Unlit).then_some((cell, None, false, true))
        }))
        .chain(
            world
                .bombs
                .values()
                .filter(|bomb| !matches!(bomb.trigger(), super::BombTrigger::Contact))
                .map(|bomb| (bomb.cell(), Some(bomb.id()), false, false)),
        )
}

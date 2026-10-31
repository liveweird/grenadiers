use std::collections::{BTreeSet, VecDeque};

use super::Mob;
use crate::game::MobNavigationPolicy;
use crate::game::geometry::{MovementSegment, MovementTrace};
use crate::game::input::Direction;
use crate::game::movement_collision::{ObstacleAccess, allowed_distance};
use crate::game::simulation::Actor;
use crate::game::types::{CellId, Position, TICKS_PER_SECOND, Tick};
use crate::game::world::World;

type Distances = Vec<Option<u16>>;

pub(super) fn target_searches(
    world: &World,
    actors: &[Actor],
    target_order: &[usize],
    tick: Tick,
    danger: &BTreeSet<CellId>,
) -> Vec<(usize, Distances)> {
    target_order
        .iter()
        .enumerate()
        .filter_map(|(rank, &index)| {
            let actor = &actors[index];
            (actor.alive() && !actor.effects().hidden_from_opponents(tick)).then_some((rank, actor))
        })
        .filter_map(|(rank, actor)| {
            actor.position().cell().and_then(|cell| {
                super::traversable(cell, world, danger, tick)
                    .then(|| (rank, bfs(world, cell, danger, tick)))
            })
        })
        .collect()
}

fn bfs(world: &World, start: CellId, danger: &BTreeSet<CellId>, tick: Tick) -> Distances {
    let mut distances = vec![None; world.arena.cell_count()];
    let mut pending = VecDeque::from([start]);
    distances[cell_index(world, start)] = Some(0);
    while let Some(cell) = pending.pop_front() {
        let distance = distances[cell_index(world, cell)].expect("queued cells are visited");
        for direction in 0..4 {
            let next = super::adjacent(cell, direction);
            let index = cell_index(world, next);
            if distances[index].is_none() && super::traversable(next, world, danger, tick) {
                distances[index] = Some(distance + 1);
                pending.push_back(next);
            }
        }
    }
    distances
}

fn cell_index(world: &World, cell: CellId) -> usize {
    usize::from(cell.row) * usize::from(world.arena.width()) + usize::from(cell.column)
}

pub(super) fn advance_mob(
    mob: &mut Mob,
    world: &World,
    danger: &BTreeSet<CellId>,
    searches: &[(usize, Distances)],
    frozen: bool,
    tick: Tick,
    policy: MobNavigationPolicy,
) -> MovementTrace {
    let mut trace = MovementTrace::new(mob.position);
    if frozen {
        mob.residual = 0;
        return trace;
    }
    let Some(cell) = mob.position.cell() else {
        return trace;
    };
    let center = Position::center(cell);
    if mob.retreat_anchor == Some(mob.position) {
        mob.retreat_anchor = None;
        mob.leg_origin = mob.position;
    } else if mob.retreat_anchor.is_none() && mob.position == center {
        mob.leg_origin = center;
    }
    let direction = choose_direction(mob, world, danger, searches, tick, cell, policy);
    let Some(direction) = direction else {
        mob.residual = 0;
        return trace;
    };
    let direction = directions()[usize::from(direction)];
    let numerator = mob.residual + super::MOB_UNITS_PER_SECOND;
    let mut distance = i32::from(numerator / TICKS_PER_SECOND);
    mob.residual = numerator % TICKS_PER_SECOND;
    if let Some(anchor) = mob.retreat_anchor {
        distance = distance.min(distance_to_target(mob.position, anchor, direction));
    }
    distance = allowed_distance(
        world,
        mob.position,
        direction,
        distance,
        &BTreeSet::new(),
        ObstacleAccess {
            can_pass: false,
            soft_grace: &BTreeSet::new(),
            oil_grace: &BTreeSet::new(),
        },
    );
    let start = mob.position;
    match direction {
        Direction::North => mob.position.y -= distance,
        Direction::East => mob.position.x += distance,
        Direction::South => mob.position.y += distance,
        Direction::West => mob.position.x -= distance,
    }
    mob.heading = direction;
    if mob.position != start {
        trace.push(MovementSegment::new(start, mob.position));
    }
    trace
}

fn choose_direction(
    mob: &mut Mob,
    world: &World,
    danger: &BTreeSet<CellId>,
    searches: &[(usize, Distances)],
    tick: Tick,
    cell: CellId,
    policy: MobNavigationPolicy,
) -> Option<u8> {
    if let Some(anchor) = mob.retreat_anchor {
        if policy == MobNavigationPolicy::LegacyV1
            || anchor.cell().is_some_and(|cell| !danger.contains(&cell))
        {
            return Some(direction_index(mob.heading));
        }
        let center = Position::center(cell);
        let (heading, anchor) =
            if super::traversable(cell, world, danger, tick) && mob.position != center {
                (super::direction_toward(mob.position, center), center)
            } else {
                let heading = opposite(mob.heading);
                let direction = direction_index(heading);
                let next = super::adjacent(cell, direction);
                if !super::traversable(next, world, danger, tick) {
                    return None;
                }
                (heading, Position::center(next))
            };
        mob.heading = heading;
        mob.retreat_anchor = Some(anchor);
        return Some(direction_index(heading));
    }
    if mob.position != mob.leg_origin
        && danger.contains(&cell)
        && (policy == MobNavigationPolicy::LegacyV1
            || mob
                .leg_origin
                .cell()
                .is_some_and(|origin| !danger.contains(&origin)))
    {
        mob.retreat_anchor = Some(mob.leg_origin);
        return Some(direction_index(opposite(mob.heading)));
    }
    if mob.position != Position::center(cell) {
        return Some(direction_index(mob.heading));
    }
    if danger.contains(&cell) {
        return (0..4)
            .find(|&dir| super::traversable(super::adjacent(cell, dir), world, danger, tick));
    }
    searches
        .iter()
        .filter_map(|(actor, distances)| {
            distances[cell_index(world, cell)].map(|distance| (distance, *actor, distances))
        })
        .min_by_key(|(distance, actor, _)| (*distance, *actor))
        .and_then(|(_, _, distances)| {
            let current = distances[cell_index(world, cell)]?;
            (0..4).find(|&dir| {
                let next = super::adjacent(cell, dir);
                distances[cell_index(world, next)] == current.checked_sub(1)
            })
        })
}

const fn directions() -> [Direction; 4] {
    [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ]
}

const fn direction_index(direction: Direction) -> u8 {
    match direction {
        Direction::North => 0,
        Direction::East => 1,
        Direction::South => 2,
        Direction::West => 3,
    }
}

const fn opposite(direction: Direction) -> Direction {
    match direction {
        Direction::North => Direction::South,
        Direction::East => Direction::West,
        Direction::South => Direction::North,
        Direction::West => Direction::East,
    }
}

fn distance_to_target(position: Position, target: Position, direction: Direction) -> i32 {
    match direction {
        Direction::North => position.y.saturating_sub(target.y),
        Direction::East => target.x.saturating_sub(position.x),
        Direction::South => target.y.saturating_sub(position.y),
        Direction::West => position.x.saturating_sub(target.x),
    }
    .max(0)
}

pub(crate) fn traces_contact_actor(traces: &[MovementTrace], actor: Position) -> bool {
    traces.iter().any(|trace| {
        trace
            .segments()
            .any(|segment| crate::game::stationary_contact_entry(segment, actor))
    })
}

pub(crate) fn actor_trace_contacts_mobs(trace: &MovementTrace, mobs: &[Mob]) -> bool {
    mobs.iter().any(|mob| {
        trace
            .segments()
            .any(|segment| crate::game::stationary_contact_entry(segment, mob.position))
    })
}

pub(crate) fn actor_segment_contacts_mobs(segment: MovementSegment, mobs: &[Mob]) -> bool {
    mobs.iter()
        .any(|mob| crate::game::stationary_sweep_contact(segment, mob.position))
}

pub(crate) fn actor_position_contacts_mobs(position: Position, mobs: &[Mob]) -> bool {
    mobs.iter().any(|mob| {
        (i64::from(position.x) - i64::from(mob.position.x)).abs()
            < i64::from(crate::game::types::HALF_FOOTPRINT) * 2
            && (i64::from(position.y) - i64::from(mob.position.y)).abs()
                < i64::from(crate::game::types::HALF_FOOTPRINT) * 2
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{EntityId, fixed_arena};
    use MobNavigationPolicy::{EscapeV2, LegacyV1};

    fn clear_world() -> World {
        let mut world = World::new(fixed_arena());
        let blocks = world.arena.blocks().clone();
        world.arena.remove_blocks(&blocks);
        world
    }

    fn mob(position: Position, heading: Direction, leg_origin: Position) -> Mob {
        Mob {
            id: EntityId(90),
            lair: 0,
            position,
            heading,
            residual: 36,
            leg_origin,
            retreat_anchor: None,
        }
    }

    fn advance(
        mob: &mut Mob,
        world: &World,
        danger: &BTreeSet<CellId>,
        tick: u64,
        policy: MobNavigationPolicy,
    ) {
        advance_mob(mob, world, danger, &[], false, Tick(tick), policy);
    }

    fn departed(origin: Position, heading: Direction) -> Position {
        let mut departed = origin;
        match heading {
            Direction::North => departed.y -= 25,
            Direction::East => departed.x += 25,
            Direction::South => departed.y += 25,
            Direction::West => departed.x -= 25,
        }
        departed
    }

    #[test]
    fn escape_navigation_completes_the_exit_from_a_dangerous_origin_on_every_axis() {
        let world = clear_world();
        let origin = Position::center(CellId::new(3, 3));
        let danger = BTreeSet::from([origin.cell().unwrap()]);
        for heading in directions() {
            let position = departed(origin, heading);
            let mut mob = mob(position, heading, origin);
            for tick in 2..=80 {
                advance(&mut mob, &world, &danger, tick, EscapeV2);
                if mob.position.cell() != origin.cell() {
                    break;
                }
            }
            assert_ne!(
                mob.position.cell(),
                origin.cell(),
                "failed heading {heading:?}"
            );
            assert_eq!(mob.heading, heading);
            assert_eq!(mob.retreat_anchor, None);
        }
    }

    #[test]
    fn legacy_navigation_retains_the_dangerous_origin_oscillation() {
        let world = clear_world();
        let origin = Position::center(CellId::new(3, 3));
        let danger = BTreeSet::from([origin.cell().unwrap()]);
        let departed = departed(origin, Direction::North);
        let mut mob = mob(departed, Direction::North, origin);

        advance(&mut mob, &world, &danger, 2, LegacyV1);
        assert_eq!(mob.position, origin);
        assert_eq!(mob.retreat_anchor, Some(origin));

        advance(&mut mob, &world, &danger, 3, LegacyV1);
        assert_eq!(mob.position, departed);
        assert_eq!(mob.heading, Direction::North);
    }

    #[test]
    fn danger_after_crossing_a_boundary_retreats_to_the_true_leg_origin_on_every_axis() {
        let world = clear_world();
        let cases = [
            (
                Position::center(CellId::new(3, 3)),
                Position { x: 4_200, y: 3_584 },
                Direction::East,
            ),
            (
                Position::center(CellId::new(5, 3)),
                Position { x: 5_000, y: 3_584 },
                Direction::West,
            ),
            (
                Position::center(CellId::new(3, 3)),
                Position { x: 3_584, y: 4_200 },
                Direction::South,
            ),
            (
                Position::center(CellId::new(3, 5)),
                Position { x: 3_584, y: 5_000 },
                Direction::North,
            ),
        ];
        for (origin, position, heading) in cases {
            let danger = BTreeSet::from([position.cell().unwrap()]);
            let clear = BTreeSet::new();
            let mut mob = mob(position, heading, origin);
            mob.residual = 0;
            let before = manhattan(mob.position, origin);
            advance(&mut mob, &world, &danger, 1, EscapeV2);
            assert!(manhattan(mob.position, origin) < before);
            assert_eq!(mob.retreat_anchor, Some(origin));

            for tick in 2..80 {
                advance(&mut mob, &world, &clear, tick, EscapeV2);
                if mob.position == origin {
                    break;
                }
            }
            assert_eq!(mob.position, origin);
            advance(&mut mob, &world, &clear, 80, EscapeV2);
            assert_eq!(mob.retreat_anchor, None);
        }
    }

    fn manhattan(left: Position, right: Position) -> u64 {
        i64::from(left.x).abs_diff(i64::from(right.x))
            + i64::from(left.y).abs_diff(i64::from(right.y))
    }
}

#[cfg(test)]
#[path = "lair_navigation_policy_tests.rs"]
mod policy_tests;

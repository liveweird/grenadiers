use std::collections::BTreeSet;

use super::{Actor, EntityId, MovementTrace, World};
use crate::game::simulation::contract::ForcedMovement;
use crate::game::{BombTrigger, movement};

pub(super) fn initialize_grace(world: &World, actors: &mut [Actor]) {
    for actor in actors {
        for bomb in world.bombs.values().filter(|bomb| {
            matches!(bomb.trigger(), BombTrigger::Contact)
                && movement::position_overlaps_cell(actor.position, bomb.cell())
        }) {
            actor.bomb_grace.insert(bomb.id());
        }
    }
}

pub(super) fn accepted_contact_ids(
    world: &World,
    traces: &[MovementTrace],
    forced_movement: Option<ForcedMovement>,
    alive_at_movement_start: impl AsRef<[bool]>,
    grace_at_movement_start: &[BTreeSet<EntityId>],
) -> BTreeSet<EntityId> {
    let alive_at_movement_start = alive_at_movement_start.as_ref();
    let mines: Vec<_> = world
        .bombs
        .values()
        .filter(|bomb| matches!(bomb.trigger(), BombTrigger::Contact))
        .copied()
        .collect();
    let mut accepted = BTreeSet::new();
    for index in 0..traces.len() {
        if !alive_at_movement_start[index]
            || (traces[index].is_empty()
                && forced_movement.is_none_or(|forced| forced.seat.index() != index))
        {
            continue;
        }
        for mine in &mines {
            let mut exempt = grace_at_movement_start[index].contains(&mine.id());
            for segment in traces[index].segments() {
                if exempt {
                    if !movement::position_overlaps_cell(segment.end(), mine.cell()) {
                        exempt = false;
                    }
                    continue;
                }
                if segment.overlaps_cell(mine.cell()) {
                    accepted.insert(mine.id());
                    break;
                }
            }
            if !accepted.contains(&mine.id())
                && let Some(forced) = forced_movement.filter(|forced| forced.seat.index() == index)
                && !exempt
                && forced.segment.overlaps_cell(mine.cell())
            {
                accepted.insert(mine.id());
            }
        }
    }
    accepted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Bomb, CellId, MovementSegment, Position, Tick, fixed_arena};

    fn world_with_mine() -> World {
        let mut world = World::new(fixed_arena());
        let mine = Bomb::new(
            EntityId(30),
            CellId::new(3, 1),
            EntityId(10),
            2,
            Tick(0),
            BombTrigger::Contact,
        )
        .unwrap();
        world.bombs.insert(mine.id(), mine);
        world
    }

    fn traces(one: MovementTrace) -> [MovementTrace; 2] {
        [
            one,
            MovementTrace::new(Position::center(CellId::new(13, 11))),
        ]
    }

    #[test]
    fn stationary_dead_and_boundary_tangent_are_not_new_contacts() {
        let world = world_with_mine();
        let center = Position::center(CellId::new(3, 1));
        assert!(
            accepted_contact_ids(
                &world,
                &traces(MovementTrace::new(center)),
                None,
                [true, true],
                &[BTreeSet::new(), BTreeSet::new()]
            )
            .is_empty()
        );

        let tangent = Position {
            x: 2_752,
            y: center.y,
        };
        let mut trace = MovementTrace::new(tangent);
        trace.push(MovementSegment::new(
            Position {
                x: 2_700,
                y: center.y,
            },
            tangent,
        ));
        assert!(
            accepted_contact_ids(
                &world,
                &traces(trace.clone()),
                None,
                [true, true],
                &[BTreeSet::new(), BTreeSet::new()]
            )
            .is_empty()
        );
        assert!(
            accepted_contact_ids(
                &world,
                &traces(trace),
                None,
                [false, true],
                &[BTreeSet::new(), BTreeSet::new()]
            )
            .is_empty()
        );
    }

    #[test]
    fn grace_ignores_exit_segment_but_a_later_reentry_segment_triggers() {
        let world = world_with_mine();
        let center = Position::center(CellId::new(3, 1));
        let outside = Position::center(CellId::new(2, 1));
        let mut exit_only = MovementTrace::new(center);
        exit_only.push(MovementSegment::new(center, outside));
        let grace = [BTreeSet::from([EntityId(30)]), BTreeSet::new()];
        assert!(
            accepted_contact_ids(&world, &traces(exit_only), None, [true, true], &grace).is_empty()
        );

        let mut exit_reenter = MovementTrace::new(center);
        exit_reenter.push(MovementSegment::new(center, outside));
        exit_reenter.push(MovementSegment::new(outside, center));
        assert_eq!(
            accepted_contact_ids(&world, &traces(exit_reenter), None, [true, true], &grace),
            BTreeSet::from([EntityId(30)])
        );
    }

    #[test]
    fn nongraced_positive_area_crossing_triggers() {
        let world = world_with_mine();
        let center = Position::center(CellId::new(3, 1));
        let outside = Position::center(CellId::new(2, 1));
        let mut trace = MovementTrace::new(outside);
        trace.push(MovementSegment::new(outside, center));
        assert_eq!(
            accepted_contact_ids(
                &world,
                &traces(trace),
                None,
                [true, true],
                &[BTreeSet::new(), BTreeSet::new()]
            ),
            BTreeSet::from([EntityId(30)])
        );
    }

    #[test]
    fn ordered_turn_segments_trigger_only_when_an_actual_segment_overlaps() {
        let world = world_with_mine();
        let mut entering = MovementTrace::new(Position { x: 2_700, y: 1_400 });
        entering.push(MovementSegment::new(
            Position { x: 2_700, y: 1_400 },
            Position { x: 2_700, y: 1_600 },
        ));
        entering.push(MovementSegment::new(
            Position { x: 2_700, y: 1_600 },
            Position { x: 2_800, y: 1_600 },
        ));
        assert_eq!(entering.len(), 2);
        assert_eq!(
            accepted_contact_ids(
                &world,
                &traces(entering),
                None,
                [true, true],
                &[BTreeSet::new(), BTreeSet::new()]
            ),
            BTreeSet::from([EntityId(30)])
        );

        let mut around_corner = MovementTrace::new(Position { x: 2_700, y: 700 });
        around_corner.push(MovementSegment::new(
            Position { x: 2_700, y: 700 },
            Position { x: 2_700, y: 2_400 },
        ));
        around_corner.push(MovementSegment::new(
            Position { x: 2_700, y: 2_400 },
            Position { x: 4_400, y: 2_400 },
        ));
        assert_eq!(around_corner.len(), 2);
        assert!(
            MovementSegment::new(
                Position { x: 2_700, y: 700 },
                Position { x: 4_400, y: 2_400 }
            )
            .overlaps_cell(CellId::new(3, 1))
        );
        assert!(
            accepted_contact_ids(
                &world,
                &traces(around_corner),
                None,
                [true, true],
                &[BTreeSet::new(), BTreeSet::new()]
            )
            .is_empty()
        );
    }
}

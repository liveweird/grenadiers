use super::*;
use crate::game::{EntityId, fixed_arena};

fn clear_world() -> World {
    let mut world = World::new(fixed_arena());
    let blocks = world.arena.blocks().clone();
    world.arena.remove_blocks(&blocks);
    world
}

fn retreating_mob(position: Position, origin: Position) -> Mob {
    Mob {
        id: EntityId(90),
        lair: 0,
        position,
        heading: Direction::West,
        residual: 0,
        leg_origin: origin,
        retreat_anchor: Some(origin),
    }
}

#[test]
fn rejected_mid_leg_escape_remains_pending_across_updates() {
    let world = clear_world();
    let origin = Position::center(CellId::new(3, 3));
    let position = Position {
        x: Position::center(CellId::new(4, 3)).x + 100,
        y: origin.y,
    };
    let danger = BTreeSet::from([
        CellId::new(3, 3),
        CellId::new(4, 2),
        CellId::new(4, 3),
        CellId::new(4, 4),
        CellId::new(5, 3),
    ]);
    let mut mob = retreating_mob(position, origin);

    for tick in 1..=20 {
        let mut budget = crate::game::mob_hazards::WorkBudget::new();
        let trace = advance_mob(
            &mut mob,
            &world,
            NavigationHazards::new(&danger, None),
            &[],
            false,
            NavigationParameters {
                tick: Tick(tick),
                policy: MobNavigationPolicy::EscapeV2,
            },
            &mut budget,
        );
        assert!(trace.segments().next().is_none());
        assert_eq!(mob.position, position);
        assert_eq!(mob.heading, Direction::West);
        assert_eq!(mob.retreat_anchor, Some(origin));
    }
}

#[test]
fn safe_current_center_on_either_side_clamps_then_resumes_perpendicularly() {
    let world = clear_world();
    let origin = Position::center(CellId::new(3, 3));
    let safe_center = Position::center(CellId::new(4, 3));
    let danger = BTreeSet::from([CellId::new(3, 3), CellId::new(5, 3)]);
    for offset in [-100, 100] {
        let position = Position {
            x: safe_center.x + offset,
            y: safe_center.y,
        };
        let mut mob = retreating_mob(position, origin);
        for tick in 1..=10 {
            let mut budget = crate::game::mob_hazards::WorkBudget::new();
            advance_mob(
                &mut mob,
                &world,
                NavigationHazards::new(&danger, None),
                &[],
                false,
                NavigationParameters {
                    tick: Tick(tick),
                    policy: MobNavigationPolicy::EscapeV2,
                },
                &mut budget,
            );
            if mob.position == safe_center {
                break;
            }
        }
        assert_eq!(mob.position, safe_center);
        assert_eq!(mob.retreat_anchor, Some(safe_center));

        let current = safe_center.cell().unwrap();
        let north = CellId::new(current.column, current.row - 1);
        let mut distances = vec![None; world.arena.cell_count()];
        distances[cell_index(&world, current)] = Some(1);
        distances[cell_index(&world, north)] = Some(0);
        let mut budget = crate::game::mob_hazards::WorkBudget::new();
        advance_mob(
            &mut mob,
            &world,
            NavigationHazards::new(&danger, None),
            &[(0, distances)],
            false,
            NavigationParameters {
                tick: Tick(11),
                policy: MobNavigationPolicy::EscapeV2,
            },
            &mut budget,
        );

        assert_eq!(mob.retreat_anchor, None);
        assert_eq!(mob.leg_origin, safe_center);
        assert_eq!(mob.heading, Direction::North);
        assert!(mob.position.y < safe_center.y);
    }
}

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
    let mut budget = crate::game::mob_hazards::WorkBudget::new();
    advance_mob(
        mob,
        world,
        NavigationHazards::new(danger, None),
        &[],
        false,
        NavigationParameters {
            tick: Tick(tick),
            policy,
        },
        &mut budget,
    );
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
    i64::from(left.x).abs_diff(i64::from(right.x)) + i64::from(left.y).abs_diff(i64::from(right.y))
}

use super::*;
use crate::game::{Arena, EntityId, MobNavigationPolicy, fixtures};

const ORIGIN: CellId = CellId::new(1, 7);

fn blocked_world() -> World {
    World::new(
        Arena::new(
            fixtures::classic_terrain(),
            BTreeSet::from([CellId::new(1, 6), CellId::new(2, 7)]),
            fixtures::classic_spawns(),
        )
        .unwrap(),
    )
}

fn mob() -> Mob {
    Mob {
        id: EntityId(90),
        lair: 0,
        position: Position::center(ORIGIN),
        heading: Direction::North,
        residual: 36,
        leg_origin: Position::center(ORIGIN),
        retreat_anchor: None,
    }
}

fn advance(mob: &mut Mob, world: &World, target: CellId, policy: MobNavigationPolicy) {
    let danger = BTreeSet::new();
    let searches = vec![(0, bfs(world, target, &danger, None, Tick(1)))];
    let mut budget = WorkBudget::new();
    advance_mob(
        mob,
        world,
        NavigationHazards::new(&danger, None),
        &searches,
        false,
        NavigationParameters {
            tick: Tick(1),
            policy,
        },
        &mut budget,
    );
}

#[test]
fn reached_target_v4_stays_centered_then_resumes_when_target_moves() {
    let world = blocked_world();
    let mut mob = mob();
    for _ in 0..30 {
        advance(
            &mut mob,
            &world,
            ORIGIN,
            MobNavigationPolicy::ReachedTargetV4,
        );
        assert_eq!(mob.position, Position::center(ORIGIN));
        assert_eq!(mob.residual, 0);
        assert_eq!(mob.retreat_anchor, None);
    }
    advance(
        &mut mob,
        &world,
        CellId::new(1, 9),
        MobNavigationPolicy::ReachedTargetV4,
    );
    assert!(mob.position.y > Position::center(ORIGIN).y);
    assert_eq!(mob.heading, Direction::South);
}

#[test]
fn archived_navigation_keeps_zero_distance_wall_entry_and_off_center_stall() {
    let world = blocked_world();
    for policy in [
        MobNavigationPolicy::LegacyV1,
        MobNavigationPolicy::EscapeV2,
        MobNavigationPolicy::TimedEscapeV3,
    ] {
        let mut mob = mob();
        for _ in 0..30 {
            advance(&mut mob, &world, ORIGIN, policy);
        }
        assert_eq!(mob.position, Position { x: 1536, y: 7488 });
        assert_eq!(mob.heading, Direction::North);
        assert_eq!(mob.leg_origin, Position::center(ORIGIN));
        let clipped = mob.position;
        for _ in 0..30 {
            advance(&mut mob, &world, CellId::new(1, 9), policy);
        }
        assert_eq!(mob.position, clipped);
    }
}

#[test]
fn reached_target_v4_keeps_positive_distance_routing_and_escape_capabilities() {
    let world = blocked_world();
    let mut previous = mob();
    let mut current = previous.clone();
    for _ in 0..15 {
        advance(
            &mut previous,
            &world,
            CellId::new(1, 9),
            MobNavigationPolicy::TimedEscapeV3,
        );
        advance(
            &mut current,
            &world,
            CellId::new(1, 9),
            MobNavigationPolicy::ReachedTargetV4,
        );
        assert_eq!(current.position, previous.position);
        assert_eq!(current.residual, previous.residual);
        assert_eq!(current.heading, previous.heading);
    }
    assert_eq!(MobNavigationPolicy::ReachedTargetV4.version(), 4);
    assert!(MobNavigationPolicy::ReachedTargetV4.has_escape_navigation());
    assert!(MobNavigationPolicy::ReachedTargetV4.has_timed_escape_navigation());
    assert!(!MobNavigationPolicy::EscapeV2.has_timed_escape_navigation());
}

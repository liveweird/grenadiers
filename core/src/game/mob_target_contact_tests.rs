use std::collections::BTreeSet;

use super::{
    ActorInit, Arena, CellId, Direction, EntityId, Equipment, MobNavigationPolicy, Position,
    RulesConfig, SeatId, SpeedLevel, State, StormCloudConfig, World, fixtures,
};
use crate::room::{RespawnLairConfig, RespawnLairSettings, RoomSettings};

#[test]
fn reached_target_stationary_mob_still_consumes_heart_then_deals_lethal_contact() {
    let origin = CellId::new(1, 7);
    let arena = Arena::new(
        fixtures::classic_terrain(),
        BTreeSet::from([CellId::new(1, 6), CellId::new(2, 7)]),
        fixtures::classic_spawns(),
    )
    .unwrap();
    let actors = SeatId::DUEL.map(|seat| ActorInit {
        id: EntityId(10 + seat.index() as u32),
        seat,
        position: Position::center(if seat == SeatId::One {
            origin
        } else {
            CellId::new(13, 11)
        }),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    });
    let rules = RulesConfig::environment_v9_with_mob_navigation(
        RoomSettings::DEFAULT,
        StormCloudConfig::Disabled,
        RespawnLairConfig::Enabled(RespawnLairSettings::new(60).unwrap()),
        MobNavigationPolicy::ReachedTargetV4,
    );
    let mut game = State::new(rules, World::new(arena), actors).unwrap();
    game.replace_test_actor_equipment(SeatId::One, Equipment::Heart);
    game.insert_test_mob(EntityId(90), Position::center(origin), Direction::North);

    game.step(&[]).unwrap();
    assert!(game.actors()[0].alive());
    assert_eq!(game.actors()[0].inventory().equipment(), None);
    assert_eq!(game.mobs()[0].position, Position::center(origin));
    for _ in 0..120 {
        if !game.actors()[0].alive() {
            break;
        }
        game.step(&[]).unwrap();
        assert_eq!(game.mobs()[0].position, Position::center(origin));
    }
    assert!(!game.actors()[0].alive());
    assert!(game.actors()[1].alive());
}

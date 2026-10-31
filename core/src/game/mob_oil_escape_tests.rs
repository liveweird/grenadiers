use std::collections::{BTreeMap, BTreeSet};

use super::{
    ActorInit, Arena, BombTrigger, CellId, Direction, EnabledItems, EntityId, MobNavigationPolicy,
    OilState, Position, RoundAssignment, RulesConfig, SeatId, SimulationSeed, SpeedLevel, State,
    StormCloudConfig, Tick, World,
};
use crate::room::{RespawnLairConfig, RespawnLairSettings, RoomSettings, SpikePitConfig};

const START: CellId = CellId::new(3, 3);
const OIL: CellId = CellId::new(9, 9);

fn game(oil_state: OilState, policy: MobNavigationPolicy) -> State {
    let mut blocks = BTreeSet::from([CellId::new(2, 3), CellId::new(4, 3)]);
    if oil_state == OilState::Concealed {
        blocks.insert(OIL);
    }
    let arena = Arena::new(
        super::fixtures::classic_terrain(),
        blocks,
        super::fixtures::classic_spawns(),
    )
    .unwrap();
    let spawns = arena.spawns().to_vec();
    let world = World::with_pickups_and_oil(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(OIL, oil_state)]),
    )
    .unwrap();
    let actors = SeatId::DUEL.map(|seat| ActorInit {
        id: EntityId(10 + seat.index() as u32),
        seat,
        position: if oil_state == OilState::Pending && seat == SeatId::Two {
            Position::center(OIL)
        } else {
            Position::center(spawns[seat.index()])
        },
        heading: Direction::East,
        speed: SpeedLevel::Base,
    });
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL);
    let rules = RulesConfig::environment_v10_with_mob_navigation(
        settings,
        StormCloudConfig::Disabled,
        RespawnLairConfig::Enabled(RespawnLairSettings::new(60).unwrap()),
        SpikePitConfig::Disabled,
        policy,
    );
    let mut game = State::new_at_tick_with_assignment_and_seed(
        rules,
        world,
        actors,
        Tick(0),
        RoundAssignment::first(),
        SimulationSeed::new([7; 32]),
    )
    .unwrap();
    game.insert_test_mob(EntityId(90), Position::center(START), Direction::South);
    game.insert_test_bomb(
        EntityId(91),
        CellId::new(3, 5),
        EntityId(10),
        3,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(100),
        },
    )
    .unwrap();
    game
}

#[test]
fn rules10_all_items_keeps_unrelated_uncertain_oil_while_v3_escapes() {
    for oil_state in [OilState::Concealed, OilState::Pending] {
        let mut nav2 = game(oil_state, MobNavigationPolicy::EscapeV2);
        let mut nav3 = game(oil_state, MobNavigationPolicy::TimedEscapeV3);

        for _ in 0..100 {
            nav2.step(&[]).unwrap();
            nav3.step(&[]).unwrap();
        }

        assert!(nav2.mobs().is_empty());
        assert_eq!(nav3.mobs().len(), 1);
        assert_eq!(nav2.world().oil.state(OIL), Some(oil_state));
        assert_eq!(nav3.world().oil.state(OIL), Some(oil_state));
    }
}

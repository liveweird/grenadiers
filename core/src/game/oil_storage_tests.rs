use std::collections::BTreeMap;

use crate::room::RoomSettings;

use crate::game::{
    ActorInit, Bomb, BombTrigger, CellId, Direction, EnabledItems, EntityId, OilBarrelState,
    OilState, PickupKind, PickupPlacementError, Position, RulesConfig, SeatId, SpeedLevel, State,
    StateError, Tick, World, fixed_arena,
};

fn actors() -> [ActorInit; 2] {
    [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(CellId::new(1, 1)),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        },
        ActorInit {
            id: EntityId(2),
            seat: SeatId::Two,
            position: Position::center(CellId::new(13, 11)),
            heading: Direction::West,
            speed: SpeedLevel::Base,
        },
    ]
}

fn rules(enabled: EnabledItems) -> RulesConfig {
    RulesConfig::expanded_v5(RoomSettings::DEFAULT.with_enabled_items(enabled))
}

#[test]
fn eight_lifetime_sites_can_mix_all_phases_and_public_view_sorts_materialized_only() {
    let sites = BTreeMap::from([
        (CellId::new(5, 1), OilState::Concealed),
        (CellId::new(6, 1), OilState::Concealed),
        (CellId::new(3, 1), OilState::Pending),
        (CellId::new(4, 1), OilState::Pending),
        (CellId::new(11, 1), OilState::Unlit),
        (CellId::new(10, 1), OilState::Unlit),
        (CellId::new(3, 3), OilState::Burning),
        (CellId::new(5, 3), OilState::Burning),
    ]);
    let world = World::with_pickups_and_oil(
        fixed_arena(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        sites,
    )
    .unwrap();
    assert_eq!(world.oil.iter().count(), 8);
    let state = State::new(rules(EnabledItems::OIL_BARREL), world, actors()).unwrap();
    let visible = state.public_view().oil_barrels;
    assert_eq!(
        visible.iter().map(|barrel| barrel.cell).collect::<Vec<_>>(),
        [
            CellId::new(10, 1),
            CellId::new(11, 1),
            CellId::new(3, 3),
            CellId::new(5, 3),
        ]
    );
    assert_eq!(visible[0].state, OilBarrelState::Unlit);
    assert_eq!(visible[2].state, OilBarrelState::Burning);

    let debug = format!("{state:?}");
    assert!(debug.contains("OilBarrels { sites: \"<redacted>\" }"));
    for private_phase in ["Concealed", "Pending", "Unlit", "Burning"] {
        assert!(!debug.contains(private_phase));
    }
}

#[test]
fn extraction_rejects_duplicate_and_ordinary_layer_conflicts() {
    let cell = CellId::new(5, 1);
    assert_eq!(
        World::with_pickups_and_oil(
            fixed_arena(),
            BTreeMap::from([(cell, PickupKind::OilBarrel)]),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(cell, OilState::Concealed)]),
        ),
        Err(PickupPlacementError::InvalidOilBarrel(cell))
    );

    let floor = CellId::new(3, 1);
    assert_eq!(
        World::with_pickups_and_oil(
            fixed_arena(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(floor, PickupKind::BlastRange)]),
            BTreeMap::from([(floor, OilState::Unlit)]),
        ),
        Err(PickupPlacementError::InvalidOilBarrel(floor))
    );
}

#[test]
fn state_rechecks_oil_enablement_and_settled_phase_conflicts() {
    let cell = CellId::new(3, 1);
    let make_world = |phase| {
        World::with_pickups_and_oil(
            fixed_arena(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(cell, phase)]),
        )
        .unwrap()
    };
    assert_eq!(
        State::new(
            rules(EnabledItems::EMPTY),
            make_world(OilState::Pending),
            actors()
        ),
        Err(StateError::UnsupportedOilBarrels)
    );

    let mut finite = make_world(OilState::Unlit);
    finite.hazards.insert(cell, Tick(10));
    assert_eq!(
        State::new(rules(EnabledItems::OIL_BARREL), finite, actors()),
        Err(StateError::InvalidOilBarrel(cell))
    );

    let mut occupied = make_world(OilState::Burning);
    occupied.bombs.insert(
        EntityId(9),
        Bomb::new(
            EntityId(9),
            cell,
            EntityId(1),
            1,
            Tick(0),
            BombTrigger::Timed {
                detonates_at: Tick(10),
            },
        )
        .unwrap(),
    );
    assert_eq!(
        State::new(rules(EnabledItems::OIL_BARREL), occupied, actors()),
        Err(StateError::InvalidOilBarrel(cell))
    );
}

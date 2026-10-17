use super::*;
use crate::game::{PickupKind, PickupView};
use std::collections::BTreeMap;

fn pickup(cell: CellId, kind: PickupKind) -> PickupView {
    PickupView { cell, kind }
}

#[test]
fn both_rules_versions_project_all_exposed_item_kinds() {
    for version in [1, 2] {
        let mut view = initial_view();
        view.rules_version = version;
        view.pickups = vec![
            pickup(CellId::new(2, 1), PickupKind::BombCapacity),
            pickup(CellId::new(3, 1), PickupKind::BlastRange),
            pickup(CellId::new(4, 1), PickupKind::MovementSpeed),
        ];
        let projected = project_state(&view).unwrap();
        assert_eq!(projected.rules_version.get(), version);
        assert_eq!(projected.state.pickups.len(), 3);
        assert!(schema_for("GameState").is_valid(&serde_json::to_value(projected.state).unwrap()));
    }
}

#[test]
fn pickup_projection_rejects_duplicate_unordered_blocked_burning_or_invalid_cells() {
    let mutations: &[fn(&mut crate::game::PublicState)] = &[
        |view| {
            let item = view.pickups[0];
            view.pickups.push(item);
        },
        |view| {
            view.pickups
                .push(pickup(CellId::new(4, 5), PickupKind::BlastRange))
        },
        |view| view.pickups = vec![view.pickups[0]; 196],
        |view| view.pickups[0].cell = CellId::new(0, 0),
        |view| view.pickups[0].cell = CellId::new(15, 5),
        |view| view.arena.blocks.push(CellId::new(5, 5)),
        |view| {
            view.flames.push(crate::game::FlameView {
                cell: CellId::new(5, 5),
                expires_at: Tick(10),
            })
        },
    ];
    for mutate in mutations {
        let mut view = initial_view();
        view.rules_version = 2;
        view.pickups = vec![pickup(CellId::new(5, 5), PickupKind::BombCapacity)];
        mutate(&mut view);
        assert_eq!(project_state(&view), Err(ProjectionError::InvalidPickups));
    }
}

#[test]
fn an_exposed_pickup_may_share_a_bomb_cell() {
    let target = CellId::new(5, 5);
    let world = World::with_pickups(
        open_world().arena().clone(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(target, PickupKind::BlastRange)]),
    )
    .unwrap();
    let mut state = State::new(
        RulesConfig::classic_v2(),
        world,
        [
            actor(10, SeatId::One, CellId::new(1, 1)),
            actor(20, SeatId::Two, CellId::new(13, 11)),
        ],
    )
    .unwrap();
    add_test_bomb_with_deadline(
        &mut state,
        EntityId(30),
        target,
        EntityId(10),
        2,
        Tick(0),
        Tick(180),
    )
    .unwrap();
    let projected = project_state(&state.public_view()).unwrap();
    assert_eq!(
        projected.state.pickups[0].cell,
        projected.state.bombs[0].cell
    );
}

#[test]
fn concealed_and_pending_contents_do_not_affect_wire_state() {
    let original = open_world();
    let terrain = std::array::from_fn(|index| {
        original
            .arena()
            .terrain(CellId::new((index % 15) as u16, (index / 15) as u16))
            .unwrap()
    });
    let arena = Arena::new(
        terrain,
        BTreeSet::from([CellId::new(5, 5)]),
        original.arena().spawns(),
    )
    .unwrap();
    let project = |kind| {
        let world = World::with_pickups(
            arena.clone(),
            BTreeMap::from([(CellId::new(5, 5), kind)]),
            BTreeMap::from([(CellId::new(6, 5), kind)]),
            BTreeMap::from([(CellId::new(7, 5), PickupKind::MovementSpeed)]),
        )
        .unwrap();
        let state = State::new(
            RulesConfig::classic_v2(),
            world,
            [
                actor(10, SeatId::One, CellId::new(1, 1)),
                actor(20, SeatId::Two, CellId::new(13, 11)),
            ],
        )
        .unwrap();
        project_state(&state.public_view()).unwrap().state
    };
    let first = project(PickupKind::BombCapacity);
    assert_eq!(first, project(PickupKind::BlastRange));
    assert_eq!(first.pickups.len(), 1);
    let json = serde_json::to_string(&first).unwrap();
    for forbidden in ["seed", "concealed", "pending", "fallback", "assignment"] {
        assert!(!json.contains(forbidden));
    }
}

#[test]
fn real_collection_precedes_its_same_tick_result_in_wire_events() {
    let target = CellId::new(5, 5);
    let dead = CellId::new(3, 5);
    let mut world = World::with_pickups(
        open_world().arena().clone(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(target, PickupKind::BlastRange)]),
    )
    .unwrap();
    world.hazards.insert(dead, Tick(10));
    let mut state = State::new(
        RulesConfig::classic_v2(),
        world,
        [actor(10, SeatId::One, dead), actor(20, SeatId::Two, target)],
    )
    .unwrap();
    let report = state.step(&[]).unwrap();
    let events = super::super::project_events(&[report]).unwrap();
    assert_eq!(
        events.iter().map(|event| event.index).collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert!(matches!(
        events[1].kind,
        super::super::EventKind::PickupCollected {
            seat: Seat::Two,
            ..
        }
    ));
    assert!(matches!(
        events[2].kind,
        super::super::EventKind::RoundEnded { .. }
    ));
    assert_eq!(
        project_state(&state.public_view()).unwrap().result,
        Some(WireRoundResult::Winner { seat: Seat::Two })
    );
    for event in events {
        assert!(schema_for("WireEvent").is_valid(&serde_json::to_value(event).unwrap()));
    }
}

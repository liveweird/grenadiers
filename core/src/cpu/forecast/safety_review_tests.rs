use std::collections::BTreeMap;

use crate::cpu::{BombKnowledge, Observation, ObservedOil, WorkBudget};
use crate::game::{
    ActorInit, BlastKind, Bomb, BombMotion, BombTrigger, CellId, Direction, EnabledItems, EntityId,
    OilBarrelState, OilState, Position, RulesConfig, SeatId, SpeedLevel, State, Tick, World,
};
use crate::room::RoomSettings;

use super::tests::{key, observation, open_arena, timed};
use super::{
    ForecastLimits, HazardKind, HypotheticalPlacement, ObservedHazardMemory, UncertaintyReason,
    forecast,
};

fn has_lethal_motion(
    forecast: &super::HazardForecast,
    cell: CellId,
    id: EntityId,
    tick: Tick,
) -> bool {
    forecast
        .cell(cell)
        .unwrap()
        .uncertainty
        .iter()
        .any(|hazard| {
            hazard.kind == HazardKind::Lethal
                && hazard.reason == UncertaintyReason::BombMotion(id)
                && hazard.window.contains(tick)
        })
}

fn run(
    observation: &Observation,
    memory: &ObservedHazardMemory,
    hypothetical: Option<HypotheticalPlacement>,
) -> super::HazardForecast {
    forecast(
        observation,
        memory,
        hypothetical,
        ForecastLimits::default(),
        &mut WorkBudget::new(65_536),
    )
    .unwrap()
}

#[test]
fn moving_into_burning_oil_can_detonate_before_the_nominal_fuse() {
    let id = EntityId(1);
    let mut parts = observation(
        0,
        BombKnowledge::Visible(vec![timed(
            id.0,
            CellId::new(4, 5),
            20,
            BlastKind::Standard,
        )]),
    )
    .into_parts();
    parts.oil.push(ObservedOil {
        cell: CellId::new(5, 5),
        state: OilBarrelState::Burning,
    });
    let observed = Observation::new(parts).unwrap();
    let result = run(&observed, &ObservedHazardMemory::new(key()), None);

    assert!(has_lethal_motion(&result, CellId::new(5, 8), id, Tick(1)));
}

#[test]
fn hypothetical_blast_can_chain_a_bomb_at_its_possible_moved_cell() {
    let id = EntityId(2);
    let observed = observation(
        0,
        BombKnowledge::Visible(vec![timed(
            id.0,
            CellId::new(4, 5),
            20,
            BlastKind::Standard,
        )]),
    );
    let result = run(
        &observed,
        &ObservedHazardMemory::new(key()),
        Some(HypotheticalPlacement {
            placed_at: Tick(7),
            owner: EntityId(2),
            cell: CellId::new(7, 5),
            range: 2,
            blast: BlastKind::Standard,
            detonates_at: Tick(10),
        }),
    );

    assert!(has_lethal_motion(&result, CellId::new(5, 8), id, Tick(10)));
}

#[test]
fn moved_chain_target_keeps_its_own_nonwaivable_provenance() {
    let a = EntityId(3);
    let b = EntityId(4);
    let previous = observation(
        1,
        BombKnowledge::Visible(vec![
            timed(a.0, CellId::new(3, 5), 11, BlastKind::Standard),
            timed(b.0, CellId::new(7, 5), 101, BlastKind::Standard),
        ]),
    );
    let current = observation(
        2,
        BombKnowledge::Visible(vec![
            timed(a.0, CellId::new(3, 5), 10, BlastKind::Standard),
            timed(b.0, CellId::new(6, 5), 100, BlastKind::Standard),
        ]),
    );
    let mut memory = ObservedHazardMemory::new(key());
    memory.observe(&previous).unwrap();
    memory.observe(&current).unwrap();
    let result = run(&current, &memory, None);

    assert!(result.stationary_assumptions().contains(&a));
    assert!(!result.stationary_assumptions().contains(&b));
    assert!(has_lethal_motion(&result, CellId::new(5, 8), b, Tick(12)));
}

#[test]
fn authoritative_motion_into_burning_oil_detonates_on_the_move_tick() {
    let oil = CellId::new(5, 5);
    let arena = open_arena();
    let spawns = arena.spawns().to_vec();
    let mut world = World::with_pickups_and_oil(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(oil, OilState::Burning)]),
    )
    .unwrap();
    let id = EntityId(50);
    let mut bomb = Bomb::new(
        id,
        CellId::new(4, 5),
        EntityId(10),
        3,
        Tick(0),
        BombTrigger::Timed {
            detonates_at: Tick(20),
        },
    )
    .unwrap();
    bomb.restore_kick_history(
        Some(BombMotion {
            direction: Direction::East,
            countdown: 0,
        }),
        true,
    );
    world.bombs.insert(id, bomb);
    let actors = [SeatId::One, SeatId::Two].map(|seat| ActorInit {
        id: EntityId(10 + seat.index() as u32),
        seat,
        position: Position::center(spawns[seat.index()]),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    });
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL);
    let mut state = State::new(RulesConfig::expanded_v5(settings), world, actors).unwrap();

    state.step(&[]).unwrap();
    assert!(state.world().has_fire(CellId::new(5, 8), Tick(1)));
    assert!(!state.world().bombs().contains_key(&id));
}

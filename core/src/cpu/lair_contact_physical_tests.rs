use std::collections::{BTreeMap, BTreeSet};
use std::ops::ControlFlow;

use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{
    StrategicIntent, StrategicObjectiveKind, StrategicPhase, StrategicSwitchReason,
    StrategicTarget, UtilityMemory,
};
use crate::cpu::routing::{
    PlantRequest, RouteFailure, RouteLimits, RouteStep, plan_lair_ordinary_plant,
};
use crate::cpu::survival::{
    ObservedBlastGeometry, WORK_ALLOWANCE, plan_farm_origins_v38_with_origin,
};
use crate::cpu::{CpuRng, MemoryKey, MobContactLease, MovementIntent, WorkBudget, WorldId};
use crate::game::blast_geometry::{BlastSource, visit_blast};
use crate::game::{
    ActorInit, Arena, BlastKind, CellId, Direction, EnabledItems, EntityId, InputAction,
    MobNavigationPolicy, Position, RoundAssignment, RulesConfig, SeatId, SeatInput, SimulationSeed,
    SpeedLevel, State, StormCloudConfig, Tick, World, fixed_arena,
};
use crate::room::{RespawnLairConfig, RespawnLairSettings, RoomSettings, SpikePitConfig};

use super::connectivity_tests::{IsolationEvidence, captured_blocks};

const CPU: SeatId = SeatId::Two;
const ORIGIN: CellId = CellId::new(10, 7);
const TARGET: CellId = CellId::new(9, 7);
const MOB_POCKET: CellId = CellId::new(7, 6);

fn controlled_state() -> (State, RoomSettings) {
    let fixed = fixed_arena();
    let terrain = std::array::from_fn(|index| {
        fixed
            .terrain(CellId::new((index % 15) as u16, (index / 15) as u16))
            .unwrap()
    });
    let arena = Arena::new(
        terrain,
        captured_blocks(),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap();
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL);
    let rules = RulesConfig::environment_v10_with_mob_navigation(
        settings,
        StormCloudConfig::Disabled,
        RespawnLairConfig::Enabled(RespawnLairSettings::DEFAULT),
        SpikePitConfig::Disabled,
        MobNavigationPolicy::TimedEscapeV3,
    );
    let state = State::new_at_tick_with_assignment_and_seed(
        rules,
        World::with_pickups(arena, BTreeMap::new(), BTreeMap::new(), BTreeMap::new()).unwrap(),
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
                seat: CPU,
                position: Position::center(ORIGIN),
                heading: Direction::East,
                speed: SpeedLevel::Base,
            },
        ],
        Tick(0),
        RoundAssignment::first(),
        SimulationSeed::new([136; 32]),
    )
    .unwrap();
    (state, settings)
}

fn memory(observation: &crate::cpu::Observation) -> UtilityMemory {
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let tick = observation.parts().tick;
    UtilityMemory::new(key).with_strategic_intent(StrategicIntent {
        objective: StrategicObjectiveKind::Farm,
        target: StrategicTarget::Block { cell: TARGET },
        approach_cell: Some(ORIGIN),
        phase: StrategicPhase::Approach,
        last_switch_reason: StrategicSwitchReason::Stalled,
        observation_tick: tick,
        score: 2,
        committed_until: Tick(tick.0 + 120),
        progress_deadline: Tick(tick.0 + 180),
        last_distance: 0,
        last_cpu_position: Position::center(ORIGIN),
        deferred_until: None,
    })
}

fn input(movement: Option<Direction>, bomb: bool) -> Vec<SeatInput> {
    let mut inputs = vec![SeatInput {
        seat: CPU,
        action: movement.map_or(InputAction::Clear, InputAction::Press),
    }];
    if bomb {
        inputs.push(SeatInput {
            seat: CPU,
            action: InputAction::PlaceBomb,
        });
    }
    inputs
}

#[test]
fn natural_trapped_mob_has_a_full_plant_route_but_old_contact_rejects_it() {
    let (mut state, settings) = controlled_state();
    for _ in 0..RespawnLairSettings::DEFAULT.spawn_ticks() {
        state.step(&[]).unwrap();
    }
    assert!(!state.mobs().is_empty());
    assert!(
        state
            .mobs()
            .iter()
            .all(|mob| mob.position.cell() == Some(MOB_POCKET))
    );
    let observed =
        crate::cpu::survival::observe_with_lairs(&state, (settings, RespawnLairSettings::DEFAULT));
    assert_eq!(
        observed
            .parts()
            .arena
            .blocks
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        captured_blocks()
    );
    assert!(observed.parts().pickups.is_empty());
    assert_eq!(observed.parts().tick, Tick(600));
    let key = MemoryKey {
        world: WorldId {
            room_instance: 27,
            physical_round: 1,
        },
        cpu_seat: CPU,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observed).unwrap();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let certificate = plan_lair_ordinary_plant(
        &observed,
        &history,
        PlantRequest {
            execution_tick: Tick(607),
            settled_stop_tick: Tick(600),
            first_movement: MovementIntent::Direction(Direction::East),
            limits: RouteLimits::default(),
        },
        &mut work,
    )
    .expect("unchanged route proof should certify the controlled east escape");
    assert_eq!(
        MobContactLease::certify_route(&observed, certificate.route(), &mut work),
        Err(RouteFailure::NoRoute)
    );

    let mut planner_work = WorkBudget::new(WORK_ALLOWANCE);
    let full = plan_farm_origins_v38_with_origin(
        &observed,
        &memory(&observed),
        &history,
        observed.parts().tick,
        None,
        &mut CpuRng::from_seed(key, [38; 32]),
        &mut planner_work,
    )
    .unwrap();
    assert!(full.survival.execution.is_none());

    let mut geometry_work = WorkBudget::new(WORK_ALLOWANCE);
    let geometry = ObservedBlastGeometry::new(&observed, &mut geometry_work).unwrap();
    let mut opened = BTreeSet::new();
    let _ = visit_blast(
        &geometry,
        BlastSource {
            cell: certificate.placement().cell,
            range: certificate.placement().range,
            kind: BlastKind::Standard,
        },
        |visit| {
            if geometry_work.try_take(1).is_err() {
                return ControlFlow::Break(());
            }
            if visit.soft_block {
                opened.insert(visit.cell);
            }
            ControlFlow::Continue(())
        },
    );
    assert_eq!(opened, BTreeSet::from([TARGET]));
    let sources = observed
        .respawn_lairs()
        .unwrap()
        .mobs()
        .iter()
        .filter_map(|mob| mob.position.cell())
        .collect::<Vec<_>>();
    let evidence = IsolationEvidence::build(
        &observed.parts().arena.blocks.iter().copied().collect(),
        &opened,
        &sources,
        &mut geometry_work,
    )
    .unwrap();
    let mut queried_snapshots = 0_u64;
    for first in
        certificate.route().start_window().start.0..certificate.route().start_window().end.0
    {
        let lane = certificate.route().lane(Tick(first)).unwrap();
        let mut previous = lane.pose_at(observed.parts().tick).unwrap().0;
        for tick in observed.parts().tick.0 + 1..certificate.route().survive_until().0 {
            let current = lane.pose_at(Tick(tick)).unwrap().0;
            assert!(evidence.excludes_segment(
                crate::game::MovementSegment::new(previous, current),
                &mut geometry_work,
            ));
            queried_snapshots += 1;
            previous = current;
        }
    }
    let lane_count =
        certificate.route().start_window().end.0 - certificate.route().start_window().start.0;
    assert_eq!(
        queried_snapshots,
        lane_count * (certificate.route().survive_until().0 - observed.parts().tick.0 - 1)
    );
    assert!(work.spent() <= WORK_ALLOWANCE);
    assert!(geometry_work.spent() <= WORK_ALLOWANCE);
    eprintln!(
        "contact topology witness: placement={} survive={} route_work={} planner_work={} geometry_work={}",
        certificate.placement().placed_at.0,
        certificate.route().survive_until().0,
        work.spent(),
        planner_work.spent(),
        geometry_work.spent(),
    );

    let first = certificate.route().start_window().start;
    let lane = certificate.route().lane(first).unwrap();
    while state.tick().0 + 1 < first.0 {
        state.step(&[]).unwrap();
    }
    while state.tick() < certificate.route().survive_until() {
        let update = Tick(state.tick().0 + 1);
        let movement = certificate
            .route()
            .steps()
            .iter()
            .enumerate()
            .find_map(|(index, step)| {
                let start = lane.step_start(index)?;
                let stop = lane.stop_update(index)?;
                (start <= update && update < stop).then_some(match step {
                    RouteStep::Move { direction, .. } => Some(*direction),
                    RouteStep::Wait { .. } => None,
                })
            })
            .flatten();
        state
            .step(&input(
                movement,
                update == certificate.placement().placed_at,
            ))
            .unwrap();
        assert!(state.actor(CPU).unwrap().alive());
    }
    assert!(!state.world().arena.blocks().contains(&TARGET));
    assert_eq!(state.live_bomb_count(CPU), 0);
    assert!(state.world().hazards().is_empty());
    assert!(
        state
            .mobs()
            .iter()
            .all(|mob| mob.position.cell() == Some(MOB_POCKET))
    );
    for barrier in [
        CellId::new(7, 5),
        CellId::new(7, 7),
        CellId::new(8, 5),
        CellId::new(8, 7),
    ] {
        assert!(state.world().arena.blocks().contains(&barrier));
    }
}

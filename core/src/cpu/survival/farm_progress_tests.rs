use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{
    StrategicIntent, StrategicObjectiveKind, StrategicPhase, StrategicSwitchReason,
    StrategicTarget, UtilityMemory,
};
use crate::cpu::{
    BombKnowledge, CpuRng, MemoryKey, MovementIntent, Observation, ObservationParts, ObservedActor,
    ObservedArena, ObservedEnvironment, ObservedRespawnLairs, ObservedRules, ObservedSpikePits,
    UnknownActorFacts, VisibleActor, WorkBudget, WorldId,
};
use crate::game::{
    CellId, Direction, EnabledItems, EntityId, LairStatusView, Position, SeatId, SpeedLevel,
    SpikePitPhase, SpikePitView, StormCloudConfig, StormCloudView, Tick, fixed_arena,
};
use crate::room::{
    RespawnLairConfig, RespawnLairSettings, RoomSettings, SpikePitConfig, SpikePitSettings,
};

use super::{
    WORK_ALLOWANCE, plan_refuge_guidance_v33_with_origin, plan_refuge_guidance_v34_with_origin,
};

const CPU: SeatId = SeatId::Three;
const TARGET: CellId = CellId::new(3, 10);
const APPROACH: CellId = CellId::new(3, 9);

fn actor(
    id: u32,
    seat: SeatId,
    position: Position,
    heading: Direction,
    alive: bool,
) -> ObservedActor {
    ObservedActor::Visible(VisibleActor {
        id: EntityId(id),
        seat,
        position,
        heading,
        speed: SpeedLevel::Base,
        alive,
        bomb_capacity: 1,
        bomb_range: 2,
        live_bomb_count: 0,
        equipment: None,
        weapon: None,
        effects: Vec::new(),
        damage_grace_remaining_ticks: 0,
        voluntary_actions_frozen: false,
        last_teleport_tick: None,
        unknown: UnknownActorFacts::ALL,
    })
}

fn captured_tick_7022() -> Observation {
    let fixed = fixed_arena();
    let terrain = (0..195)
        .map(|index| fixed.terrain(CellId::new(index % 15, index / 15)).unwrap())
        .collect();
    let blocks = [
        (3, 1),
        (5, 1),
        (3, 2),
        (5, 2),
        (7, 2),
        (5, 3),
        (6, 3),
        (7, 3),
        (8, 3),
        (1, 4),
        (5, 4),
        (9, 4),
        (2, 5),
        (3, 5),
        (4, 5),
        (6, 5),
        (7, 5),
        (8, 5),
        (10, 5),
        (3, 6),
        (5, 6),
        (7, 6),
        (9, 6),
        (11, 6),
        (2, 7),
        (3, 7),
        (4, 7),
        (6, 7),
        (7, 7),
        (8, 7),
        (10, 7),
        (11, 7),
        (12, 7),
        (5, 8),
        (9, 8),
        (5, 9),
        (6, 9),
        (7, 9),
        (8, 9),
        (9, 9),
        (3, 10),
        (5, 10),
        (7, 10),
        (9, 10),
        (11, 10),
        (5, 11),
        (9, 11),
    ]
    .into_iter()
    .map(|(column, row)| CellId::new(column, row))
    .collect();
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL);
    let parts = ObservationParts {
        world: WorldId {
            room_instance: 133,
            physical_round: 1,
        },
        tick: Tick(7_022),
        recipient: CPU,
        rules: ObservedRules {
            version: 10,
            tick_rate: 60,
            running_updates: 6_828,
            remaining_ticks: 3_972,
            settings,
        },
        arena: ObservedArena {
            width: 15,
            height: 13,
            terrain,
            blocks,
        },
        actors: vec![
            actor(
                1,
                SeatId::One,
                Position { x: 1_536, y: 1_536 },
                Direction::East,
                true,
            ),
            actor(
                2,
                SeatId::Two,
                Position { x: 7_717, y: 1_536 },
                Direction::East,
                false,
            ),
            actor(
                3,
                CPU,
                Position { x: 3_584, y: 9_434 },
                Direction::North,
                true,
            ),
            actor(
                4,
                SeatId::Four,
                Position { x: 7_078, y: 1_536 },
                Direction::East,
                true,
            ),
        ],
        bombs: BombKnowledge::Visible(Vec::new()),
        flames: Vec::new(),
        oil: Vec::new(),
        pickups: Vec::new(),
    };
    Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(
            StormCloudConfig::Single,
            Some(StormCloudView {
                position: Position { x: 1_888, y: 9_728 },
                direction: Direction::East,
                leg_remaining_ticks: 84,
            }),
            vec![Some(0); 4],
        ),
        Some(ObservedRespawnLairs::new(
            RespawnLairConfig::Enabled(RespawnLairSettings::DEFAULT),
            [CellId::new(6, 6), CellId::new(8, 6)]
                .into_iter()
                .map(|cell| LairStatusView {
                    cell,
                    next_spawn_at: Some(Tick(7_022)),
                    living_mobs: 0,
                })
                .collect(),
            Vec::new(),
        )),
        Some(ObservedSpikePits::new(
            SpikePitConfig::Enabled(SpikePitSettings::DEFAULT),
            [(5, 5), (9, 5), (5, 7), (9, 7)]
                .into_iter()
                .map(|(column, row)| SpikePitView {
                    cell: CellId::new(column, row),
                    phase: SpikePitPhase::Idle,
                    remaining_ticks: 0,
                })
                .collect(),
        )),
    )
    .unwrap()
}

fn key(observation: &Observation) -> MemoryKey {
    MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    }
}

fn stalled_memory(observation: &Observation) -> UtilityMemory {
    UtilityMemory::new(key(observation)).with_strategic_intent(StrategicIntent {
        objective: StrategicObjectiveKind::Farm,
        target: StrategicTarget::Block { cell: TARGET },
        approach_cell: Some(APPROACH),
        phase: StrategicPhase::Approach,
        last_switch_reason: StrategicSwitchReason::Stalled,
        observation_tick: Tick(7_016),
        score: 2,
        committed_until: Tick(7_136),
        progress_deadline: Tick(7_196),
        last_distance: 0,
        last_cpu_position: Position { x: 3_584, y: 9_434 },
        deferred_until: Some(Tick(7_136)),
    })
}

fn plan(observation: &Observation, hard: bool) -> super::LairSurvivalPlan {
    let key = key(observation);
    let mut history = ObservedHazardMemory::new(key);
    history.observe(observation).unwrap();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planner = if hard {
        plan_refuge_guidance_v34_with_origin
    } else {
        plan_refuge_guidance_v33_with_origin
    };
    let planned = planner(
        observation,
        &stalled_memory(observation),
        &history,
        observation.parts().tick,
        None,
        &mut CpuRng::from_seed(key, [133; 32]),
        &mut work,
    )
    .unwrap();
    assert!(work.spent() <= WORK_ALLOWANCE);
    planned
}

#[test]
fn captured_farm_approach_declines_when_the_full_lair_route_exhausts_work() {
    let observed = captured_tick_7022();
    for hard in [false, true] {
        let plan = plan(&observed, hard);
        assert_eq!(plan.survival.control.intent.manual, None);
        assert!(plan.survival.execution.is_none());
    }

    let centered = observed
        .map_parts(|parts| {
            let ObservedActor::Visible(cpu) = &mut parts.actors[CPU.index()] else {
                unreachable!()
            };
            cpu.position = Position::center(APPROACH);
        })
        .unwrap();
    let key = key(&centered);
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&centered).unwrap();
    for movement in [
        MovementIntent::Stop,
        MovementIntent::Direction(Direction::North),
        MovementIntent::Direction(Direction::East),
        MovementIntent::Direction(Direction::South),
        MovementIntent::Direction(Direction::West),
    ] {
        let direct = crate::cpu::routing::plan_lair_ordinary_plant(
            &centered,
            &history,
            crate::cpu::routing::PlantRequest {
                execution_tick: Tick(7_029),
                settled_stop_tick: Tick(7_022),
                first_movement: movement,
                limits: crate::cpu::routing::RouteLimits::default(),
            },
            &mut WorkBudget::new(WORK_ALLOWANCE),
        );
        assert_eq!(
            direct,
            Err(crate::cpu::routing::PlantFailure::Route(
                crate::cpu::routing::RouteFailure::WorkExhausted
            )),
            "{movement:?}"
        );
    }
}

fn controlled_plan(observation: &Observation) -> super::LairSurvivalPlan {
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: observation.parts().recipient,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(observation).unwrap();
    plan_refuge_guidance_v33_with_origin(
        observation,
        &UtilityMemory::new(key),
        &history,
        observation.parts().tick,
        None,
        &mut CpuRng::from_seed(key, [136; 32]),
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap()
}

#[test]
fn controlled_state_proves_centering_is_the_only_missing_plant_precondition() {
    const CONTROL_CPU: SeatId = SeatId::Two;
    const CONTROL_APPROACH: CellId = CellId::new(7, 7);
    const CONTROL_TARGET: CellId = CellId::new(8, 7);
    for lairs in [false, true] {
        let (mut off_center, settings) =
            super::reliable_v27_oil_tests::state(lairs, None, false, false);
        let center = Position::center(CONTROL_APPROACH);
        off_center.set_test_actor_pose(
            CONTROL_CPU,
            Position {
                x: center.x,
                y: center.y - 294,
            },
            Direction::North,
        );
        let observed = super::reliable_v27_oil_tests::observe(&off_center, settings);
        let declined = controlled_plan(&observed);
        assert_eq!(
            declined.survival.control.intent.manual, None,
            "lairs={lairs}"
        );
        assert!(declined.survival.execution.is_none(), "lairs={lairs}");

        let (mut centered, settings) =
            super::reliable_v27_oil_tests::state(lairs, None, false, false);
        let basis = super::reliable_v27_oil_tests::observe(&centered, settings);
        assert!(basis.parts().arena.blocks.contains(&CONTROL_TARGET));
        let productive = controlled_plan(&basis);
        assert_eq!(
            productive.survival.control.intent.manual,
            Some(crate::cpu::ManualIntent::Bomb),
            "lairs={lairs}"
        );
        assert_eq!(productive.contact.is_some(), lairs);
        super::reliable_v27_oil_tests::execute(&mut centered, settings, basis, productive);
        assert!(centered.actor(CONTROL_CPU).unwrap().alive());
        assert!(!centered.world().arena().blocks().contains(&CONTROL_TARGET));
    }
}

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
    ExecutionCertificate, WORK_ALLOWANCE, plan_farm_refuges_v36_with_origin,
    plan_hard_farm_refuges_v37_with_origin, plan_refuge_guidance_v33_with_origin,
    plan_refuge_guidance_v34_with_origin,
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

pub(super) fn captured_tick_7022() -> Observation {
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

pub(super) fn key(observation: &Observation) -> MemoryKey {
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

#[test]
fn legal_open_corridor_off_center_already_certifies_without_centering() {
    let captured = captured_tick_7022();
    let mut parts = captured.into_parts();
    parts.arena.blocks.retain(|cell| *cell == TARGET);
    let open = Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0); 4]),
        None,
        None,
    )
    .unwrap();
    let planned = plan(&open, false);
    assert_eq!(
        planned.survival.control.intent.manual,
        Some(crate::cpu::ManualIntent::Bomb)
    );
    assert!(planned.survival.execution.is_some());
}

pub(super) fn north_farm_memory(observation: &Observation) -> UtilityMemory {
    let approach = CellId::new(3, 8);
    UtilityMemory::new(key(observation)).with_strategic_intent(StrategicIntent {
        objective: StrategicObjectiveKind::Farm,
        target: StrategicTarget::Block {
            cell: CellId::new(3, 7),
        },
        approach_cell: Some(approach),
        phase: StrategicPhase::Approach,
        last_switch_reason: StrategicSwitchReason::Stalled,
        observation_tick: observation.parts().tick,
        score: 2,
        committed_until: Tick(observation.parts().tick.0 + 120),
        progress_deadline: Tick(observation.parts().tick.0 + 180),
        last_distance: 0,
        last_cpu_position: Position::center(approach),
        deferred_until: None,
    })
}

#[test]
fn joint_refuge_planner_automatically_finds_the_western_full_contact_route() {
    let observed = captured_tick_7022()
        .map_parts(|parts| {
            let ObservedActor::Visible(cpu) = &mut parts.actors[CPU.index()] else {
                unreachable!()
            };
            cpu.position = Position::center(CellId::new(3, 8));
            cpu.heading = Direction::North;
        })
        .unwrap();
    let memory = north_farm_memory(&observed);
    let key = key(&observed);
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observed).unwrap();

    for hard in [false, true] {
        let mut work = WorkBudget::new(WORK_ALLOWANCE);
        let mut rng = CpuRng::from_seed(key, [136 + u8::from(hard); 32]);
        let planned = if hard {
            plan_hard_farm_refuges_v37_with_origin(
                &observed,
                &memory,
                &history,
                observed.parts().tick,
                None,
                &mut rng,
                &mut work,
            )
        } else {
            plan_farm_refuges_v36_with_origin(
                &observed,
                &memory,
                &history,
                observed.parts().tick,
                None,
                &mut rng,
                &mut work,
            )
        }
        .unwrap();
        assert_eq!(
            planned.survival.control.intent.manual,
            Some(crate::cpu::ManualIntent::Bomb)
        );
        let Some(ExecutionCertificate::Plant(certificate)) = planned.survival.execution else {
            panic!("v{} did not return a plant", if hard { 37 } else { 36 });
        };
        assert_eq!(certificate.placement().cell, CellId::new(3, 8));
        assert_eq!(certificate.route().steps().len(), 8);
        assert_eq!(certificate.route().survive_until(), Tick(7_239));
        assert!(planned.contact.is_some());
        assert!(work.spent() <= WORK_ALLOWANCE);
    }
}

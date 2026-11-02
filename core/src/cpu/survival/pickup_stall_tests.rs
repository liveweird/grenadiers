use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::policy::{StrategicObjectiveKind, StrategicTarget, UtilityMemory};
use crate::cpu::routing::{PulseRequest, RouteMode, certify_pickup_pulse};
use crate::cpu::{
    BombKnowledge, CpuRng, MemoryKey, MovementIntent, Observation, ObservationParts, ObservedActor,
    ObservedArena, ObservedEnvironment, ObservedFlame, ObservedPickup, ObservedRespawnLairs,
    ObservedRules, ObservedSpikePits, UnknownActorFacts, VisibleActor, WorkBudget, WorldId,
};
use crate::game::{
    CellId, Direction, EnabledItems, EntityId, LairStatusView, MobView, PickupKind, Position,
    SeatId, SpeedLevel, SpikePitPhase, SpikePitView, StormCloudConfig, StormCloudView, Terrain,
    Tick, fixed_arena,
};
use crate::room::{RespawnLairSettings, RoomSettings, SpikePitSettings};

use super::{
    WORK_ALLOWANCE, plan_counter_pressure_v29_with_origin,
    plan_hard_pickup_progress_v32_with_origin, plan_pickup_progress_v31_with_origin,
    plan_reliable_v27_with_origin,
};

const CPU: SeatId = SeatId::Two;
const CAPACITY: CellId = CellId::new(8, 1);

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

fn captured_tick_1800(blocking_contact: bool) -> Observation {
    let fixed = fixed_arena();
    let terrain = (0..195)
        .map(|index| {
            fixed
                .terrain(CellId::new(index % 15, index / 15))
                .unwrap_or(Terrain::Wall)
        })
        .collect();
    let blocks = [
        (3, 1),
        (6, 1),
        (7, 1),
        (3, 2),
        (3, 3),
        (4, 3),
        (7, 3),
        (10, 3),
        (3, 4),
        (5, 4),
        (7, 4),
        (9, 4),
        (11, 4),
        (1, 5),
        (2, 5),
        (3, 5),
        (6, 5),
        (8, 5),
        (11, 5),
        (12, 5),
        (3, 6),
        (11, 6),
        (2, 7),
        (3, 7),
        (6, 7),
        (8, 7),
        (11, 7),
        (12, 7),
        (3, 8),
        (5, 8),
        (7, 8),
        (9, 8),
        (11, 8),
        (3, 9),
        (4, 9),
        (7, 9),
        (10, 9),
        (11, 9),
        (3, 10),
        (11, 10),
        (6, 11),
        (7, 11),
        (8, 11),
    ]
    .into_iter()
    .map(|(column, row)| CellId::new(column, row))
    .collect();
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL);
    let parts = ObservationParts {
        world: WorldId {
            room_instance: 131,
            physical_round: 1,
        },
        tick: Tick(1_800),
        recipient: CPU,
        rules: ObservedRules {
            version: 10,
            punch_activation_policy: crate::game::PunchActivationPolicy::ManualV1,
            tick_rate: 60,
            running_updates: 1_599,
            remaining_ticks: 9_201,
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
                CPU,
                Position { x: 9_668, y: 1_536 },
                Direction::West,
                true,
            ),
            actor(
                3,
                SeatId::Three,
                Position { x: 2_050, y: 9_728 },
                Direction::East,
                false,
            ),
            actor(
                4,
                SeatId::Four,
                Position {
                    x: 13_310,
                    y: 9_728,
                },
                Direction::West,
                false,
            ),
        ],
        bombs: BombKnowledge::Visible(Vec::new()),
        flames: Vec::new(),
        oil: Vec::new(),
        pickups: vec![
            ObservedPickup {
                cell: CAPACITY,
                kind: PickupKind::BombCapacity,
            },
            ObservedPickup {
                cell: CellId::new(11, 2),
                kind: PickupKind::BlastRange,
            },
        ],
    };
    let environment = ObservedEnvironment::new(
        StormCloudConfig::Single,
        Some(StormCloudView {
            position: Position { x: 6_656, y: 5_128 },
            direction: Direction::North,
            leg_remaining_ticks: 65,
        }),
        vec![Some(0); 4],
    );
    let lairs = ObservedRespawnLairs::new(
        crate::room::RespawnLairConfig::Enabled(RespawnLairSettings::DEFAULT),
        vec![
            LairStatusView {
                cell: CellId::new(6, 6),
                next_spawn_at: None,
                living_mobs: 1,
            },
            LairStatusView {
                cell: CellId::new(8, 6),
                next_spawn_at: None,
                living_mobs: 1,
            },
        ],
        vec![
            MobView {
                id: EntityId(11),
                lair: 0,
                position: if blocking_contact {
                    Position { x: 9_000, y: 1_536 }
                } else {
                    Position { x: 7_680, y: 6_656 }
                },
                heading: Direction::South,
                frozen: false,
            },
            MobView {
                id: EntityId(12),
                lair: 1,
                position: Position { x: 9_728, y: 6_656 },
                heading: Direction::South,
                frozen: false,
            },
        ],
    );
    let pits = ObservedSpikePits::new(
        crate::room::SpikePitConfig::Enabled(SpikePitSettings::DEFAULT),
        [(5, 5), (9, 5), (5, 7), (9, 7)]
            .into_iter()
            .map(|(column, row)| SpikePitView {
                cell: CellId::new(column, row),
                phase: SpikePitPhase::Idle,
                remaining_ticks: 0,
            })
            .collect(),
    );
    Observation::new_rules_v10(parts, environment, Some(lairs), Some(pits)).unwrap()
}

fn key(observation: &Observation) -> MemoryKey {
    MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    }
}

#[test]
fn captured_hard29_stops_before_a_safe_capacity_pickup_pulse() {
    let observed = captured_tick_1800(false);
    let key = key(&observed);
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observed).unwrap();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let plan = plan_counter_pressure_v29_with_origin(
        &observed,
        &UtilityMemory::new(key),
        &history,
        observed.parts().tick,
        None,
        &mut CpuRng::from_seed(key, [131; 32]),
        &mut work,
    )
    .unwrap();
    let intent = plan
        .survival
        .control
        .next_memory
        .strategic_intent()
        .unwrap();
    assert_eq!(intent.objective, StrategicObjectiveKind::Collect);
    assert_eq!(
        intent.target,
        StrategicTarget::Pickup {
            cell: CAPACITY,
            kind: PickupKind::BombCapacity,
        }
    );
    assert_eq!(plan.survival.control.intent.movement, MovementIntent::Stop);

    let mut proof_work = WorkBudget::new(WORK_ALLOWANCE);
    let forecast = forecast(
        &observed,
        &history,
        None,
        ForecastLimits::default(),
        &mut proof_work,
    )
    .unwrap();
    let (lease, contact) = certify_pickup_pulse(
        &observed,
        &forecast,
        PulseRequest {
            movement: MovementIntent::Direction(Direction::West),
            hold_ticks: 1,
            start_window: TickWindow {
                start: Tick(1_801),
                end: Tick(1_808),
            },
            stopped_safe_until: Tick(1_818),
            mode: RouteMode::CertifiedOnly,
            settled_stop_tick: Tick(1_800),
        },
        true,
        &mut proof_work,
    )
    .unwrap();
    assert_eq!(lease.movement(), MovementIntent::Direction(Direction::West));
    assert_eq!(lease.hold_ticks(), 1);
    assert!(contact.is_some());
    assert!(proof_work.spent() <= WORK_ALLOWANCE);

    for planner in [
        plan_pickup_progress_v31_with_origin,
        plan_hard_pickup_progress_v32_with_origin,
    ] {
        let mut work = WorkBudget::new(WORK_ALLOWANCE);
        let fixed = planner(
            &observed,
            &UtilityMemory::new(key),
            &history,
            observed.parts().tick,
            None,
            &mut CpuRng::from_seed(key, [131; 32]),
            &mut work,
        )
        .unwrap();
        assert_eq!(
            fixed.survival.control.intent.movement,
            MovementIntent::Direction(Direction::West)
        );
        assert_eq!(fixed.survival.lease.as_ref().unwrap().hold_ticks(), 1);
        assert!(fixed.contact.is_some());
        assert!(work.spent() <= WORK_ALLOWANCE);
    }

    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let archived = plan_reliable_v27_with_origin(
        &observed,
        &UtilityMemory::new(key),
        &history,
        observed.parts().tick,
        None,
        &mut CpuRng::from_seed(key, [131; 32]),
        &mut work,
    )
    .unwrap();
    assert_eq!(
        archived.survival.control.intent.movement,
        MovementIntent::Stop
    );
}

#[test]
fn pickup_progress_keeps_forecast_and_mob_contact_guards() {
    for observed in [
        captured_tick_1800(false)
            .map_parts(|parts| {
                parts.flames.push(ObservedFlame {
                    cell: CellId::new(9, 1),
                    remaining_ticks: 20,
                });
            })
            .unwrap(),
        captured_tick_1800(true),
    ] {
        let key = key(&observed);
        let mut history = ObservedHazardMemory::new(key);
        history.observe(&observed).unwrap();
        for planner in [
            plan_pickup_progress_v31_with_origin,
            plan_hard_pickup_progress_v32_with_origin,
        ] {
            let plan = planner(
                &observed,
                &UtilityMemory::new(key),
                &history,
                observed.parts().tick,
                None,
                &mut CpuRng::from_seed(key, [134; 32]),
                &mut WorkBudget::new(WORK_ALLOWANCE),
            )
            .unwrap();
            assert_ne!(
                plan.survival.control.intent.movement,
                MovementIntent::Direction(Direction::West)
            );
        }
    }
}

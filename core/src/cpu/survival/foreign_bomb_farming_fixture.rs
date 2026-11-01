use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{
    StrategicIntent, StrategicObjectiveKind, StrategicPhase, StrategicSwitchReason,
    StrategicTarget, UtilityMemory,
};
use crate::cpu::{
    BombKnowledge, MemoryKey, Observation, ObservationParts, ObservedActor, ObservedArena,
    ObservedBomb, ObservedBombTrigger, ObservedEnvironment, ObservedRespawnLairs, ObservedRules,
    ObservedSpikePits, UnknownActorFacts, UnknownBombFacts, VisibleActor, WorldId,
};
use crate::game::{
    BlastKind, CellId, Direction, EnabledItems, EntityId, LairStatusView, Position, SeatId,
    SpeedLevel, SpikePitPhase, SpikePitView, StormCloudConfig, StormCloudView, Tick, fixed_arena,
};
use crate::room::{
    RespawnLairConfig, RespawnLairSettings, RoomSettings, SpikePitConfig, SpikePitSettings,
};

pub(super) const CPU: SeatId = SeatId::Two;
pub(super) const SETTLED_STOP: Tick = Tick(234);

pub(super) struct CapturedBasis {
    pub observation: Observation,
    pub hazard_memory: ObservedHazardMemory,
    pub memory: UtilityMemory,
}

pub(super) fn captured_basis() -> CapturedBasis {
    let observation = captured_observation();
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let mut hazard_memory = ObservedHazardMemory::new(key);
    // The tick-231 input is a controlled reconstruction of the two retained
    // bomb samples. It changes only public clock and fuse facts needed to
    // reproduce the exact tracked history; it is not a restored match state.
    let previous = observation
        .clone()
        .map_parts(|parts| {
            parts.tick = Tick(231);
            parts.rules.running_updates = 51;
            parts.rules.remaining_ticks = 10_749;
            let BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                unreachable!()
            };
            for bomb in bombs {
                let ObservedBombTrigger::Timed {
                    remaining_ticks,
                    paused: false,
                } = &mut bomb.trigger
                else {
                    unreachable!()
                };
                *remaining_ticks = 154;
            }
        })
        .unwrap();
    hazard_memory.observe(&previous).unwrap();
    hazard_memory.observe(&observation).unwrap();
    let memory = UtilityMemory::from_route(key, &[CellId::new(11, 1)], false)
        .with_strategic_intent(StrategicIntent {
            objective: StrategicObjectiveKind::Farm,
            target: StrategicTarget::Block {
                cell: CellId::new(11, 2),
            },
            approach_cell: Some(CellId::new(11, 1)),
            phase: StrategicPhase::Approach,
            last_switch_reason: StrategicSwitchReason::Initial,
            observation_tick: Tick(210),
            score: 2,
            committed_until: Tick(303),
            progress_deadline: Tick(390),
            last_distance: 1,
            last_cpu_position: Position {
                x: 12_750,
                y: 1_536,
            },
            deferred_until: None,
        });
    CapturedBasis {
        observation,
        hazard_memory,
        memory,
    }
}

fn captured_observation() -> Observation {
    let parts = ObservationParts {
        world: WorldId {
            room_instance: 5_461,
            physical_round: 1,
        },
        tick: Tick(234),
        recipient: CPU,
        rules: ObservedRules {
            version: 10,
            tick_rate: 60,
            running_updates: 54,
            remaining_ticks: 10_746,
            settings: RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        },
        arena: ObservedArena {
            width: 15,
            height: 13,
            terrain: fixed_terrain(),
            blocks: blocks(),
        },
        actors: vec![
            actor(1, SeatId::One, 1_536, 1_536, Direction::East, 0),
            actor(2, CPU, 11_727, 1_536, Direction::West, 0),
            actor(3, SeatId::Three, 1_638, 11_776, Direction::East, 1),
            actor(4, SeatId::Four, 13_722, 11_776, Direction::West, 1),
        ],
        bombs: BombKnowledge::Visible(vec![
            bomb(0, 3, CellId::new(1, 10)),
            bomb(5, 4, CellId::new(13, 10)),
        ]),
        flames: Vec::new(),
        oil: Vec::new(),
        pickups: Vec::new(),
    };
    Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(
            StormCloudConfig::Single,
            Some(StormCloudView {
                position: Position { x: 7_248, y: 6_656 },
                direction: Direction::West,
                leg_remaining_ticks: 74,
            }),
            vec![Some(0); 4],
        ),
        Some(ObservedRespawnLairs::new(
            RespawnLairConfig::Enabled(RespawnLairSettings::new(10).unwrap()),
            [CellId::new(6, 6), CellId::new(8, 6)]
                .map(|cell| LairStatusView {
                    cell,
                    next_spawn_at: Some(Tick(780)),
                    living_mobs: 0,
                })
                .into(),
            Vec::new(),
        )),
        Some(ObservedSpikePits::new(
            SpikePitConfig::Enabled(SpikePitSettings::new(36, 180).unwrap()),
            [(5, 5), (9, 5), (5, 7), (9, 7)]
                .map(|(column, row)| SpikePitView {
                    cell: CellId::new(column, row),
                    phase: SpikePitPhase::Idle,
                    remaining_ticks: 0,
                })
                .into(),
        )),
    )
    .unwrap()
}

fn fixed_terrain() -> Vec<crate::game::Terrain> {
    let arena = fixed_arena();
    (0..195)
        .map(|index| arena.terrain(CellId::new(index % 15, index / 15)).unwrap())
        .collect()
}

fn actor(
    id: u32,
    seat: SeatId,
    x: i32,
    y: i32,
    heading: Direction,
    live_bomb_count: u16,
) -> ObservedActor {
    ObservedActor::Visible(VisibleActor {
        id: EntityId(id),
        seat,
        position: Position { x, y },
        heading,
        speed: SpeedLevel::Base,
        alive: true,
        bomb_capacity: 1,
        bomb_range: 2,
        live_bomb_count,
        equipment: None,
        weapon: None,
        effects: Vec::new(),
        damage_grace_remaining_ticks: 0,
        voluntary_actions_frozen: false,
        last_teleport_tick: None,
        unknown: UnknownActorFacts::ALL,
    })
}

fn bomb(id: u32, owner: u32, cell: CellId) -> ObservedBomb {
    ObservedBomb {
        id: EntityId(id),
        placed_at: Tick(205),
        cell,
        owner: EntityId(owner),
        range: 2,
        trigger: ObservedBombTrigger::Timed {
            remaining_ticks: 151,
            paused: false,
        },
        blast: BlastKind::Standard,
        unknown: UnknownBombFacts::ALL,
    }
}

fn blocks() -> Vec<CellId> {
    [
        (5, 1),
        (9, 1),
        (3, 2),
        (7, 2),
        (11, 2),
        (3, 3),
        (4, 3),
        (10, 3),
        (11, 3),
        (1, 4),
        (3, 4),
        (7, 4),
        (11, 4),
        (13, 4),
        (1, 5),
        (2, 5),
        (3, 5),
        (4, 5),
        (6, 5),
        (8, 5),
        (10, 5),
        (11, 5),
        (12, 5),
        (13, 5),
        (1, 7),
        (2, 7),
        (3, 7),
        (4, 7),
        (6, 7),
        (8, 7),
        (10, 7),
        (11, 7),
        (12, 7),
        (13, 7),
        (1, 8),
        (3, 8),
        (7, 8),
        (11, 8),
        (13, 8),
        (3, 9),
        (4, 9),
        (10, 9),
        (11, 9),
        (3, 10),
        (7, 10),
        (11, 10),
        (5, 11),
        (9, 11),
    ]
    .map(|(column, row)| CellId::new(column, row))
    .into()
}

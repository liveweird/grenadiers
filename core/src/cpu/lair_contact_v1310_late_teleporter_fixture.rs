use crate::cpu::{
    BombKnowledge, Observation, ObservationParts, ObservedActor, ObservedArena,
    ObservedEnvironment, ObservedPickup, ObservedRespawnLairs, ObservedRules, ObservedSpikePits,
    UnknownActorFacts, VisibleActor, WorldId,
};
use crate::game::{
    CellId, Direction, EnabledItems, EntityId, LairStatusView, MobView, PickupKind, Position,
    SeatId, SpeedLevel, SpikePitPhase, SpikePitView, StormCloudConfig, StormCloudView, Tick,
    fixed_arena,
};
use crate::room::{
    RespawnLairConfig, RespawnLairSettings, RoomSettings, SpikePitConfig, SpikePitSettings,
};

pub(crate) const CPU: SeatId = SeatId::Four;

// Reconstructs every public field retained by sample 88/tick 5_461. The raw
// capture does not retain a room identity or admission settings, so the world
// id is fixture-local and default/all-items is an explicit assumption. Every
// actor's controller facts remain UnknownActorFacts::ALL.
pub(crate) fn captured_tick_5461() -> Observation {
    let fixed = fixed_arena();
    let terrain = (0..195)
        .map(|index| fixed.terrain(CellId::new(index % 15, index / 15)).unwrap())
        .collect();
    #[rustfmt::skip]
    let blocks = [
        (5, 1), (5, 2), (7, 2), (4, 3), (7, 3), (3, 4), (5, 4), (7, 4), (9, 4),
        (2, 5), (3, 5), (5, 5), (6, 5), (7, 5), (8, 5), (9, 5), (11, 5), (12, 5),
        (1, 6), (3, 6), (7, 6), (11, 6), (13, 6), (1, 7), (2, 7), (3, 7), (5, 7),
        (6, 7), (7, 7), (8, 7), (9, 7), (11, 7), (12, 7), (3, 8), (5, 8), (7, 8),
        (9, 8), (11, 8), (4, 9), (7, 9), (10, 9), (3, 10), (5, 10), (7, 10),
        (9, 10), (4, 11), (5, 11), (9, 11),
    ]
    .into_iter()
    .map(|(column, row)| CellId::new(column, row))
    .collect();
    #[rustfmt::skip]
    let actors = vec![
        actor(1, SeatId::One, Position { x: 11_776, y: 10_243 }, Direction::East, true, Some(Tick(4_025))),
        actor(2, SeatId::Two, Position { x: 3_314, y: 9_728 }, Direction::West, true, Some(Tick(1_931))),
        actor(3, SeatId::Three, Position { x: 8_667, y: 3_584 }, Direction::West, false, Some(Tick(1_931))),
        actor(4, CPU, Position { x: 1_536, y: 2_724 }, Direction::South, true, Some(Tick(4_025))),
    ];
    let parts = ObservationParts {
        world: WorldId {
            room_instance: 140,
            physical_round: 1,
        },
        tick: Tick(5_461),
        recipient: CPU,
        rules: ObservedRules {
            version: 10,
            punch_activation_policy: crate::game::PunchActivationPolicy::ManualV1,
            tick_rate: 60,
            running_updates: 5_256,
            remaining_ticks: 5_544,
            settings: RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        },
        arena: ObservedArena {
            width: 15,
            height: 13,
            terrain,
            blocks,
        },
        actors,
        bombs: BombKnowledge::Visible(Vec::new()),
        flames: Vec::new(),
        oil: Vec::new(),
        pickups: vec![ObservedPickup {
            cell: CellId::new(3, 2),
            kind: PickupKind::Teleporter,
        }],
    };
    Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(
            StormCloudConfig::Single,
            Some(StormCloudView {
                position: Position { x: 3_648, y: 7_680 },
                direction: Direction::East,
                leg_remaining_ticks: 120,
            }),
            vec![Some(0); 4],
        ),
        Some(ObservedRespawnLairs::new(
            RespawnLairConfig::Enabled(RespawnLairSettings::DEFAULT),
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
                    id: EntityId(12),
                    lair: 0,
                    position: Position { x: 5_632, y: 6_656 },
                    heading: Direction::South,
                    frozen: false,
                },
                MobView {
                    id: EntityId(13),
                    lair: 1,
                    position: Position { x: 9_728, y: 6_656 },
                    heading: Direction::South,
                    frozen: false,
                },
            ],
        )),
        Some(ObservedSpikePits::new(
            SpikePitConfig::Enabled(SpikePitSettings::DEFAULT),
            [(6, 3), (8, 3), (6, 9), (8, 9)]
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

fn actor(
    id: u32,
    seat: SeatId,
    position: Position,
    heading: Direction,
    alive: bool,
    last_teleport_tick: Option<Tick>,
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
        last_teleport_tick,
        unknown: UnknownActorFacts::ALL,
    })
}

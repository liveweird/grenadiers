use crate::cpu::{
    BombKnowledge, Observation, ObservationParts, ObservedActor, ObservedArena,
    ObservedEnvironment, ObservedPickup, ObservedRespawnLairs, ObservedRules, ObservedSpikePits,
    UnknownActorFacts, VisibleActor, WorldId,
};
use crate::game::{
    CellId, Direction, EnabledItems, EntityId, Equipment, LairStatusView, MobView, PickupKind,
    Position, SeatId, SpeedLevel, SpikePitPhase, SpikePitView, StormCloudConfig, StormCloudView,
    Tick, fixed_arena,
};
use crate::room::{
    RespawnLairConfig, RespawnLairSettings, RoomSettings, SpikePitConfig, SpikePitSettings,
};

pub(crate) const CPU: SeatId = SeatId::Three;

// Reconstructs every public field retained by sample 99. The raw capture omits
// admission settings and all private actor/controller facts. Default/all-items
// is therefore an explicit configuration assumption; private facts remain
// unknown rather than being inferred from later behavior.
pub(crate) fn captured_tick_6121() -> Observation {
    let fixed = fixed_arena();
    let terrain = (0..195)
        .map(|index| fixed.terrain(CellId::new(index % 15, index / 15)).unwrap())
        .collect();
    let blocks = [
        (5, 1),
        (6, 1),
        (7, 1),
        (8, 1),
        (5, 2),
        (9, 2),
        (2, 3),
        (3, 3),
        (4, 3),
        (5, 3),
        (7, 3),
        (9, 3),
        (10, 3),
        (11, 3),
        (1, 4),
        (3, 4),
        (5, 4),
        (7, 4),
        (9, 4),
        (11, 4),
        (1, 5),
        (2, 5),
        (4, 5),
        (7, 5),
        (10, 5),
        (12, 5),
        (1, 6),
        (3, 6),
        (5, 6),
        (7, 6),
        (9, 6),
        (11, 6),
        (2, 7),
        (4, 7),
        (7, 7),
        (10, 7),
        (12, 7),
        (3, 8),
        (5, 8),
        (7, 8),
        (9, 8),
        (11, 8),
        (2, 9),
        (4, 9),
        (5, 9),
        (7, 9),
        (9, 9),
        (10, 9),
        (11, 9),
        (12, 9),
        (5, 10),
        (9, 10),
        (6, 11),
        (7, 11),
        (8, 11),
        (9, 11),
    ]
    .into_iter()
    .map(|(column, row)| CellId::new(column, row))
    .collect();
    let parts = ObservationParts {
        world: WorldId {
            room_instance: 139,
            physical_round: 1,
        },
        tick: Tick(6_121),
        recipient: CPU,
        rules: ObservedRules {
            version: 10,
            punch_activation_policy: crate::game::PunchActivationPolicy::ManualV1,
            tick_rate: 60,
            running_updates: 5_916,
            remaining_ticks: 4_884,
            settings: RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
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
                Position::center(CellId::new(1, 1)),
                Direction::East,
                None,
            ),
            actor(
                2,
                SeatId::Two,
                Position { x: 9_843, y: 1_536 },
                Direction::East,
                Some(Equipment::Punch),
            ),
            actor(
                3,
                CPU,
                Position {
                    x: 5_824,
                    y: 11_776,
                },
                Direction::East,
                Some(Equipment::Punch),
            ),
            actor(
                4,
                SeatId::Four,
                Position {
                    x: 11_776,
                    y: 11_481,
                },
                Direction::North,
                Some(Equipment::Punch),
            ),
        ],
        bombs: BombKnowledge::Visible(Vec::new()),
        flames: Vec::new(),
        oil: Vec::new(),
        pickups: [
            (13, 5, PickupKind::InvisibleBombs),
            (1, 7, PickupKind::InvisibleBombs),
            (13, 7, PickupKind::InvisibleBombs),
            (3, 9, PickupKind::Teleporter),
        ]
        .into_iter()
        .map(|(column, row, kind)| ObservedPickup {
            cell: CellId::new(column, row),
            kind,
        })
        .collect(),
    };
    Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(
            StormCloudConfig::Single,
            Some(StormCloudView {
                position: Position { x: 3_584, y: 4_384 },
                direction: Direction::North,
                leg_remaining_ticks: 100,
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
                    id: EntityId(9),
                    lair: 0,
                    position: Position { x: 5_632, y: 5_632 },
                    heading: Direction::West,
                    frozen: false,
                },
                MobView {
                    id: EntityId(10),
                    lair: 1,
                    position: Position { x: 9_728, y: 5_632 },
                    heading: Direction::East,
                    frozen: false,
                },
            ],
        )),
        Some(ObservedSpikePits::new(
            SpikePitConfig::Enabled(SpikePitSettings::DEFAULT),
            [(6, 5), (8, 5), (6, 7), (8, 7)]
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
    equipment: Option<Equipment>,
) -> ObservedActor {
    ObservedActor::Visible(VisibleActor {
        id: EntityId(id),
        seat,
        position,
        heading,
        speed: SpeedLevel::Base,
        alive: true,
        bomb_capacity: 1,
        bomb_range: 2,
        live_bomb_count: 0,
        equipment,
        weapon: None,
        effects: Vec::new(),
        damage_grace_remaining_ticks: 0,
        voluntary_actions_frozen: false,
        last_teleport_tick: None,
        unknown: UnknownActorFacts::ALL,
    })
}

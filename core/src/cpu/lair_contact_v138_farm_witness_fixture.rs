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

pub(super) const CPU: SeatId = SeatId::Three;

// Reconstructs every field retained by public sample 90. The capture omitted
// admission settings and private actor facts: default/all-items is an explicit
// reconstruction assumption consistent with its visible item set, and private
// facts remain unknown rather than being restored from hidden state.
pub(super) fn captured_tick_5580() -> Observation {
    let fixed = fixed_arena();
    let terrain = (0..195)
        .map(|index| fixed.terrain(CellId::new(index % 15, index / 15)).unwrap())
        .collect();
    let blocks = [
        (4, 1),
        (5, 1),
        (7, 1),
        (3, 2),
        (7, 2),
        (1, 3),
        (2, 3),
        (5, 3),
        (6, 3),
        (8, 3),
        (9, 3),
        (3, 4),
        (5, 4),
        (9, 4),
        (11, 4),
        (1, 5),
        (2, 5),
        (5, 5),
        (9, 5),
        (5, 6),
        (9, 6),
        (5, 7),
        (9, 7),
        (5, 8),
        (9, 8),
        (6, 9),
        (8, 9),
        (9, 9),
        (7, 10),
        (7, 11),
    ]
    .into_iter()
    .map(|(column, row)| CellId::new(column, row))
    .collect();
    let parts = ObservationParts {
        world: WorldId {
            room_instance: 138,
            physical_round: 1,
        },
        tick: Tick(5_580),
        recipient: CPU,
        rules: ObservedRules {
            version: 10,
            tick_rate: 60,
            running_updates: 5_379,
            remaining_ticks: 5_421,
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
                true,
                1,
                None,
            ),
            actor(
                2,
                SeatId::Two,
                Position {
                    x: 11_903,
                    y: 11_776,
                },
                Direction::West,
                false,
                3,
                None,
            ),
            actor(
                3,
                CPU,
                Position { x: 4_646, y: 5_632 },
                Direction::East,
                true,
                2,
                Some(Equipment::Punch),
            ),
            actor(
                4,
                SeatId::Four,
                Position {
                    x: 11_263,
                    y: 11_776,
                },
                Direction::East,
                false,
                1,
                None,
            ),
        ],
        bombs: BombKnowledge::Visible(Vec::new()),
        flames: Vec::new(),
        oil: Vec::new(),
        pickups: [
            (9, 1, PickupKind::Landmine),
            (3, 8, PickupKind::ShortFlame),
            (11, 8, PickupKind::ShortFlame),
            (5, 11, PickupKind::Landmine),
            (9, 11, PickupKind::Landmine),
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
                position: Position { x: 8_728, y: 7_680 },
                direction: Direction::East,
                leg_remaining_ticks: 125,
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
                    next_spawn_at: Some(Tick(5_889)),
                    living_mobs: 0,
                },
            ],
            vec![MobView {
                id: EntityId(18),
                lair: 0,
                position: Position { x: 7_680, y: 6_656 },
                heading: Direction::South,
                frozen: false,
            }],
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
    alive: bool,
    bomb_capacity: u16,
    equipment: Option<Equipment>,
) -> ObservedActor {
    ObservedActor::Visible(VisibleActor {
        id: EntityId(id),
        seat,
        position,
        heading,
        speed: SpeedLevel::Base,
        alive,
        bomb_capacity,
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

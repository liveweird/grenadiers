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

pub(crate) const CPU: SeatId = SeatId::Three;
pub(crate) const TARGET: CellId = CellId::new(13, 7);
pub(super) const OTHER: CellId = CellId::new(1, 7);

// Reconstructs every public field retained by sample 20/tick 1_381. The raw
// capture has no room identity or private Stop/controller history, so the
// world id is local and tests explicitly assume a settled Stop at this tick.
pub(crate) fn captured_tick_1381() -> Observation {
    let fixed = fixed_arena();
    let terrain = (0..195)
        .map(|index| fixed.terrain(CellId::new(index % 15, index / 15)).unwrap())
        .collect();
    #[rustfmt::skip]
    let blocks = [
        (5,1),(9,1),(3,2),(7,2),(11,2),(3,3),(4,3),(10,3),(11,3),(1,4),
        (3,4),(7,4),(11,4),(13,4),(1,5),(2,5),(3,5),(4,5),(6,5),(8,5),
        (10,5),(11,5),(12,5),(13,5),(2,7),(3,7),(4,7),(6,7),(8,7),(10,7),
        (11,7),(12,7),(3,8),(7,8),(11,8),(3,9),(4,9),(10,9),(11,9),(7,10),
    ]
    .into_iter().map(|(column,row)| CellId::new(column,row)).collect();
    let parts = ObservationParts {
        world: WorldId {
            room_instance: 1_381,
            physical_round: 1,
        },
        tick: Tick(1_381),
        recipient: CPU,
        rules: ObservedRules {
            version: 10,
            tick_rate: 60,
            running_updates: 1_185,
            remaining_ticks: 9_615,
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
            ),
            actor(
                2,
                SeatId::Two,
                Position {
                    x: 11_481,
                    y: 1_536,
                },
                Direction::West,
            ),
            actor(3, CPU, Position { x: 9_728, y: 9_385 }, Direction::North),
            actor(
                4,
                SeatId::Four,
                Position { x: 9_728, y: 8_745 },
                Direction::South,
            ),
        ],
        bombs: BombKnowledge::Visible(Vec::new()),
        flames: Vec::new(),
        oil: Vec::new(),
        pickups: vec![
            ObservedPickup {
                cell: OTHER,
                kind: PickupKind::Teleporter,
            },
            ObservedPickup {
                cell: TARGET,
                kind: PickupKind::Teleporter,
            },
        ],
    };
    Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(
            StormCloudConfig::Single,
            Some(StormCloudView {
                position: Position { x: 4_344, y: 4_608 },
                direction: Direction::West,
                leg_remaining_ticks: 95,
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
                    id: EntityId(10),
                    lair: 0,
                    position: Position::center(CellId::new(7, 6)),
                    heading: Direction::South,
                    frozen: false,
                },
                MobView {
                    id: EntityId(11),
                    lair: 1,
                    position: Position::center(CellId::new(9, 6)),
                    heading: Direction::South,
                    frozen: false,
                },
            ],
        )),
        Some(ObservedSpikePits::new(
            SpikePitConfig::Enabled(SpikePitSettings::DEFAULT),
            [
                (5, 5, SpikePitPhase::Idle, 0),
                (9, 5, SpikePitPhase::Idle, 0),
                (5, 7, SpikePitPhase::Idle, 0),
                (9, 7, SpikePitPhase::Warning, 5),
            ]
            .into_iter()
            .map(|(column, row, phase, remaining_ticks)| SpikePitView {
                cell: CellId::new(column, row),
                phase,
                remaining_ticks,
            })
            .collect(),
        )),
    )
    .unwrap()
}

fn actor(id: u32, seat: SeatId, position: Position, heading: Direction) -> ObservedActor {
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
        equipment: None,
        weapon: None,
        effects: Vec::new(),
        damage_grace_remaining_ticks: 0,
        voluntary_actions_frozen: false,
        last_teleport_tick: None,
        unknown: UnknownActorFacts::ALL,
    })
}

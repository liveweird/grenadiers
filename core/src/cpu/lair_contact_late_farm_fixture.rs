use crate::cpu::{
    BombKnowledge, Observation, ObservationParts, ObservedActor, ObservedArena,
    ObservedEnvironment, ObservedRespawnLairs, ObservedRules, ObservedSpikePits, UnknownActorFacts,
    VisibleActor, WorldId,
};
use crate::game::{
    ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, Equipment, InputAction,
    LairStatusView, Position, RoundAssignment, RulesConfig, SeatId, SeatInput, SimulationSeed,
    SpeedLevel, SpikePitPhase, SpikePitView, State, StormCloudConfig, StormCloudView, Tick, World,
    fixed_arena,
};
use crate::room::{
    RespawnLairConfig, RespawnLairSettings, RoomSettings, SpikePitConfig, SpikePitSettings,
};

pub(super) const CPU: SeatId = SeatId::Three;
pub(super) const APPROACH: CellId = CellId::new(3, 6);
pub(super) const TARGET: CellId = CellId::new(3, 5);

pub(super) fn captured_tick_10741() -> Observation {
    let fixed = fixed_arena();
    let terrain = (0..195)
        .map(|index| fixed.terrain(CellId::new(index % 15, index / 15)).unwrap())
        .collect();
    let blocks = serde_json::from_str::<Vec<[u16; 2]>>(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/blocked-birth-farm-blocks.json"
    )))
    .unwrap()
    .into_iter()
    .map(|[column, row]| CellId::new(column, row))
    .collect();
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL);
    let parts = ObservationParts {
        world: WorldId {
            room_instance: 137,
            physical_round: 1,
        },
        tick: Tick(10_741),
        recipient: CPU,
        rules: ObservedRules {
            version: 10,
            tick_rate: 60,
            running_updates: 10_545,
            remaining_ticks: 255,
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
                None,
            ),
            actor(
                2,
                SeatId::Two,
                Position {
                    x: 11_136,
                    y: 3_584,
                },
                Direction::East,
                false,
                None,
            ),
            actor(
                3,
                CPU,
                Position { x: 3_584, y: 6_974 },
                Direction::South,
                true,
                Some(Equipment::Punch),
            ),
            actor(
                4,
                SeatId::Four,
                Position { x: 8_704, y: 3_392 },
                Direction::North,
                true,
                Some(Equipment::Punch),
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
                position: Position { x: 1_928, y: 2_560 },
                direction: Direction::East,
                leg_remaining_ticks: 79,
            }),
            vec![Some(107), Some(0), Some(0), Some(0)],
        ),
        Some(ObservedRespawnLairs::new(
            RespawnLairConfig::Enabled(RespawnLairSettings::DEFAULT),
            [CellId::new(6, 6), CellId::new(8, 6)]
                .into_iter()
                .map(|cell| LairStatusView {
                    cell,
                    next_spawn_at: Some(Tick(10_741)),
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

pub(super) fn controlled_physical_state() -> State {
    let observation = captured_tick_10741();
    let parts = observation.parts();
    let arena = Arena::new(
        parts.arena.terrain.clone().try_into().unwrap(),
        parts.arena.blocks.iter().copied().collect(),
        [
            CellId::new(1, 1),
            CellId::new(13, 1),
            CellId::new(1, 11),
            CellId::new(13, 11),
        ],
    )
    .unwrap();
    let actors = parts
        .actors
        .iter()
        .map(|actor| {
            let ObservedActor::Visible(actor) = actor else {
                unreachable!()
            };
            ActorInit {
                id: actor.id,
                seat: actor.seat,
                position: actor.position,
                heading: actor.heading,
                speed: actor.speed,
            }
        })
        .collect::<Vec<_>>();
    // This executes only the certified local movement and blast geometry. It
    // does not reproduce the captured public cloud/lair/pit dynamics, hidden
    // loot or randomness, or replay provenance.
    let mut state = State::new_at_tick_with_assignment_and_seed(
        RulesConfig::environment_v10(
            parts.rules.settings,
            StormCloudConfig::Disabled,
            RespawnLairConfig::Disabled,
            SpikePitConfig::Disabled,
        ),
        World::new(arena),
        actors,
        parts.tick,
        RoundAssignment::first(),
        SimulationSeed::new([137; 32]),
    )
    .unwrap();
    state
        .eliminate_departed([false, true, false, false])
        .unwrap();
    state.replace_test_actor_equipment(CPU, Equipment::Punch);
    state.replace_test_actor_equipment(SeatId::Four, Equipment::Punch);
    state
}

pub(super) fn input(movement: Option<Direction>, bomb: bool) -> Vec<SeatInput> {
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

fn actor(
    id: u32,
    seat: SeatId,
    position: Position,
    heading: Direction,
    alive: bool,
    equipment: Option<Equipment>,
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
        equipment,
        weapon: None,
        effects: Vec::new(),
        damage_grace_remaining_ticks: 0,
        voluntary_actions_frozen: false,
        last_teleport_tick: None,
        unknown: UnknownActorFacts::ALL,
    })
}

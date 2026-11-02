use crate::cpu::{
    BombKnowledge, Observation, ObservationParts, ObservedActor, ObservedArena, ObservedBomb,
    ObservedBombTrigger, ObservedEnvironment, ObservedFlame, ObservedRespawnLairs, ObservedRules,
    ObservedSpikePits, UnknownActorFacts, UnknownBombFacts, VisibleActor, WorldId,
};
use crate::game::{
    ActorInit, Arena, BlastKind, BombTrigger, CellId, Direction, EnabledItems, EntityId, Position,
    RoundAssignment, RulesConfig, SeatId, SimulationSeed, SpeedLevel, SpikePitPhase, SpikePitView,
    State, StormCloudConfig, StormCloudView, Tick, World, fixed_arena,
};
use crate::room::{
    RespawnLairConfig, RespawnLairSettings, RoomSettings, SpikePitConfig, SpikePitSettings,
};

pub(super) const CPU: SeatId = SeatId::Three;
pub(super) const START: Position = Position { x: 2050, y: 9728 };

pub(crate) fn captured_scene(recipient: SeatId) -> Observation {
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL);
    let actors = [
        actor(
            EntityId(1),
            SeatId::One,
            Position { x: 1536, y: 1536 },
            Direction::East,
            0,
        ),
        actor(
            EntityId(2),
            SeatId::Two,
            Position { x: 13_824, y: 2980 },
            Direction::North,
            1,
        ),
        actor(EntityId(3), SeatId::Three, START, Direction::East, 1),
        actor(
            EntityId(4),
            SeatId::Four,
            Position { x: 13_310, y: 9728 },
            Direction::West,
            1,
        ),
    ];
    let bombs = BombKnowledge::Visible(vec![
        bomb(7, CellId::new(12, 3), 2),
        bomb(8, CellId::new(1, 8), 3),
        bomb(9, CellId::new(13, 8), 4),
    ]);
    let parts = ObservationParts {
        world: WorldId {
            room_instance: 131,
            physical_round: 1,
        },
        tick: Tick(540),
        recipient,
        rules: ObservedRules {
            version: 10,
            punch_activation_policy: crate::game::PunchActivationPolicy::ManualV1,
            tick_rate: 60,
            running_updates: 339,
            remaining_ticks: 10_461,
            settings,
        },
        arena: ObservedArena {
            width: 15,
            height: 13,
            terrain: fixed_terrain(),
            blocks: blocks(),
        },
        actors: actors.into(),
        bombs,
        flames: Vec::new(),
        oil: Vec::new(),
        pickups: Vec::new(),
    };
    Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(
            StormCloudConfig::Single,
            Some(StormCloudView {
                position: Position { x: 8704, y: 4968 },
                direction: Direction::North,
                leg_remaining_ticks: 45,
            }),
            vec![Some(0); 4],
        ),
        Some(ObservedRespawnLairs::new(
            RespawnLairConfig::Enabled(RespawnLairSettings::DEFAULT),
            vec![
                crate::game::LairStatusView {
                    cell: CellId::new(6, 6),
                    next_spawn_at: Some(Tick(801)),
                    living_mobs: 0,
                },
                crate::game::LairStatusView {
                    cell: CellId::new(8, 6),
                    next_spawn_at: Some(Tick(801)),
                    living_mobs: 0,
                },
            ],
            Vec::new(),
        )),
        Some(ObservedSpikePits::new(
            SpikePitConfig::Enabled(SpikePitSettings::DEFAULT),
            [
                CellId::new(5, 5),
                CellId::new(9, 5),
                CellId::new(5, 7),
                CellId::new(9, 7),
            ]
            .into_iter()
            .map(|cell| SpikePitView {
                cell,
                phase: SpikePitPhase::Idle,
                remaining_ticks: 0,
            })
            .collect(),
        )),
    )
    .unwrap()
}

pub(super) fn physical_state() -> State {
    let observation = captured_scene(CPU);
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
    let mut world = World::new(arena);
    let BombKnowledge::Visible(bombs) = &parts.bombs else {
        unreachable!()
    };
    for bomb in bombs {
        let ObservedBombTrigger::Timed {
            remaining_ticks,
            paused: false,
        } = bomb.trigger
        else {
            unreachable!()
        };
        world.bombs.insert(
            bomb.id,
            crate::game::bombs::Bomb::new(
                bomb.id,
                bomb.cell,
                bomb.owner,
                bomb.range,
                bomb.placed_at,
                BombTrigger::Timed {
                    detonates_at: Tick(parts.tick.0 + u64::from(remaining_ticks)),
                },
            )
            .unwrap(),
        );
    }
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
    // This isolates the captured local blast geometry. It does not claim to
    // restore hidden cloud RNG, mob state, spike state, or loot state.
    State::new_at_tick_with_assignment_and_seed(
        RulesConfig::environment_v10(
            parts.rules.settings,
            StormCloudConfig::Disabled,
            RespawnLairConfig::Enabled(RespawnLairSettings::DEFAULT),
            SpikePitConfig::Disabled,
        ),
        world,
        actors,
        parts.tick,
        RoundAssignment::first(),
        SimulationSeed::new([131; 32]),
    )
    .unwrap()
}

pub(super) fn physical_observation(state: &State, template: &Observation) -> Observation {
    let view = state.public_view();
    let mut parts = template.clone().into_parts();
    parts.tick = view.tick;
    parts.rules.running_updates = view.running_updates as u16;
    parts.rules.remaining_ticks = view.remaining_ticks as u16;
    parts.arena.blocks = view.arena.blocks;
    for observed in &mut parts.actors {
        let ObservedActor::Visible(actor) = observed else {
            unreachable!()
        };
        let current = state.actor(actor.seat).unwrap();
        actor.position = current.position();
        actor.heading = current.heading();
        actor.alive = current.alive();
        actor.live_bomb_count = state.live_bomb_count(actor.seat) as u16;
    }
    parts.bombs = BombKnowledge::Visible(
        view.bombs
            .into_iter()
            .map(|bomb| {
                let BombTrigger::Timed { detonates_at } = bomb.trigger else {
                    unreachable!()
                };
                ObservedBomb {
                    id: bomb.id,
                    cell: bomb.cell,
                    owner: bomb.owner,
                    range: bomb.range,
                    placed_at: bomb.placed_at,
                    trigger: ObservedBombTrigger::Timed {
                        remaining_ticks: (detonates_at.0 - view.tick.0) as u16,
                        paused: false,
                    },
                    blast: bomb.blast,
                    unknown: UnknownBombFacts::ALL,
                }
            })
            .collect(),
    );
    parts.flames = view
        .flames
        .into_iter()
        .map(|flame| ObservedFlame {
            cell: flame.cell,
            remaining_ticks: (flame.expires_at.0 - view.tick.0) as u16,
        })
        .collect();
    Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0); 4]),
        Some(ObservedRespawnLairs::new(
            RespawnLairConfig::Enabled(RespawnLairSettings::DEFAULT),
            state.lair_statuses().unwrap().to_vec(),
            state.mobs(),
        )),
        None,
    )
    .unwrap()
}

fn actor(
    id: EntityId,
    seat: SeatId,
    position: Position,
    heading: Direction,
    live_bomb_count: u16,
) -> ObservedActor {
    ObservedActor::Visible(VisibleActor {
        id,
        seat,
        position,
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

fn bomb(id: u32, cell: CellId, owner: u32) -> ObservedBomb {
    ObservedBomb {
        id: EntityId(id),
        cell,
        owner: EntityId(owner),
        range: 2,
        placed_at: Tick(490),
        trigger: ObservedBombTrigger::Timed {
            remaining_ticks: 130,
            paused: false,
        },
        blast: BlastKind::Standard,
        unknown: UnknownBombFacts::ALL,
    }
}

fn blocks() -> Vec<CellId> {
    [
        (3, 1),
        (6, 1),
        (7, 1),
        (8, 1),
        (3, 2),
        (11, 2),
        (3, 3),
        (4, 3),
        (7, 3),
        (10, 3),
        (11, 3),
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
        (13, 5),
        (3, 6),
        (11, 6),
        (1, 7),
        (2, 7),
        (3, 7),
        (6, 7),
        (8, 7),
        (11, 7),
        (12, 7),
        (13, 7),
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
    .collect()
}

fn fixed_terrain() -> Vec<crate::game::Terrain> {
    let arena = fixed_arena();
    (0..13)
        .flat_map(|row| (0..15).map(move |column| CellId::new(column, row)))
        .map(|cell| arena.terrain(cell).unwrap())
        .collect()
}

use std::collections::{BTreeMap, BTreeSet};

use grenadiers_core::cpu::{
    BombKnowledge, Observation, ObservationParts, ObservedActor, ObservedArena, ObservedBomb,
    ObservedBombTrigger, ObservedEnvironment, ObservedFlame, ObservedOil, ObservedPickup,
    ObservedRules, UnknownActorFacts, UnknownBombFacts, VisibleActor,
};
use grenadiers_core::game::{
    ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, Position, RoundAssignment,
    RulesConfig, SeatId, SimulationSeed, SpeedLevel, State, StormCloudConfig, Tick, World,
    fixed_arena,
};
use grenadiers_core::room::{RespawnLairConfig, RoomSettings, SpikePitConfig};

use super::refuge_handoff_owner_support::{CPU, WORLD};

pub(super) fn state() -> (State, RoomSettings) {
    let settings = RoomSettings::new(SpeedLevel::Four, 256, 3, 240)
        .unwrap()
        .with_enabled_items(EnabledItems::ALL);
    let fixed = fixed_arena();
    let terrain = std::array::from_fn(|index| {
        fixed
            .terrain(CellId::new((index % 15) as u16, (index / 15) as u16))
            .unwrap()
    });
    let arena = Arena::new(
        terrain,
        BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(13, 11), CellId::new(13, 1)],
    )
    .unwrap();
    let exposed = BTreeMap::from([
        (
            CellId::new(5, 8),
            grenadiers_core::game::PickupKind::BlastRange,
        ),
        (
            CellId::new(5, 7),
            grenadiers_core::game::PickupKind::BlastRange,
        ),
        (
            CellId::new(5, 6),
            grenadiers_core::game::PickupKind::BombCapacity,
        ),
    ]);
    let world = World::with_pickups(arena, BTreeMap::new(), BTreeMap::new(), exposed).unwrap();
    let actors = [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(CellId::new(8, 5)),
            heading: Direction::East,
            speed: settings.starting_speed(),
        },
        ActorInit {
            id: EntityId(2),
            seat: CPU,
            position: Position::center(CellId::new(5, 9)),
            heading: Direction::North,
            speed: settings.starting_speed(),
        },
        ActorInit {
            id: EntityId(3),
            seat: SeatId::Three,
            position: Position::center(CellId::new(13, 11)),
            heading: Direction::West,
            speed: settings.starting_speed(),
        },
    ];
    (
        State::new_at_tick_with_assignment_and_seed(
            RulesConfig::environment_v10(
                settings,
                StormCloudConfig::Disabled,
                RespawnLairConfig::Disabled,
                SpikePitConfig::Disabled,
            ),
            world,
            actors,
            Tick(30),
            RoundAssignment::first(),
            SimulationSeed::new([30; 32]),
        )
        .unwrap(),
        settings,
    )
}

pub(super) fn observe(state: &State, settings: RoomSettings) -> Observation {
    let view = state.public_view();
    let actors = view
        .actors
        .iter()
        .map(|actor| {
            ObservedActor::Visible(VisibleActor {
                id: actor.id,
                seat: actor.seat,
                position: actor.position,
                heading: actor.heading,
                speed: actor.speed,
                alive: actor.alive,
                bomb_capacity: actor.bomb_capacity,
                bomb_range: actor.bomb_range,
                live_bomb_count: actor.live_bomb_count as u16,
                equipment: None,
                weapon: None,
                effects: Vec::new(),
                damage_grace_remaining_ticks: 0,
                voluntary_actions_frozen: false,
                last_teleport_tick: None,
                unknown: UnknownActorFacts::ALL,
            })
        })
        .collect();
    let bombs = view
        .bombs
        .iter()
        .map(|bomb| ObservedBomb {
            id: bomb.id,
            placed_at: bomb.placed_at,
            cell: bomb.cell,
            owner: bomb.owner,
            range: bomb.range,
            trigger: match bomb.trigger {
                grenadiers_core::game::BombTrigger::Timed { detonates_at } => {
                    ObservedBombTrigger::Timed {
                        remaining_ticks: (detonates_at.0 - view.tick.0) as u16,
                        paused: false,
                    }
                }
                grenadiers_core::game::BombTrigger::Remote => ObservedBombTrigger::Remote,
                grenadiers_core::game::BombTrigger::Contact => ObservedBombTrigger::Contact,
            },
            blast: bomb.blast,
            unknown: UnknownBombFacts::ALL,
        })
        .collect();
    let parts = ObservationParts {
        world: WORLD,
        tick: view.tick,
        recipient: CPU,
        rules: ObservedRules {
            version: 10,
            tick_rate: 60,
            running_updates: view.running_updates as u16,
            remaining_ticks: view.remaining_ticks as u16,
            settings,
        },
        arena: ObservedArena {
            width: view.arena.width,
            height: view.arena.height,
            terrain: view.arena.terrain,
            blocks: view.arena.blocks,
        },
        actors,
        bombs: BombKnowledge::Visible(bombs),
        flames: view
            .flames
            .into_iter()
            .filter_map(|flame| {
                let remaining = flame.expires_at.0.checked_sub(view.tick.0)?;
                (remaining > 0).then_some(ObservedFlame {
                    cell: flame.cell,
                    remaining_ticks: remaining as u16,
                })
            })
            .collect(),
        oil: view
            .oil_barrels
            .into_iter()
            .map(|oil| ObservedOil {
                cell: oil.cell,
                state: oil.state,
            })
            .collect(),
        pickups: view
            .pickups
            .into_iter()
            .map(|pickup| ObservedPickup {
                cell: pickup.cell,
                kind: pickup.kind,
            })
            .collect(),
    };
    Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(StormCloudConfig::Disabled, None, vec![Some(0); 3]),
        None,
        None,
    )
    .unwrap()
}

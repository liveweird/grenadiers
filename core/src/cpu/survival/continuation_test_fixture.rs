use std::collections::{BTreeMap, BTreeSet};

use crate::game::{
    ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, PickupKind, Position,
    RoundAssignment, RulesConfig, SeatId, SimulationSeed, State, StormCloudConfig, Tick, World,
    fixed_arena,
};
use crate::room::{RespawnLairConfig, RespawnLairSettings, RoomSettings, SpikePitConfig};

pub(super) const CPU: SeatId = SeatId::Two;
pub(super) const ORIGIN: CellId = CellId::new(1, 9);
const RIVAL_ORIGIN: CellId = CellId::new(13, 9);

fn blocks() -> BTreeSet<CellId> {
    [
        (3, 1),
        (4, 1),
        (6, 1),
        (7, 1),
        (8, 1),
        (10, 1),
        (3, 2),
        (5, 2),
        (9, 2),
        (11, 2),
        (1, 3),
        (2, 3),
        (5, 3),
        (6, 3),
        (8, 3),
        (9, 3),
        (12, 3),
        (1, 4),
        (5, 4),
        (7, 4),
        (9, 4),
        (13, 4),
        (2, 5),
        (3, 5),
        (5, 5),
        (6, 5),
        (7, 5),
        (8, 5),
        (9, 5),
        (11, 5),
        (12, 5),
        (1, 6),
        (3, 6),
        (5, 6),
        (7, 6),
        (9, 6),
        (11, 6),
        (13, 6),
        (2, 7),
        (3, 7),
        (5, 7),
        (6, 7),
        (7, 7),
        (8, 7),
        (9, 7),
        (11, 7),
        (12, 7),
        (1, 8),
        (5, 8),
        (7, 8),
        (9, 8),
        (13, 8),
        (2, 9),
        (5, 9),
        (6, 9),
        (8, 9),
        (9, 9),
        (12, 9),
        (3, 10),
        (5, 10),
        (9, 10),
        (11, 10),
        (4, 11),
        (6, 11),
        (7, 11),
        (8, 11),
        (10, 11),
    ]
    .into_iter()
    .map(|(column, row)| CellId::new(column, row))
    .collect()
}

pub(super) fn state_with_lairs(enabled: bool) -> (State, RoomSettings) {
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL);
    let fixed = fixed_arena();
    let terrain = std::array::from_fn(|index| {
        fixed
            .terrain(CellId::new((index % 15) as u16, (index / 15) as u16))
            .unwrap()
    });
    let arena = Arena::new(terrain, blocks(), [CellId::new(1, 1), CellId::new(13, 11)]).unwrap();
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([
            (ORIGIN, PickupKind::ShortFuse),
            (RIVAL_ORIGIN, PickupKind::ShortFuse),
        ]),
    )
    .unwrap();
    let actors = [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(RIVAL_ORIGIN),
            heading: Direction::South,
            speed: settings.starting_speed(),
        },
        ActorInit {
            id: EntityId(2),
            seat: CPU,
            position: Position::center(ORIGIN),
            heading: Direction::South,
            speed: settings.starting_speed(),
        },
    ];
    let state = State::new_at_tick_with_assignment_and_seed(
        RulesConfig::environment_v10(
            settings,
            StormCloudConfig::Disabled,
            if enabled {
                RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap())
            } else {
                RespawnLairConfig::Disabled
            },
            SpikePitConfig::Disabled,
        ),
        world,
        actors,
        Tick(0),
        RoundAssignment::first(),
        SimulationSeed::new([35; 32]),
    )
    .unwrap();
    (state, settings)
}

pub(super) fn observe(state: &State, settings: RoomSettings) -> crate::cpu::Observation {
    super::counter_pressure_v29_tests::observe_state(state, settings)
}

pub(super) fn step_until(state: &mut State, tick: Tick) {
    while state.tick() < tick {
        state.step(&[]).unwrap();
    }
}

pub(super) fn displaced_forecast(
    basis: &crate::cpu::Observation,
    key: crate::cpu::MemoryKey,
) -> (crate::cpu::forecast::HazardForecast, crate::cpu::WorkBudget) {
    let prior = basis
        .clone()
        .map_parts(|parts| {
            parts.tick = Tick(parts.tick.0 - 1);
            parts.rules.running_updates -= 1;
            parts.rules.remaining_ticks += 1;
            let crate::cpu::BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                unreachable!()
            };
            bombs[0].cell = CellId::new(1, 10);
            for bomb in bombs {
                let crate::cpu::ObservedBombTrigger::Timed {
                    remaining_ticks, ..
                } = &mut bomb.trigger
                else {
                    unreachable!()
                };
                *remaining_ticks += 1;
            }
        })
        .unwrap();
    let mut history = crate::cpu::forecast::ObservedHazardMemory::new(key);
    history.observe(&prior).unwrap();
    history.observe(basis).unwrap();
    let mut work = crate::cpu::WorkBudget::new(super::WORK_ALLOWANCE);
    let hazards = crate::cpu::forecast::forecast(
        basis,
        &history,
        None,
        crate::cpu::forecast::ForecastLimits::default(),
        &mut work,
    )
    .unwrap();
    (hazards, work)
}

pub(super) fn unsupported_scenes(basis: &crate::cpu::Observation) -> Vec<crate::cpu::Observation> {
    let pickup = |cell, kind| {
        basis
            .clone()
            .map_parts(|parts| {
                parts
                    .pickups
                    .push(crate::cpu::ObservedPickup { cell, kind })
            })
            .unwrap()
    };
    let paused = basis
        .clone()
        .map_parts(|parts| {
            let crate::cpu::BombKnowledge::Visible(bombs) = &mut parts.bombs else {
                unreachable!()
            };
            let crate::cpu::ObservedBombTrigger::Timed { paused, .. } = &mut bombs[0].trigger
            else {
                unreachable!()
            };
            *paused = true;
        })
        .unwrap();
    let flame = basis
        .clone()
        .map_parts(|parts| {
            parts.flames.push(crate::cpu::ObservedFlame {
                cell: CellId::new(2, 11),
                remaining_ticks: 2,
            });
        })
        .unwrap();
    let oil = basis
        .clone()
        .map_parts(|parts| {
            parts.oil.push(crate::cpu::ObservedOil {
                cell: CellId::new(2, 11),
                state: crate::game::OilBarrelState::Burning,
            });
        })
        .unwrap();
    vec![
        pickup(CellId::new(2, 11), PickupKind::Teleporter),
        pickup(ORIGIN, PickupKind::Diarrhea),
        paused,
        flame,
        oil,
    ]
}

pub(super) fn rebased_scene(basis: &crate::cpu::Observation) -> crate::cpu::Observation {
    basis
        .clone()
        .map_parts(|parts| parts.world.physical_round += 1)
        .unwrap()
}

pub(super) fn observe_for(
    state: &State,
    settings: RoomSettings,
    recipient: SeatId,
) -> crate::cpu::Observation {
    let tick = state.tick();
    observe(state, settings)
        .map_parts(|parts| {
            parts.recipient = recipient;
            let crate::cpu::ObservedActor::Visible(actor) = parts
                .actors
                .iter_mut()
                .find(|actor| actor.seat() == recipient)
                .unwrap()
            else {
                unreachable!()
            };
            actor.effects = state
                .actor(recipient)
                .unwrap()
                .effects()
                .observations()
                .filter_map(|(kind, expires_at)| {
                    Some(crate::cpu::ObservedEffect {
                        kind,
                        remaining_ticks: u16::try_from(expires_at.0.checked_sub(tick.0)?).ok()?,
                    })
                })
                .collect();
        })
        .unwrap()
}

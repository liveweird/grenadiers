use crate::game::{
    ActorInit, ArenaGenerationConfig, EffectKind, EnabledItems, PlayerCount, Position,
    RoundAssignment, RulesConfig, SeatId, SimulationSeed, SpeedLevel, State, StormCloudConfig,
    World, generate_multiplayer_arena, generate_variable_multiplayer_arena,
};
use crate::protocol::Counter;
use crate::room::BoardSize as DomainBoard;
use crate::room::{RespawnLairConfig, RespawnLairSettings, RespawnLairTuning};

use super::*;

pub(super) struct WorldCase {
    pub(super) room: RoomView,
    pub(super) capture: ExpandedCapture,
    pub(super) generation: crate::game::GenerationRecord,
    pub(super) state: State,
    pub(super) lifecycle: crate::room::RosterMatchLifecycle,
}

pub(super) fn counter(value: u32) -> Counter {
    Counter::new(value).unwrap()
}

fn generated(
    board: DomainBoard,
    count: PlayerCount,
    enabled: EnabledItems,
    seed: [u8; 32],
) -> crate::game::GeneratedArena {
    let config = ArenaGenerationConfig {
        player_count: count,
        board,
        assignment: RoundAssignment::first(),
    };
    if board == DomainBoard::FifteenByThirteen {
        generate_multiplayer_arena(seed, enabled, config).unwrap()
    } else {
        generate_variable_multiplayer_arena(seed, enabled, config).unwrap()
    }
}

pub(super) fn world_case(
    board: DomainBoard,
    count: PlayerCount,
    cloud: StormCloudConfig,
    lairs: RespawnLairConfig,
    advance_ticks: usize,
    effects: (bool, bool),
    seed: [u8; 32],
) -> WorldCase {
    world_case_with_navigation(
        board,
        count,
        cloud,
        lairs,
        advance_ticks,
        effects,
        seed,
        crate::game::MobNavigationPolicy::LegacyV1,
    )
}

#[allow(clippy::too_many_arguments)] // Add an explicit private AI identity to the existing test matrix.
pub(super) fn world_case_with_navigation(
    board: DomainBoard,
    count: PlayerCount,
    cloud: StormCloudConfig,
    lairs: RespawnLairConfig,
    advance_ticks: usize,
    effects: (bool, bool),
    seed: [u8; 32],
    navigation: crate::game::MobNavigationPolicy,
) -> WorldCase {
    let (cloak_center_actor, freeze_mobs) = effects;
    let enabled = if cloak_center_actor {
        EnabledItems::INVISIBILITY_CLOAK
    } else if freeze_mobs {
        EnabledItems::STOPWATCH
    } else {
        EnabledItems::EMPTY
    };
    let (world, generation) =
        World::from_generated(generated(board, count, enabled, seed)).unwrap();
    let spawns = world.arena().spawns();
    let actors = count
        .seats()
        .iter()
        .map(|&value| ActorInit {
            id: crate::game::EntityId(10 + value.index() as u32),
            seat: value,
            position: Position::center(spawns[value.index()]),
            heading: crate::game::Direction::East,
            speed: SpeedLevel::Base,
        })
        .collect::<Vec<_>>();
    let rules = RulesConfig::environment_v9_with_mob_navigation(
        crate::room::RoomSettings::DEFAULT.with_enabled_items(enabled),
        cloud,
        lairs,
        navigation,
    );
    let mut lifecycle = super::server_conformance_lifecycle::lifecycle(count, board, enabled, true);
    let mut state = State::new_at_tick_with_assignment_and_seed(
        rules,
        world,
        actors,
        lifecycle.tick(),
        RoundAssignment::first(),
        SimulationSeed::new(seed),
    )
    .unwrap();
    let mut reports = Vec::new();
    for _ in 0..advance_ticks {
        state.step(&[]).unwrap();
        lifecycle.complete_running_tick(None).unwrap();
    }
    if freeze_mobs {
        state.apply_test_effect(SeatId::One, EffectKind::Stopwatch, state.tick());
    }
    if cloud == StormCloudConfig::Single {
        let center = state.storm_cloud().unwrap().position;
        let centered_seat = if cloak_center_actor {
            SeatId::Two
        } else {
            SeatId::One
        };
        state.set_test_actor_pose(centered_seat, center, crate::game::Direction::East);
        if cloak_center_actor {
            state.apply_test_effect(
                SeatId::Two,
                EffectKind::InvisibilityCloak,
                crate::game::Tick(0),
            );
        }
        reports.push(state.step(&[]).unwrap());
        lifecycle.complete_running_tick(None).unwrap();
    }
    let capture = capture_expanded(&state, &reports).unwrap();
    WorldCase {
        room: super::server_conformance_lifecycle::room_view_with_tuning(
            &lifecycle,
            cloud,
            match lairs {
                RespawnLairConfig::Disabled => {
                    RespawnLairTuning::new(false, RespawnLairSettings::DEFAULT)
                }
                RespawnLairConfig::Enabled(settings) => RespawnLairTuning::new(true, settings),
            },
            true,
        ),
        capture,
        generation,
        state,
        lifecycle,
    }
}

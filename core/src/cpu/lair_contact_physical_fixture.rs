use std::collections::BTreeMap;

use crate::cpu::policy::{
    StrategicIntent, StrategicObjectiveKind, StrategicPhase, StrategicSwitchReason,
    StrategicTarget, UtilityMemory,
};
use crate::cpu::{MemoryKey, Observation};
use crate::game::{
    ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, InputAction, MobNavigationPolicy,
    Position, RoundAssignment, RulesConfig, SeatId, SeatInput, SimulationSeed, SpeedLevel, State,
    StormCloudConfig, Tick, World, fixed_arena,
};
use crate::room::{RespawnLairConfig, RespawnLairSettings, RoomSettings, SpikePitConfig};

use super::connectivity_tests::captured_blocks;

pub(super) const CPU: SeatId = SeatId::Two;
pub(super) const ORIGIN: CellId = CellId::new(10, 7);
pub(super) const TARGET: CellId = CellId::new(9, 7);
pub(super) const MOB_POCKET: CellId = CellId::new(7, 6);

pub(super) fn controlled_state() -> (State, RoomSettings) {
    let fixed = fixed_arena();
    let terrain = std::array::from_fn(|index| {
        fixed
            .terrain(CellId::new((index % 15) as u16, (index / 15) as u16))
            .unwrap()
    });
    let arena = Arena::new(
        terrain,
        captured_blocks(),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap();
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL);
    let rules = RulesConfig::environment_v10_with_mob_navigation(
        settings,
        StormCloudConfig::Disabled,
        RespawnLairConfig::Enabled(RespawnLairSettings::DEFAULT),
        SpikePitConfig::Disabled,
        MobNavigationPolicy::TimedEscapeV3,
    );
    let state = State::new_at_tick_with_assignment_and_seed(
        rules,
        World::with_pickups(arena, BTreeMap::new(), BTreeMap::new(), BTreeMap::new()).unwrap(),
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position: Position::center(CellId::new(1, 1)),
                heading: Direction::East,
                speed: SpeedLevel::Base,
            },
            ActorInit {
                id: EntityId(2),
                seat: CPU,
                position: Position::center(ORIGIN),
                heading: Direction::East,
                speed: SpeedLevel::Base,
            },
        ],
        Tick(0),
        RoundAssignment::first(),
        SimulationSeed::new([136; 32]),
    )
    .unwrap();
    (state, settings)
}

pub(super) fn memory(observation: &Observation) -> UtilityMemory {
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let tick = observation.parts().tick;
    UtilityMemory::new(key).with_strategic_intent(StrategicIntent {
        objective: StrategicObjectiveKind::Farm,
        target: StrategicTarget::Block { cell: TARGET },
        approach_cell: Some(ORIGIN),
        phase: StrategicPhase::Approach,
        last_switch_reason: StrategicSwitchReason::Stalled,
        observation_tick: tick,
        score: 2,
        committed_until: Tick(tick.0 + 120),
        progress_deadline: Tick(tick.0 + 180),
        last_distance: 0,
        last_cpu_position: Position::center(ORIGIN),
        deferred_until: None,
    })
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

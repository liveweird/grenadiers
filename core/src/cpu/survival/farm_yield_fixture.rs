use std::collections::{BTreeMap, BTreeSet};

use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::{MemoryKey, Observation};
use crate::game::{
    ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, InputAction, PickupKind, Position,
    RoundAssignment, RulesConfig, SeatId, SeatInput, SimulationSeed, State, StormCloudConfig, Tick,
    World, fixed_arena,
};
use crate::room::{RespawnLairConfig, RoomSettings, SpikePitConfig};

pub(crate) const CPU: SeatId = SeatId::Two;
pub(crate) const START: CellId = CellId::new(5, 5);
pub(crate) const FARM_TARGET: CellId = CellId::new(3, 7);
pub(crate) const SECOND_BLOCK: CellId = CellId::new(7, 7);
pub(crate) const BASELINE_ORIGIN: CellId = CellId::new(3, 6);
pub(crate) const HIGH_YIELD_ORIGIN: CellId = CellId::new(4, 7);
pub(crate) const EARLY_HIGH_YIELD_ORIGIN: CellId = CellId::new(5, 7);

pub(crate) struct FarmYieldBasis {
    pub(crate) state: State,
    pub(crate) settings: RoomSettings,
    pub(crate) observation: Observation,
    pub(crate) history: ObservedHazardMemory,
    pub(crate) key: MemoryKey,
}

pub(crate) fn collected_basis_at(destination: CellId) -> FarmYieldBasis {
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL);
    let mut state = State::new_at_tick_with_assignment_and_seed(
        RulesConfig::environment_v10(
            settings,
            StormCloudConfig::Disabled,
            RespawnLairConfig::Disabled,
            SpikePitConfig::Disabled,
        ),
        World::with_pickups(
            arena(),
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(START, PickupKind::BlastRange)]),
        )
        .unwrap(),
        actors(settings),
        Tick(0),
        RoundAssignment::first(),
        SimulationSeed::new([58; 32]),
    )
    .unwrap();
    state.step(&[]).unwrap();
    assert_eq!(state.actor(CPU).unwrap().bomb_range(), 3);
    walk_to(&mut state, destination);
    let observation = super::reliable_v27_oil_tests::observe(&state, settings);
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observation).unwrap();
    FarmYieldBasis {
        state,
        settings,
        observation,
        history,
        key,
    }
}

fn walk_to(state: &mut State, destination: CellId) {
    let legs: &[(Direction, usize)] = match destination {
        START => &[],
        BASELINE_ORIGIN => &[(Direction::West, 40), (Direction::South, 20)],
        HIGH_YIELD_ORIGIN => &[(Direction::South, 40), (Direction::West, 20)],
        EARLY_HIGH_YIELD_ORIGIN => &[(Direction::South, 40)],
        _ => panic!("unsupported controlled destination"),
    };
    for &(direction, updates) in legs {
        for update in 0..updates {
            let inputs = (update == 0)
                .then_some(SeatInput {
                    seat: CPU,
                    action: InputAction::Press(direction),
                })
                .into_iter()
                .collect::<Vec<_>>();
            state.step(&inputs).unwrap();
        }
    }
    if !legs.is_empty() {
        state
            .step(&[SeatInput {
                seat: CPU,
                action: InputAction::Clear,
            }])
            .unwrap();
    }
    assert_eq!(
        state.actor(CPU).unwrap().position(),
        Position::center(destination)
    );
}

fn arena() -> Arena {
    let fixed = fixed_arena();
    let terrain = std::array::from_fn(|index| {
        let column = u16::try_from(index % 15).unwrap();
        let row = u16::try_from(index / 15).unwrap();
        fixed.terrain(CellId::new(column, row)).unwrap()
    });
    Arena::new(
        terrain,
        BTreeSet::from([
            FARM_TARGET,
            SECOND_BLOCK,
            CellId::new(10, 9),
            CellId::new(12, 9),
            CellId::new(11, 8),
            CellId::new(11, 10),
        ]),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap()
}

fn actors(settings: RoomSettings) -> [ActorInit; 2] {
    [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(CellId::new(11, 9)),
            heading: Direction::West,
            speed: settings.starting_speed(),
        },
        ActorInit {
            id: EntityId(2),
            seat: CPU,
            position: Position::center(START),
            heading: Direction::East,
            speed: settings.starting_speed(),
        },
    ]
}

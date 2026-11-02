use std::collections::{BTreeMap, BTreeSet};

use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::{MemoryKey, Observation};
use crate::game::{
    ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, InputAction, PickupKind, Position,
    RoundAssignment, RulesConfig, SeatId, SeatInput, SimulationSeed, State, StormCloudConfig, Tick,
    World, fixed_arena,
};
use crate::room::{RespawnLairConfig, RespawnLairSettings, RoomSettings, SpikePitConfig};

pub(crate) const CPU: SeatId = SeatId::Two;
pub(crate) const RIVAL: SeatId = SeatId::One;
pub(crate) const START: CellId = CellId::new(5, 5);
pub(crate) const RIVAL_CELL: CellId = CellId::new(7, 7);
pub(crate) const NORTH_ORIGIN: CellId = CellId::new(7, 5);
pub(crate) const WEST_ORIGIN: CellId = CellId::new(5, 7);
pub(crate) const SOUTH_EXIT_BLOCK: CellId = CellId::new(7, 8);

pub(crate) struct PressureBasis {
    pub(crate) observation: Observation,
    pub(crate) history: ObservedHazardMemory,
    pub(crate) key: MemoryKey,
}

pub(crate) fn collected_basis_at(destination: CellId) -> PressureBasis {
    collected_basis_at_with_lairs(destination, false)
}

pub(crate) fn collected_basis_at_with_lairs(destination: CellId, lairs: bool) -> PressureBasis {
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL);
    let lairs = if lairs {
        RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap())
    } else {
        RespawnLairConfig::Disabled
    };
    let mut state = State::new_at_tick_with_assignment_and_seed(
        RulesConfig::environment_v10(
            settings,
            StormCloudConfig::Disabled,
            lairs,
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
        SimulationSeed::new([57; 32]),
    )
    .unwrap();
    state.step(&[]).unwrap();
    assert_eq!(state.actor(CPU).unwrap().bomb_range(), 3);
    if destination != START {
        let direction = match destination {
            NORTH_ORIGIN => Direction::East,
            WEST_ORIGIN => Direction::South,
            _ => panic!("unsupported controlled destination"),
        };
        for update in 0..40 {
            let input = (update == 0)
                .then_some(SeatInput {
                    seat: CPU,
                    action: InputAction::Press(direction),
                })
                .into_iter()
                .collect::<Vec<_>>();
            state.step(&input).unwrap();
        }
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
    let observation = super::reliable_v27_oil_tests::observe(&state, settings);
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observation).unwrap();
    PressureBasis {
        observation,
        history,
        key,
    }
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
        BTreeSet::from([SOUTH_EXIT_BLOCK]),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap()
}

fn actors(settings: RoomSettings) -> [ActorInit; 2] {
    [
        ActorInit {
            id: EntityId(1),
            seat: RIVAL,
            position: Position::center(RIVAL_CELL),
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

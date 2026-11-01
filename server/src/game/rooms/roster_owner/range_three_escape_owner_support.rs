use std::collections::{BTreeMap, BTreeSet};

use grenadiers_core::{
    game::{
        ActorInit, Arena, CellId, Direction, EntityId, PickupKind, Position, RoundActorInit,
        SeatId, SimulationSeed, State, Tick, World, fixed_arena,
    },
    room::{CpuProfile, MatchPhase},
};
use tokio::time::Instant;

use super::{
    late_teleporter_owner_support::scenario_driver_for_cpus,
    range_three_escape_diagnostics::{QueuedBasis, SelectorResult, capture},
};
use crate::game::rooms::cpu::{AdmissionTrace, PlanningBasis, PumpReport};

const CPUS: [SeatId; 3] = [SeatId::Two, SeatId::Three, SeatId::Four];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Trigger {
    pub placed_at: Tick,
    pub detonates_at: Tick,
    pub cells: [CellId; 3],
    pub oil: Vec<CellId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ControllerFlags {
    pub escaping: bool,
    pub origin: bool,
    pub pulse: bool,
    pub route: bool,
    pub continuation: bool,
    pub fence: bool,
}

#[derive(Clone, Debug)]
pub(super) struct ControllerTransition {
    pub seat: SeatId,
    pub tick: Tick,
    pub flags: ControllerFlags,
    pub state: String,
}

#[derive(Debug)]
pub(super) struct Outcome {
    pub with_oil: bool,
    pub initial_blocks: usize,
    pub initial_positions: [Position; 3],
    pub trigger: Option<Trigger>,
    pub first_deaths: [Option<Tick>; 3],
    pub max_visible_flames: usize,
    pub flames_clear_after_trigger: Option<Tick>,
    pub final_tick: Tick,
    pub final_phase: MatchPhase,
    pub queued_bases: Vec<QueuedBasis>,
    pub recovery_bases: Vec<QueuedBasis>,
    pub transitions: Vec<ControllerTransition>,
    pub worker: PumpReport,
    pub admissions: [AdmissionTrace; 3],
}

pub(super) fn run(with_oil: bool, ticks: usize) -> Outcome {
    let scene = if with_oil {
        scene_with_oil
    } else {
        scene_without_oil
    };
    let (mut owner, planner) =
        scenario_driver_for_cpus(CpuProfile::HARD_V53, &CPUS, scene, None, [47; 32]);
    let initial = owner.runtime.state_for_test().unwrap();
    let initial_blocks = initial.world().arena().blocks().len();
    let initial_positions = CPUS.map(|seat| initial.actor(seat).unwrap().position());
    let mut trigger = None;
    let mut first_deaths = [None; 3];
    let mut max_visible_flames = 0;
    let mut flames_clear_after_trigger = None;
    let mut queued_bases = Vec::new();
    let mut recovery_bases = Vec::new();
    let mut transitions = Vec::new();
    let mut prior_flags = [None; 3];
    let mut worker = PumpReport::default();
    for _ in 0..ticks {
        let report = planner.pump_for_test_inspecting(usize::MAX, |basis| {
            capture_diagnostics(basis, with_oil, &mut queued_bases, &mut recovery_bases);
        });
        merge(&mut worker, report);
        owner.tick(Instant::now()).unwrap();
        assert_eq!(owner.runtime.lifecycle().round_attempt(), 1);
        let state = owner.runtime.state_for_test().unwrap();
        let tick = state.tick();
        trigger = trigger.or_else(|| synchronized_trigger(state, &owner, with_oil));
        let visible_flames = owner.cpus[0]
            .latest_observation_for_test()
            .map_or(0, |observation| observation.parts().flames.len());
        max_visible_flames = max_visible_flames.max(visible_flames);
        if trigger
            .as_ref()
            .is_some_and(|value| tick > value.detonates_at)
            && visible_flames == 0
            && flames_clear_after_trigger.is_none()
        {
            flames_clear_after_trigger = Some(tick);
        }
        for (index, seat) in CPUS.into_iter().enumerate() {
            if !state.actor(seat).unwrap().alive() && first_deaths[index].is_none() {
                first_deaths[index] = Some(tick);
            }
            let controller = owner.cpus[index].controller_state_for_test();
            let flags = flags(&controller);
            if prior_flags[index] != Some(flags) {
                transitions.push(ControllerTransition {
                    seat,
                    tick,
                    flags,
                    state: controller,
                });
                prior_flags[index] = Some(flags);
            }
        }
    }
    let final_tick = owner.runtime.state_for_test().unwrap().tick();
    let final_phase = owner.runtime.lifecycle().phase();
    Outcome {
        with_oil,
        initial_blocks,
        initial_positions,
        trigger,
        first_deaths,
        max_visible_flames,
        flames_clear_after_trigger,
        final_tick,
        final_phase,
        queued_bases,
        recovery_bases,
        transitions,
        worker,
        admissions: std::array::from_fn(|index| owner.cpus[index].admission_trace_for_test()),
    }
}

fn capture_diagnostics(
    basis: PlanningBasis<'_>,
    with_oil: bool,
    captures: &mut Vec<QueuedBasis>,
    recovery: &mut Vec<QueuedBasis>,
) {
    let seat = basis.observation.parts().recipient;
    let need_first = !captures.iter().any(|capture| capture.seat == seat);
    let need_recovery = !recovery.iter().any(|capture| capture.seat == seat);
    if !need_first && !need_recovery {
        return;
    }
    let Some(captured) = capture(basis, with_oil) else {
        return;
    };
    if need_first {
        captures.push(captured.clone());
    }
    if need_recovery && captured.selector.result != SelectorResult::None {
        recovery.push(captured);
    }
}

fn synchronized_trigger(
    state: &State,
    owner: &super::driver::Driver,
    with_oil: bool,
) -> Option<Trigger> {
    if CPUS
        .iter()
        .any(|&seat| state.actor(seat).unwrap().bomb_range() != 3)
    {
        return None;
    }
    let bombs = state.world().bombs().values().copied().collect::<Vec<_>>();
    if bombs.len() != 3 || bombs.iter().any(|bomb| bomb.range() != 3) {
        return None;
    }
    let placed_at = bombs[0].placed_at();
    let detonates_at = bombs[0].detonates_at()?;
    if bombs
        .iter()
        .any(|bomb| bomb.placed_at() != placed_at || bomb.detonates_at() != Some(detonates_at))
    {
        return None;
    }
    let observation = owner.cpus[0].latest_observation_for_test()?;
    let oil = observation
        .parts()
        .oil
        .iter()
        .map(|oil| oil.cell)
        .collect::<Vec<_>>();
    if oil.len() != usize::from(with_oil) * 2 {
        return None;
    }
    Some(Trigger {
        placed_at,
        detonates_at,
        cells: std::array::from_fn(|index| bombs[index].cell()),
        oil,
    })
}

fn flags(state: &str) -> ControllerFlags {
    ControllerFlags {
        escaping: state.contains(" escaping=true"),
        origin: state.contains(" origin=true"),
        pulse: state.contains(" pulse=true"),
        route: state.contains(" route=true"),
        continuation: state.contains(" continuation=true"),
        fence: state.contains(" fence=true"),
    }
}

fn scene_with_oil(basis: &State) -> State {
    controlled_scene(basis, true)
}

fn scene_without_oil(basis: &State) -> State {
    controlled_scene(basis, false)
}

fn controlled_scene(basis: &State, with_oil: bool) -> State {
    let fixed = fixed_arena();
    let terrain = std::array::from_fn(|index| {
        fixed
            .terrain(CellId::new((index % 15) as u16, (index / 15) as u16))
            .unwrap()
    });
    #[rustfmt::skip]
    let blocks = [(3,1),(4,1),(5,1),(9,1),(10,1),(11,1),(3,2),(5,2),(7,2),(9,2),(11,2),
        (1,3),(2,3),(3,3),(4,3),(5,3),(6,3),(8,3),(9,3),(10,3),(11,3),(12,3),(13,3),
        (1,4),(3,4),(5,4),(7,4),(9,4),(11,4),(13,4),(1,5),(2,5),(3,5),(5,5),(6,5),
        (8,5),(9,5),(11,5),(12,5),(13,5),(7,6),(1,7),(2,7),(3,7),(5,7),(6,7),(8,7),
        (9,7),(11,7),(12,7),(13,7),(1,8),(3,8),(5,8),(7,8),(9,8),(11,8),(13,8),(1,9),
        (2,9),(3,9),(4,9),(5,9),(6,9),(8,9),(9,9),(10,9),(11,9),(12,9),(13,9),(3,10),
        (5,10),(7,10),(9,10),(11,10),(3,11),(4,11),(5,11),(9,11),(10,11),(11,11)]
        .into_iter().map(|(column,row)| CellId::new(column,row)).collect::<BTreeSet<_>>();
    let arena = Arena::new(
        terrain,
        blocks,
        [
            CellId::new(1, 1),
            CellId::new(13, 1),
            CellId::new(1, 11),
            CellId::new(13, 11),
        ],
    )
    .unwrap();
    let mut concealed = BTreeMap::from([
        (CellId::new(11, 1), PickupKind::BlastRange),
        (CellId::new(3, 11), PickupKind::BlastRange),
        (CellId::new(11, 11), PickupKind::BlastRange),
    ]);
    if with_oil {
        concealed.insert(CellId::new(10, 1), PickupKind::OilBarrel);
        concealed.insert(CellId::new(13, 8), PickupKind::OilBarrel);
    }
    let world = World::with_pickups(arena, concealed, BTreeMap::new(), BTreeMap::new()).unwrap();
    let actors = [
        actor(1, SeatId::One, 1_536, 1_536),
        actor(2, SeatId::Two, 13_824, 1_536),
        actor(3, SeatId::Three, 1_536, 11_776),
        actor(4, SeatId::Four, 13_824, 11_776),
    ];
    State::new_round(
        *basis.rules(),
        world,
        actors,
        basis.tick(),
        basis.round_assignment(),
        Some(SimulationSeed::new([47; 32])),
    )
    .unwrap()
}

fn actor(id: u32, seat: SeatId, x: i32, y: i32) -> RoundActorInit {
    RoundActorInit {
        actor: ActorInit {
            id: EntityId(id),
            seat,
            position: Position { x, y },
            heading: Direction::East,
            speed: grenadiers_core::game::SpeedLevel::Base,
        },
        participating: true,
    }
}

fn merge(total: &mut PumpReport, next: PumpReport) {
    total.dequeued += next.dequeued;
    total.delivered += next.delivered;
    total.cancelled += next.cancelled;
    total.failed += next.failed;
    total.dropped += next.dropped;
    total.work.extend(next.work);
    total.planned_intents.extend(next.planned_intents);
    total.planner_times.extend(next.planner_times);
}

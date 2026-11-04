use std::{collections::BTreeSet, sync::Arc};

#[path = "layout_episode_trace_profile.rs"]
mod profile;
#[path = "layout_episode_runtime.rs"]
mod runtime_support;
#[path = "layout_episode_witness.rs"]
pub(super) mod witness;
pub(super) use profile::{run_zero_bomber_trace, run_zero_bomber_trace_with_profile};
use witness::{
    PlanSnapshot, StationaryPreflight, StepReplayWitness, death_witness, digest, merge,
    record_events, trace_identity, update_lifecycles,
};

use grenadiers_core::{
    game::{
        ArenaConstructionProfile, BombTrigger, BombView, CellId, EffectKind, EntityId, InputAction,
        MobView, SeatId, SpikePitView, StepReport, StormCloudView, Tick, World,
    },
    room::{BoardSize, CpuProfile, MatchPhase, SpikePitSettings},
};
use tokio::{sync::watch, time::Instant};

use super::super::{RosterOwnerNotice, driver::Driver, validate_cpus};
use crate::game::rooms::{
    RandomFill,
    cpu::{ContinuationTrace, CpuController, CpuPlanner, PumpReport, RouteTrace},
    roster_runtime::RosterAction,
};

const CPU_SEATS: [SeatId; 3] = [SeatId::Two, SeatId::Three, SeatId::Four];
const EFFECTS: [EffectKind; 11] = [
    EffectKind::Shield,
    EffectKind::Stopwatch,
    EffectKind::InvisibilityCloak,
    EffectKind::GoldenFlame,
    EffectKind::ShortFuse,
    EffectKind::Diarrhea,
    EffectKind::SlowFoot,
    EffectKind::ShortFlame,
    EffectKind::LongFuse,
    EffectKind::ReverseControls,
    EffectKind::InvisibleBombs,
];

#[derive(Clone, Copy, Debug)]
pub(super) struct LayoutCase {
    pub label: &'static str,
    pub board: BoardSize,
    pub lairs: bool,
    pub seed: u8,
    pub profile: ArenaConstructionProfile,
    pub pit_settings: SpikePitSettings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct BombLifecycle {
    pub id: EntityId,
    pub owner: EntityId,
    pub cell: CellId,
    pub trigger: BombTrigger,
    pub placed_at: Tick,
    pub detonated_at: Option<Tick>,
    pub all_finite_flames_clear_at: Option<Tick>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DynamicFrame {
    pub tick: Tick,
    pub actors: Vec<grenadiers_core::game::ActorView>,
    pub bombs: Vec<BombView>,
    pub(super) blocks: Vec<CellId>,
    pub(super) flames: Vec<grenadiers_core::game::FlameView>,
    oil: Vec<grenadiers_core::game::OilBarrelView>,
    pickups: Vec<grenadiers_core::game::PickupView>,
    cloud: Option<StormCloudView>,
    pits: Option<Vec<SpikePitView>>,
    mobs: Vec<MobView>,
    pub accepted_inputs: Vec<(SeatId, InputAction)>,
    pub continuation_authority: [bool; 3],
    pub report: StepReport,
    observation_digests: [Option<u64>; 3],
    controller_basis_ticks: [Option<Tick>; 3],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DeathWitness {
    pub tick: Tick,
    pub phase: MatchPhase,
    pub eliminated: Vec<SeatId>,
    pub positions: Vec<grenadiers_core::game::Position>,
    pub effects: Vec<Vec<EffectKind>>,
    pub pits: Option<Vec<SpikePitView>>,
    pub mobs: Vec<MobView>,
    pub fire: Vec<CellId>,
    pub report: StepReport,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LayoutTrace {
    pub case: LayoutCase,
    pub initial_world: World,
    pub initial_actors: Vec<grenadiers_core::game::Actor>,
    pub initial_pits: Option<Vec<SpikePitView>>,
    pub initial_cloud: Option<StormCloudView>,
    pub frames: Vec<DynamicFrame>,
    pub event_placements: [usize; 3],
    pub post_step_remaining_counter: [usize; 3],
    pub first_bombs: [Option<BombLifecycle>; 3],
    pub first_death: Option<DeathWitness>,
    pub pump: PumpReport,
    pub plan_witnesses: Vec<PlanSnapshot>,
    pub route_traces: [Vec<RouteTrace>; 3],
    pub continuation_traces: [ContinuationTrace; 3],
    pub stationary_preflight: Vec<StationaryPreflight>,
    pub step_replay: Option<StepReplayWitness>,
}

impl PartialEq for LayoutCase {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
            && self.board == other.board
            && self.lairs == other.lairs
            && self.seed == other.seed
            && self.profile == other.profile
            && self.pit_settings == other.pit_settings
    }
}

impl Eq for LayoutCase {}

pub(super) fn run_layout_trace(case: LayoutCase, ticks: usize) -> LayoutTrace {
    run_layout_trace_with_profile(case, ticks, CpuProfile::HARD_V61)
}

pub(super) fn run_layout_trace_with_profile(
    case: LayoutCase,
    ticks: usize,
    profile: CpuProfile,
) -> LayoutTrace {
    run_layout_trace_inner(
        case,
        ticks,
        profile,
        Some((SeatId::Four, Tick(300), Tick(541))),
        false,
    )
}

fn run_layout_trace_inner(
    case: LayoutCase,
    ticks: usize,
    profile: CpuProfile,
    capture: Option<(SeatId, Tick, Tick)>,
    matrix_identity: bool,
) -> LayoutTrace {
    let identity = trace_identity(matrix_identity);
    let mut runtime = runtime_support::runtime(case, &identity);
    let admission = runtime.prepare_join(SeatId::One).unwrap();
    runtime.commit_admission(admission);
    runtime
        .action(
            SeatId::One,
            1,
            RosterAction::Ready {
                ready: true,
                settings_revision: 1,
            },
        )
        .unwrap();
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    let planner = CpuPlanner::without_workers();
    let cpus = CPU_SEATS
        .map(|seat| {
            CpuController::new(
                planner.clone(),
                seat,
                identity.instance,
                [case.seed.wrapping_add(seat.index() as u8); 32],
                profile,
            )
        })
        .into_iter()
        .collect::<Vec<_>>();
    if profile == CpuProfile::HARD_V64 {
        assert_eq!(validate_cpus(&runtime, &cpus), Ok(()));
    } else if profile == CpuProfile::HARD_V63 {
        assert_eq!(
            validate_cpus(&runtime, &cpus),
            Err(crate::game::rooms::roster_runtime::RosterRuntimeError::UnsupportedConfiguration)
        );
    }
    let notice = runtime.current_notice().unwrap();
    let (notices, receiver) = watch::channel(RosterOwnerNotice::Publication(notice.clone()));
    drop(receiver);
    let random_fill: Arc<RandomFill> = Arc::new(|bytes| {
        bytes.fill(91);
        Ok(())
    });
    let mut driver = Driver::new(
        runtime,
        notice.clone(),
        notices,
        Instant::now() + std::time::Duration::from_secs(300),
        random_fill,
        cpus,
    );
    driver.publish(notice);
    let initial = driver.runtime.replay_inspection();
    let state = &initial.simulation.as_ref().unwrap().state;
    let initial_world = state.world().clone();
    let initial_actors = state.actors().to_vec();
    let initial_pits = state.spike_pits();
    let initial_cloud = state.storm_cloud();
    let actor_ids = CPU_SEATS.map(|seat| state.actors()[seat.index()].id());
    let mut known = state
        .world()
        .bombs()
        .keys()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut event_placements = [0; 3];
    let mut post_step_remaining_counter = [0; 3];
    let mut first_bombs = [None; 3];
    let mut first_death = None;
    let mut frames = Vec::with_capacity(ticks);
    let mut pump = PumpReport::default();
    let witness = case.profile == ArenaConstructionProfile::LairClearanceV5
        && case.board == BoardSize::ThirteenByEleven
        && case.lairs
        && case.pit_settings == SpikePitSettings::new(60, 300).unwrap();
    let mut plan_witnesses = Vec::new();
    let mut step_replay = None;
    let mut stationary_preflight = Vec::new();
    for _ in 0..ticks {
        let before = witness
            .then(|| driver.runtime.state_for_test().cloned())
            .flatten()
            .filter(|state| state.tick() == Tick(510));
        let pumped = if let Some((seat, start, end)) = capture {
            planner.pump_for_test_witnessing(usize::MAX, |result| {
                if result.basis.observation.parts().recipient == seat
                    && (start..=end).contains(&result.basis.observation.parts().tick)
                {
                    let demolition_enabled = witness::demolition_enabled(&plan_witnesses);
                    plan_witnesses.push(PlanSnapshot::capture(result, demolition_enabled));
                }
            })
        } else {
            planner.pump_for_test(usize::MAX)
        };
        merge(&mut pump, pumped);
        driver.tick(Instant::now()).unwrap();
        let inspection = driver.runtime.replay_inspection();
        let state = &inspection.simulation.as_ref().unwrap().state;
        let (inputs, report) = driver.runtime.last_step_trace_for_test().unwrap();
        if let Some(before) = before {
            step_replay = Some(StepReplayWitness::prove(before, inputs, report, state));
        }
        record_events(report, &actor_ids, &mut event_placements, &mut first_bombs);
        for (&id, bomb) in state.world().bombs() {
            if known.insert(id)
                && let Some(index) = actor_ids.iter().position(|owner| *owner == bomb.owner())
            {
                post_step_remaining_counter[index] += 1;
            }
        }
        update_lifecycles(report, state, &mut first_bombs);
        let public = state.public_view();
        let basis = std::array::from_fn(|index| {
            driver.cpus[index]
                .latest_observation_for_test()
                .map(|observation| (digest(&observation), observation.parts().tick))
        });
        if witness
            && (Tick(399)..Tick(542)).contains(&state.tick())
            && let Some(observation) = driver.cpus[2].latest_observation_for_test()
            && observation.parts().tick == state.tick()
        {
            let short = Tick(state.tick().0.saturating_add(14).min(542));
            for safe_until in [Tick(542), short] {
                for hold_ticks in [0, 3] {
                    let mut work = grenadiers_core::cpu::WorkBudget::new(8_192);
                    let outcome = grenadiers_core::cpu::survival::stationary_suffix_preflight(
                        &observation,
                        safe_until,
                        hold_ticks,
                        &mut work,
                    );
                    stationary_preflight.push(StationaryPreflight {
                        publication: state.tick(),
                        safe_until,
                        hold_ticks,
                        spent: work.spent(),
                        outcome,
                    });
                }
            }
        }
        let frame = DynamicFrame {
            tick: state.tick(),
            actors: public.actors,
            bombs: public.bombs,
            blocks: public.arena.blocks,
            flames: public.flames,
            oil: public.oil_barrels,
            pickups: public.pickups,
            cloud: state.storm_cloud(),
            pits: state.spike_pits(),
            mobs: state.mobs(),
            accepted_inputs: inputs
                .iter()
                .map(|input| (input.seat, input.action))
                .collect(),
            continuation_authority: std::array::from_fn(|index| {
                driver.cpus[index].has_continuation_authority_for_test()
            }),
            report: report.clone(),
            observation_digests: basis.map(|value| value.map(|pair| pair.0)),
            controller_basis_ticks: basis.map(|value| value.map(|pair| pair.1)),
        };
        if first_death.is_none() {
            first_death = death_witness(state, inspection.lifecycle.phase(), report);
        }
        frames.push(frame);
        if state.result().is_some() {
            break;
        }
    }
    merge(&mut pump, planner.pump_for_test(usize::MAX));
    let route_traces = std::array::from_fn(|index| {
        if witness {
            driver.cpus[index].route_trace_for_test().to_vec()
        } else {
            Vec::new()
        }
    });
    let continuation_traces =
        std::array::from_fn(|index| driver.cpus[index].continuation_trace_for_test());
    LayoutTrace {
        case,
        initial_world,
        initial_actors,
        initial_pits,
        initial_cloud,
        frames,
        event_placements,
        post_step_remaining_counter,
        first_bombs,
        first_death,
        pump,
        plan_witnesses,
        route_traces,
        continuation_traces,
        stationary_preflight,
        step_replay,
    }
}

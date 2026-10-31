use std::collections::BTreeSet;

use crate::cpu::forecast::TickWindow;
use crate::cpu::{
    BombKnowledge, MovementIntent, Observation, ObservedActor, ObservedBomb, ObservedBombTrigger,
    ObservedEnvironment, ObservedPickup, UnknownBombFacts, WorkBudget,
};
use crate::game::{
    BlastKind, CellId, Direction, EntityId, PickupKind, Position, SpikePitPhase, StormCloudConfig,
    StormCloudView, Tick,
};

use super::motion::advance;
use super::route::{SearchNode, initial_lanes};
use super::route_checks::{CollisionProofs, check_transition_all, one_tick};
use super::route_expand::terminal;
use super::validation::{Context, validate};
use super::{RouteFailure, RouteLimits, RouteMode, RouteRequest};

#[derive(Clone, Copy)]
enum TerminalHazard {
    None,
    Pickup,
    FutureBomb,
}

fn storm_observation(
    phase: SpikePitPhase,
    remaining: u64,
    shield: Option<u16>,
    grace: u8,
    actor_on_pit: bool,
    hazard: TerminalHazard,
) -> Observation {
    let (base, pit) = super::spike_pits::tests::observed(phase, remaining, shield, grace, false);
    let pits = base.spike_pits().cloned();
    let lairs = base.respawn_lairs().cloned();
    let mut parts = base.into_parts();
    let recipient = parts.recipient;
    let actor = parts
        .actors
        .iter_mut()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == recipient => Some(actor),
            _ => None,
        })
        .unwrap();
    if !actor_on_pit {
        actor.position = Position::center(CellId::new(2, 1));
    }
    let actor_cell = actor.position.cell().unwrap();
    match hazard {
        TerminalHazard::None => {}
        TerminalHazard::Pickup => parts.pickups.push(ObservedPickup {
            cell: actor_cell,
            kind: PickupKind::MovementSpeed,
        }),
        TerminalHazard::FutureBomb => {
            parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
                id: EntityId(90),
                placed_at: Tick(0),
                cell: CellId::new(actor_cell.column + 1, actor_cell.row),
                owner: EntityId(1),
                range: 2,
                trigger: ObservedBombTrigger::Timed {
                    remaining_ticks: 10,
                    paused: false,
                },
                blast: BlastKind::Standard,
                unknown: UnknownBombFacts::ALL,
            }]);
        }
    }
    let actor_count = parts.actors.len();
    let observed = Observation::new_rules_v10(
        parts,
        ObservedEnvironment::new(
            StormCloudConfig::Single,
            Some(StormCloudView {
                position: Position::center(CellId::new(7, 6)),
                direction: Direction::East,
                leg_remaining_ticks: 128,
            }),
            vec![Some(0); actor_count],
        ),
        lairs,
        pits,
    )
    .unwrap();
    if actor_on_pit {
        let recipient = observed.parts().recipient;
        assert!(observed.parts().actors.iter().any(|actor| {
            matches!(actor, ObservedActor::Visible(actor)
                if actor.seat == recipient && actor.position == Position::center(pit))
        }));
    }
    observed
}

fn request(end: u64) -> RouteRequest {
    RouteRequest {
        start_window: TickWindow {
            start: Tick(1),
            end: Tick(2),
        },
        settled_stop_tick: Tick(0),
        survive_until: Tick(end),
        mode: RouteMode::CertifiedOnly,
        limits: RouteLimits::default(),
    }
}

fn prepared<'a>(
    observation: &'a Observation,
    forecast: &'a crate::cpu::forecast::HazardForecast,
    request: RouteRequest,
    work: &mut WorkBudget,
) -> (Context<'a>, SearchNode) {
    let context = validate(
        observation,
        forecast,
        request.start_window,
        request.survive_until,
    )
    .unwrap();
    let (lanes, assumptions) =
        initial_lanes(&context, &[forecast], request, None, None, work).unwrap();
    (
        context,
        SearchNode {
            steps: Vec::new(),
            lanes,
            assumptions,
        },
    )
}

fn per_tick_terminal(
    context: &Context<'_>,
    forecast: &crate::cpu::forecast::HazardForecast,
    request: RouteRequest,
    node: &mut SearchNode,
    work: &mut WorkBudget,
) -> Result<bool, RouteFailure> {
    let mut assumptions = BTreeSet::new();
    for lane in &mut node.lanes {
        let mut pits = lane.pits.clone();
        let mut simulation = lane.motion.clone();
        for tick in lane.ready.0..request.survive_until.0 {
            super::storm::charge_tick(context.storm, work)?;
            let tick = Tick(tick);
            let segment = advance(context, &mut simulation, MovementIntent::Stop, tick)?;
            match pits.advance(tick, segment[0].start(), &segment, work) {
                Ok(()) => {}
                Err(RouteFailure::NoRoute) => return Ok(false),
                Err(error) => return Err(error),
            }
            match check_transition_all(
                context,
                &[forecast],
                &segment,
                one_tick(tick)?,
                request.mode,
                CollisionProofs {
                    placement_origin: lane.origin_grace,
                    recipient_barrier: lane.recipient_barrier,
                },
                work,
            ) {
                Ok(found) => assumptions.extend(found),
                Err(RouteFailure::NoRoute) => return Ok(false),
                Err(error) => return Err(error),
            }
        }
    }
    node.assumptions.extend(assumptions);
    Ok(true)
}

#[test]
fn rules10_storm_terminal_matches_per_tick_pit_oracle_with_bounded_work() {
    for (phase, remaining, shield, grace, on_pit, hazard, expected) in [
        (
            SpikePitPhase::Idle,
            0,
            None,
            0,
            false,
            TerminalHazard::None,
            true,
        ),
        (
            SpikePitPhase::Idle,
            0,
            None,
            0,
            true,
            TerminalHazard::None,
            false,
        ),
        (
            SpikePitPhase::Warning,
            1,
            None,
            0,
            true,
            TerminalHazard::None,
            false,
        ),
        (
            SpikePitPhase::Raised,
            60,
            Some(2),
            0,
            true,
            TerminalHazard::None,
            false,
        ),
        (
            SpikePitPhase::Raised,
            60,
            None,
            2,
            true,
            TerminalHazard::None,
            false,
        ),
        (
            SpikePitPhase::Idle,
            0,
            None,
            0,
            false,
            TerminalHazard::Pickup,
            false,
        ),
        (
            SpikePitPhase::Idle,
            0,
            None,
            0,
            false,
            TerminalHazard::FutureBomb,
            false,
        ),
    ] {
        let observation = storm_observation(phase, remaining, shield, grace, on_pit, hazard);
        let forecast = super::tests::forecast_for(&observation);
        let request = request(61);
        let (context, initial) = prepared(
            &observation,
            &forecast,
            request,
            &mut WorkBudget::new(65_536),
        );
        let mut optimized = initial.clone();
        let mut oracle = initial;
        let mut optimized_work = WorkBudget::new(65_536);
        let mut oracle_work = WorkBudget::new(65_536);
        assert_eq!(
            terminal(
                &context,
                &[&forecast],
                request,
                &mut optimized,
                &mut optimized_work,
            )
            .unwrap(),
            expected,
        );
        assert_eq!(
            per_tick_terminal(&context, &forecast, request, &mut oracle, &mut oracle_work,)
                .unwrap(),
            expected,
        );
        if expected {
            assert_eq!(optimized.assumptions, oracle.assumptions);
            assert!(optimized_work.spent() < oracle_work.spent());
        }
    }
}

#[test]
fn rules10_storm_terminal_preserves_empty_boundary_and_replay_guards() {
    let observation =
        storm_observation(SpikePitPhase::Idle, 0, None, 0, false, TerminalHazard::None);
    let forecast = super::tests::forecast_for(&observation);
    let empty = request(1);
    let (context, mut empty_node) =
        prepared(&observation, &forecast, empty, &mut WorkBudget::new(65_536));
    assert!(
        terminal(
            &context,
            &[&forecast],
            empty,
            &mut empty_node,
            &mut WorkBudget::new(65_536),
        )
        .unwrap()
    );

    let certificate = super::route::plan_respawn_route(
        &observation,
        &forecast,
        request(61),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    let lane = certificate.lane(Tick(1)).unwrap();
    assert!(lane.storm_deadlines_at(Tick(60)).is_some());
    assert!(lane.spike_pit_guard_at(Tick(60)).is_some());
}

#[test]
fn archived_storm_terminal_keeps_per_tick_work_accounting() {
    let actor_position = Position::center(CellId::new(2, 1));
    let observation = super::storm_tests::custom_environment_observation(
        StormCloudView {
            position: Position::center(CellId::new(7, 6)),
            direction: Direction::East,
            leg_remaining_ticks: 128,
        },
        actor_position,
        0,
    );
    let forecast = super::tests::forecast_for(&observation);
    let request = request(61);
    let (context, initial) = prepared(
        &observation,
        &forecast,
        request,
        &mut WorkBudget::new(65_536),
    );
    let mut actual = initial.clone();
    let mut oracle = initial;
    let mut actual_work = WorkBudget::new(65_536);
    let mut oracle_work = WorkBudget::new(65_536);
    assert_eq!(
        terminal(
            &context,
            &[&forecast],
            request,
            &mut actual,
            &mut actual_work,
        ),
        per_tick_terminal(&context, &forecast, request, &mut oracle, &mut oracle_work,),
    );
    assert_eq!(actual.assumptions, oracle.assumptions);
    assert_eq!(actual_work.spent(), oracle_work.spent());
}

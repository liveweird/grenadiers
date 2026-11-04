#[path = "layout_v64_demolition_tests.rs"]
mod demolition;
#[path = "layout_recovery_horizon_tests.rs"]
mod horizon;
#[path = "layout_v65_reactive_tests.rs"]
mod reactive;
#[path = "layout_episode_trace.rs"]
mod trace;
#[path = "layout_zero_bomber_tests.rs"]
mod zero_bomber;

use grenadiers_core::{
    game::{ArenaConstructionProfile, CellId, SeatId, StepEventKind, Terrain, Tick},
    room::{BoardSize, SpikePitSettings},
};

use trace::{LayoutCase, LayoutTrace, run_layout_trace};

const UPDATES: usize = 840;

fn pit_settings() -> [(&'static str, SpikePitSettings); 2] {
    [
        ("pit-36-180", SpikePitSettings::new(36, 180).unwrap()),
        ("pit-60-300", SpikePitSettings::new(60, 300).unwrap()),
    ]
}

#[test]
fn own_blast_execution_witness_replays_tick_511_and_names_the_route_boundary() {
    let result = run_layout_trace(
        LayoutCase {
            label: "own-blast-witness",
            board: BoardSize::ThirteenByEleven,
            lairs: true,
            seed: 73,
            profile: ArenaConstructionProfile::LairClearanceV5,
            pit_settings: SpikePitSettings::new(60, 300).unwrap(),
        },
        UPDATES,
    );
    let replay = result.step_replay.as_ref().expect("tick 510/511 witness");
    assert_eq!(replay.before.tick(), Tick(510));
    assert_eq!(replay.after.tick(), Tick(511));
    let owner = replay.before.actors()[SeatId::Four.index()].id();
    let bomb = replay
        .before
        .world()
        .bombs()
        .values()
        .find(|bomb| bomb.owner() == owner)
        .copied()
        .expect("Seat Four live bomb");
    assert_eq!(bomb.id().0, 5);
    assert_eq!(bomb.cell(), CellId::new(7, 7));
    assert_eq!(bomb.range(), 2);
    let target = replay.after.actors()[SeatId::Four.index()]
        .position()
        .cell()
        .unwrap();
    assert_eq!(target, CellId::new(7, 5));
    for cell in [CellId::new(7, 6), target] {
        assert_eq!(
            replay.before.world().arena().terrain(cell),
            Some(Terrain::Floor)
        );
        assert!(!replay.before.world().arena().blocks().contains(&cell));
    }
    assert!(replay.report.events.iter().any(
        |event| matches!(event.kind, StepEventKind::BombDetonated(view) if view.id == bomb.id())
    ));
    assert!(replay.report.events.iter().any(|event| matches!(
        event.kind,
        StepEventKind::ActorEliminated {
            seat: SeatId::Four,
            ..
        }
    )));
    let planned = result
        .plan_witnesses
        .iter()
        .find(|witness| {
            witness
                .plan
                .as_ref()
                .and_then(|plan| plan.execution.as_ref())
                .and_then(|execution| execution.plant())
                .is_some_and(|plant| {
                    plant.placement().placed_at == Tick(331)
                        && plant.placement().cell == CellId::new(7, 7)
                })
        })
        .expect("worker result for Seat Four bomb 5");
    let plan = planned.plan.as_ref().unwrap();
    let execution = plan.execution.as_ref().unwrap();
    let route = execution.route();
    let lane = route.lane(Tick(331)).expect("admitted lane");
    let poses = (planned.observation.parts().tick.0..route.survive_until().0)
        .filter_map(|tick| lane.pose_at(Tick(tick)).map(|pose| (Tick(tick), pose)))
        .collect::<Vec<_>>();
    let placement_frame = result
        .frames
        .iter()
        .find(|frame| frame.tick == Tick(331))
        .expect("placement frame");
    assert!(
        placement_frame.accepted_inputs.iter().any(|input| {
            *input == (SeatId::Four, grenadiers_core::game::InputAction::PlaceBomb)
        })
    );
    let invalidation_frame = result
        .frames
        .iter()
        .find(|frame| frame.tick == Tick(333))
        .expect("first invalidation frame");
    let actual_actor = invalidation_frame
        .actors
        .iter()
        .find(|actor| actor.seat == SeatId::Four)
        .unwrap();
    assert_eq!(
        (actual_actor.position, actual_actor.heading),
        lane.pose_at(Tick(333)).unwrap(),
        "the certified actor pose remains exact at invalidation"
    );
    assert!(invalidation_frame.bombs.iter().any(|actual| {
        actual.owner == owner
            && actual.cell == execution.plant().unwrap().placement().cell
            && actual.placed_at == execution.plant().unwrap().placement().placed_at
    }));
    let foreign = invalidation_frame
        .bombs
        .iter()
        .find(|actual| actual.owner != owner)
        .copied()
        .expect("simultaneous foreign plant");
    let recovery_plans = result
        .plan_witnesses
        .iter()
        .filter(|witness| witness.observation.parts().tick >= Tick(333))
        .map(|witness| {
            (
                witness.observation.parts().tick,
                witness.settled_stop_tick,
                witness.escape_origin.is_some(),
                witness.work_spent,
                witness.plan.as_ref().map(|plan| {
                    (
                        plan.control.intent,
                        plan.control.hold_ticks,
                        plan.lease.is_some(),
                        plan.execution.is_some(),
                    )
                }),
                witness.full_proof.spent,
                witness.full_proof.plan.as_ref().map(|plan| {
                    (
                        plan.control.intent,
                        plan.control.hold_ticks,
                        plan.lease.is_some(),
                        plan.execution.is_some(),
                    )
                }),
                witness.full_proof.error,
                witness.corrected_recovery.clone(),
            )
        })
        .collect::<Vec<_>>();
    let refusal_probes = [Tick(492), Tick(495)].map(|tick| {
        let basis = result
            .plan_witnesses
            .iter()
            .find(|witness| witness.observation.parts().tick == tick)
            .expect("stopped recovery basis");
        (tick, trace::witness::pulse_refusal_probes(basis))
    });
    let refusal_frame = result
        .frames
        .iter()
        .find(|frame| frame.tick == Tick(495))
        .expect("first uncertified stop frame");
    let refusal_actor = refusal_frame
        .actors
        .iter()
        .find(|actor| actor.seat == SeatId::Four)
        .unwrap();
    let arena = replay.before.world().arena();
    let refusal_neighborhood = [
        CellId::new(7, 4),
        CellId::new(8, 5),
        CellId::new(7, 6),
        CellId::new(6, 5),
    ]
    .map(|cell| (cell, arena.terrain(cell), arena.blocks().contains(&cell)));
    let mut recovery_motion = Vec::new();
    let mut previous = None;
    for frame in result
        .frames
        .iter()
        .filter(|frame| (Tick(333)..=Tick(511)).contains(&frame.tick))
    {
        let actor = frame
            .actors
            .iter()
            .find(|actor| actor.seat == SeatId::Four)
            .unwrap();
        let pose = (actor.position, actor.heading);
        if previous != Some(pose) || !frame.accepted_inputs.is_empty() {
            recovery_motion.push((frame.tick, pose, frame.accepted_inputs.clone()));
        }
        previous = Some(pose);
    }
    eprintln!(
        "{}",
        serde_json::json!({
            "basisTick": planned.observation.parts().tick.0,
            "policy": format!("{:?}", planned.policy),
            "memory": format!("{:?}", planned.memory),
            "hazardMemory": format!("{:?}", planned.hazard_memory),
            "settledStop": planned.settled_stop_tick.0,
            "rngKey": format!("{:?}", planned.rng_key),
            "workSpent": planned.work_spent,
            "rngDrawsBefore": planned.rng_draws_before,
            "rngDrawsAfter": planned.rng_draws_after,
            "hasContact": planned.has_contact,
            "hasEscapeOrigin": planned.escape_origin.is_some(),
            "control": format!("{:?}", plan.control),
            "placement": format!("{:?}", execution.plant().unwrap().placement()),
            "routeStart": format!("{:?}", route.start_window()),
            "surviveUntil": route.survive_until().0,
            "poses": format!("{poses:?}"),
            "routeTrace": format!("{:?}", result.route_traces[2]),
            "invalidationPose": format!("{:?}", (actual_actor.position, actual_actor.heading)),
            "invalidationBombs": format!("{:?}", invalidation_frame.bombs),
            "foreignBomb": format!("{:?}", foreign),
            "recoveryPlans": format!("{:?}", recovery_plans),
            "recoveryMotion": format!("{:?}", recovery_motion),
            "refusalProbes": format!("{:?}", refusal_probes),
            "refusalActor": format!("{:?}", refusal_actor),
            "refusalNeighborhood": format!("{:?}", refusal_neighborhood),
            "placementInputs": format!("{:?}", placement_frame.accepted_inputs),
            "tick511Inputs": format!("{:?}", replay.inputs),
            "tick511Report": format!("{:?}", replay.report),
        })
    );
}

fn run_pair(
    board: BoardSize,
    lairs: bool,
    seed: u8,
    pit_label: &'static str,
    pits: SpikePitSettings,
) -> (LayoutTrace, LayoutTrace) {
    let reflected = run_layout_trace(
        LayoutCase {
            label: pit_label,
            board,
            lairs,
            seed,
            profile: ArenaConstructionProfile::ReflectedV3V4,
            pit_settings: pits,
        },
        UPDATES,
    );
    let clearance = run_layout_trace(
        LayoutCase {
            label: pit_label,
            board,
            lairs,
            seed,
            profile: ArenaConstructionProfile::LairClearanceV5,
            pit_settings: pits,
        },
        UPDATES,
    );
    (reflected, clearance)
}

#[test]
fn no_lair_profiles_have_identical_private_world_observations_and_trajectory() {
    for (label, pits) in pit_settings() {
        let (reflected, clearance) = run_pair(BoardSize::ThirteenByEleven, false, 73, label, pits);
        assert_eq!(reflected.initial_world, clearance.initial_world, "{label}");
        assert_eq!(
            reflected.initial_actors, clearance.initial_actors,
            "{label}"
        );
        assert_eq!(reflected.initial_pits, clearance.initial_pits, "{label}");
        assert_eq!(reflected.initial_cloud, clearance.initial_cloud, "{label}");
        assert_eq!(reflected.frames, clearance.frames, "{label}");
        assert_eq!(
            reflected.event_placements, clearance.event_placements,
            "{label}"
        );
        assert_eq!(
            reflected.post_step_remaining_counter, clearance.post_step_remaining_counter,
            "{label}"
        );
        assert_eq!(reflected.first_bombs, clearance.first_bombs, "{label}");
        assert_eq!(reflected.first_death, clearance.first_death, "{label}");
        assert_eq!(reflected.pump.dequeued, clearance.pump.dequeued, "{label}");
        assert_eq!(
            reflected.pump.delivered, clearance.pump.delivered,
            "{label}"
        );
        assert_eq!(
            reflected.pump.cancelled, clearance.pump.cancelled,
            "{label}"
        );
        assert_eq!(reflected.pump.failed, clearance.pump.failed, "{label}");
        assert_eq!(reflected.pump.dropped, clearance.pump.dropped, "{label}");
        assert_eq!(reflected.pump.work, clearance.pump.work, "{label}");
        assert_eq!(
            reflected.pump.planned_intents, clearance.pump.planned_intents,
            "{label}"
        );
        assert_eq!(
            reflected.pump.planned_intents_by_job, clearance.pump.planned_intents_by_job,
            "{label}"
        );
    }
}

#[test]
fn lair_clearance_matrix_reports_authoritative_placements_and_first_hazards() {
    for board in [BoardSize::ThirteenByEleven, BoardSize::FifteenByThirteen] {
        for (label, pits) in pit_settings() {
            let (reflected, clearance) = run_pair(board, true, 73, label, pits);
            for result in [&reflected, &clearance] {
                assert!(
                    result
                        .event_placements
                        .iter()
                        .zip(result.post_step_remaining_counter)
                        .all(|(events, residual)| *events >= residual),
                    "{result:?}"
                );
                eprintln!(
                    "{}",
                    serde_json::json!({
                    "case": result.case.label,
                    "profile": format!("{:?}", result.case.profile),
                    "board": format!("{:?}", result.case.board),
                    "lairs": result.case.lairs,
                    "seed": result.case.seed,
                    "pitWarning": result.case.pit_settings.warning_ticks(),
                    "pitRaised": result.case.pit_settings.raised_ticks(),
                    "updates": result.frames.len(),
                    "eventPlacements": result.event_placements,
                    "postStepRemainingCounter": result.post_step_remaining_counter,
                    "firstBombs": format!("{:?}", result.first_bombs),
                    "firstDeath": format!("{:?}", result.first_death),
                    "dequeued": result.pump.dequeued,
                    "delivered": result.pump.delivered,
                    "failed": result.pump.failed,
                    })
                );
            }
        }
    }
}

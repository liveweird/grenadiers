use grenadiers_core::{
    cpu::{BombKnowledge, ManualIntent},
    game::{
        ArenaConstructionProfile, CellId, EntityId, InputAction, SeatId, StepEventKind, Terrain,
        Tick,
    },
    room::{BoardSize, CpuDifficulty, CpuPolicy, CpuProfile, SpikePitSettings},
};

use super::trace::{LayoutCase, LayoutTrace, run_zero_bomber_trace_with_profile};
use crate::game::rooms::cpu::{CpuController, CpuPlanner};

#[path = "layout_v65_reactive_counterfactual.rs"]
mod counterfactual;
#[path = "layout_v65_reactive_support.rs"]
mod support;
use support::{
    assert_reactive_escape_recovers_revoked_plant, reactive_outcome_ledger,
    reactive_witness_survives,
};

const UPDATES: usize = 840;

#[test]
fn queued_hard65_uses_a_controlled_foreign_bomb_and_survives_its_useful_plant() {
    let case = LayoutCase {
        label: "controlled-foreign-bomb-gen7",
        board: BoardSize::ThirteenByEleven,
        lairs: true,
        seed: 73,
        profile: ArenaConstructionProfile::WeightedLairClearanceV7,
        pit_settings: SpikePitSettings::new(60, 300).unwrap(),
    };
    let archived =
        run_zero_bomber_trace_with_profile(case, UPDATES, SeatId::Two, CpuProfile::HARD_V64);
    let reactive =
        run_zero_bomber_trace_with_profile(case, UPDATES, SeatId::Two, CpuProfile::HARD_V65);

    assert_eq!(archived.initial_world, reactive.initial_world);
    assert_eq!(archived.initial_actors, reactive.initial_actors);
    assert_eq!(archived.initial_pits, reactive.initial_pits);
    assert_eq!(archived.initial_cloud, reactive.initial_cloud);
    assert!(
        controlled_bomb_was_accepted(&archived),
        "controlled frames={:#?}",
        archived
            .frames
            .iter()
            .filter(|frame| (Tick(180)..=Tick(182)).contains(&frame.tick))
            .collect::<Vec<_>>()
    );
    assert!(controlled_bomb_was_accepted(&reactive));
    let recovery = assert_reactive_escape_recovers_revoked_plant(&reactive);
    counterfactual::assert_unsafe_pose_has_directional_escape(recovery);
    eprintln!(
        "HARD65_REACTIVE_PLACEMENT_LEDGER {:#?}",
        reactive_outcome_ledger(&reactive)
    );

    let witness = reactive
        .plan_witnesses
        .iter()
        .find(|witness| {
            witness.plan.as_ref().is_some_and(|plan| {
                plan.execution.as_ref().is_some_and(|execution| {
                    execution.requires_reactive_support() && execution.plant().is_some()
                })
            }) && reactive_witness_survives(&reactive, witness)
        })
        .unwrap_or_else(|| {
            panic!(
                "Hard65 must have a surviving reactive plant; ledger={:#?} invalidations={:#?}",
                reactive_outcome_ledger(&reactive),
                reactive.route_traces[0]
                    .iter()
                    .filter(|entry| entry.invalidator.is_some())
                    .collect::<Vec<_>>(),
            )
        });
    let actor = witness
        .observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            grenadiers_core::cpu::ObservedActor::Visible(actor)
                if actor.seat == witness.observation.parts().recipient =>
            {
                Some(actor.id)
            }
            _ => None,
        })
        .unwrap();
    assert!(matches!(
        &witness.observation.parts().bombs,
        BombKnowledge::Visible(bombs) if bombs.iter().any(|bomb| bomb.owner != actor)
    ));
    assert!(witness.work_spent <= grenadiers_core::cpu::survival::WORK_ALLOWANCE);
    let plan = witness.plan.as_ref().unwrap();
    let execution = plan.execution.as_ref().unwrap();
    for profile in [
        CpuProfile::HARD_V64,
        CpuProfile::RECOVERY_V62_STANDARD,
        CpuProfile::RECOVERY_V62_EASY,
        CpuProfile {
            policy: CpuPolicy::UtilityV66,
            difficulty: CpuDifficulty::Standard,
        },
    ] {
        let controller = CpuController::new(
            CpuPlanner::without_workers(),
            SeatId::Two,
            4_028,
            [65; 32],
            profile,
        );
        assert!(!controller.supports_execution_certificate_for_test(execution));
    }
    for profile in [CpuProfile::HARD_V65, CpuProfile::HARD_V66] {
        let controller = CpuController::new(
            CpuPlanner::without_workers(),
            SeatId::Two,
            4_028,
            [65; 32],
            profile,
        );
        assert!(controller.supports_execution_certificate_for_test(execution));
    }
    assert_copied_reactive_certificate_is_rejected(&reactive, witness, plan);
    assert_eq!(plan.control.intent.manual, Some(ManualIntent::Bomb));
    let incoming = witness.memory.strategic_intent().unwrap();
    let retained = plan.control.next_memory.strategic_intent().unwrap();
    assert_eq!(retained.target, incoming.target);
    assert_eq!(retained.committed_until, incoming.committed_until);
    assert_eq!(retained.progress_deadline, incoming.progress_deadline);
    assert_eq!(retained.score, incoming.score);
    assert_eq!(retained.last_switch_reason, incoming.last_switch_reason);
    let placement = execution
        .plant()
        .expect("the opaque wrapper must expose its checked plant placement")
        .placement();
    assert_eq!(placement.placed_at, Tick(292));
    assert_eq!(placement.cell, CellId::new(8, 1));
    let target = execution
        .reactive_block_contribution()
        .expect("the reactive certificate must expose its sealed block contribution");
    assert!(witness.observation.parts().arena.blocks.contains(&target));
    assert!(observation_blast_line_reaches(
        &witness.observation,
        placement.cell,
        target,
        placement.range,
    ));
    assert_reactive_bomb_destroyed_target_and_owner_progressed(
        &reactive,
        placement.placed_at,
        placement.cell,
        target,
    );
}

fn observation_blast_line_reaches(
    observation: &grenadiers_core::cpu::Observation,
    origin: CellId,
    target: CellId,
    range: u16,
) -> bool {
    blast_line_reaches_parts(
        origin,
        target,
        range,
        |cell| {
            observation
                .parts()
                .arena
                .index(cell)
                .map(|index| observation.parts().arena.terrain[index])
        },
        |cell| observation.parts().arena.blocks.contains(&cell),
    )
}

fn assert_copied_reactive_certificate_is_rejected(
    trace: &LayoutTrace,
    witness: &super::trace::witness::PlanSnapshot,
    plan: &grenadiers_core::cpu::survival::SurvivalPlan,
) {
    let admission_tick = plan
        .execution
        .as_ref()
        .and_then(|execution| execution.manual_execution_tick())
        .and_then(|tick| tick.0.checked_sub(1))
        .map(Tick)
        .unwrap();
    let current = trace
        .publication_observations
        .iter()
        .find(|observation| observation.parts().tick == admission_tick)
        .expect("the physical trace must retain the exact admission publication");
    for profile in [
        CpuProfile::HARD_V64,
        CpuProfile::RECOVERY_V62_STANDARD,
        CpuProfile::RECOVERY_V62_EASY,
    ] {
        let basis = witness.observation.clone();
        let mut controller = CpuController::new(
            CpuPlanner::without_workers(),
            SeatId::Two,
            basis.parts().world.room_instance,
            [66; 32],
            profile,
        );
        controller.reset_world(basis.parts().world.physical_round);
        assert!(controller.accept_observation(&basis));
        controller.on_stop_admitted(witness.settled_stop_tick);
        controller.inject_plan_result_for_test(basis.clone(), 1, plan.clone());
        assert!(controller.poll(basis.parts().tick, 1).is_none());
        assert!(controller.accept_observation(current));
        assert!(controller.poll(admission_tick, 1).is_none());
        assert!(!controller.has_pending());
        let rejection = controller.admission_trace_for_test();
        assert_eq!(rejection.invalid_shape, 1, "profile={profile:?}");
        assert_eq!(rejection.route, 0, "profile={profile:?}");
    }
}

fn controlled_bomb_was_accepted(trace: &LayoutTrace) -> bool {
    trace.frames.iter().any(|frame| {
        frame
            .accepted_inputs
            .contains(&(SeatId::One, InputAction::PlaceBomb))
    })
}

fn assert_reactive_bomb_destroyed_target_and_owner_progressed(
    trace: &LayoutTrace,
    placed_at: Tick,
    placement_cell: CellId,
    target: CellId,
) {
    let owner = trace.initial_actors[SeatId::Two.index()].id();
    let placed = trace
        .frames
        .iter()
        .filter(|frame| frame.tick == placed_at)
        .flat_map(|frame| &frame.report.events)
        .find_map(|event| match event.kind {
            StepEventKind::BombPlaced(bomb)
                if bomb.owner == owner && bomb.cell == placement_cell =>
            {
                Some(bomb)
            }
            _ => None,
        })
        .expect("the admitted reactive manual command must place a real bomb");
    assert_eq!(placed.id, EntityId(6));
    assert_eq!(
        trace
            .frames
            .iter()
            .filter(|frame| (placed_at..=Tick(503)).contains(&frame.tick))
            .flat_map(|frame| &frame.accepted_inputs)
            .filter(|(seat, action)| { *seat == SeatId::Two && *action == InputAction::PlaceBomb })
            .count(),
        1,
        "own-bomb recovery must not introduce another manual command before completion"
    );
    assert!(blast_line_reaches(trace, placed.cell, target, placed.range));
    let detonated_at = trace
        .frames
        .iter()
        .flat_map(|frame| &frame.report.events)
        .find_map(|event| match event.kind {
            StepEventKind::BombDetonated(bomb) if bomb.id == placed.id => Some(event.tick),
            _ => None,
        })
        .expect("the reactive bomb must physically detonate");
    assert!(
        trace
            .frames
            .iter()
            .find(|frame| frame.tick == detonated_at)
            .unwrap()
            .report
            .events
            .iter()
            .filter_map(|event| match event.kind {
                StepEventKind::BombDetonated(bomb) if bomb.id != placed.id => Some(bomb),
                _ => None,
            })
            .all(|bomb| !blast_line_reaches(trace, bomb.cell, target, bomb.range))
    );
    let before = trace
        .frames
        .iter()
        .find(|frame| frame.tick.0 + 1 == detonated_at.0)
        .unwrap();
    let at_detonation = trace
        .frames
        .iter()
        .find(|frame| frame.tick == detonated_at)
        .unwrap();
    assert!(before.blocks.contains(&target));
    assert!(!at_detonation.blocks.contains(&target));
    let clear_at = at_detonation
        .flames
        .iter()
        .map(|flame| flame.expires_at)
        .max()
        .expect("the physical detonation must publish finite flames");
    assert!(
        trace
            .frames
            .iter()
            .filter(|frame| (placed_at..=clear_at).contains(&frame.tick))
            .all(|frame| frame.actors[SeatId::Two.index()].alive)
    );
    assert!(
        trace
            .frames
            .iter()
            .filter(|frame| frame.tick > clear_at)
            .any(|frame| frame.accepted_inputs.iter().any(|(seat, action)| {
                *seat == SeatId::Two
                    && matches!(
                        action,
                        InputAction::Press(_) | InputAction::PlaceBomb | InputAction::UseEquipment
                    )
            }))
    );
}

fn blast_line_reaches(trace: &LayoutTrace, origin: CellId, target: CellId, range: u16) -> bool {
    blast_line_reaches_parts(
        origin,
        target,
        range,
        |cell| trace.initial_world.arena().terrain(cell),
        |cell| trace.initial_world.arena().blocks().contains(&cell),
    )
}

fn blast_line_reaches_parts(
    origin: CellId,
    target: CellId,
    range: u16,
    terrain: impl Fn(CellId) -> Option<Terrain>,
    blocked: impl Fn(CellId) -> bool,
) -> bool {
    let same_row = origin.row == target.row;
    let same_column = origin.column == target.column;
    let distance = if same_row {
        origin.column.abs_diff(target.column)
    } else if same_column {
        origin.row.abs_diff(target.row)
    } else {
        return false;
    };
    if distance > range || !blocked(target) {
        return false;
    }
    let column_step = target.column.cmp(&origin.column) as i16;
    let row_step = target.row.cmp(&origin.row) as i16;
    (1..distance).all(|offset| {
        let cell = CellId::new(
            origin
                .column
                .wrapping_add_signed(column_step * offset as i16),
            origin.row.wrapping_add_signed(row_step * offset as i16),
        );
        terrain(cell) == Some(Terrain::Floor) && !blocked(cell)
    })
}

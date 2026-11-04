use grenadiers_core::{
    cpu::BombKnowledge,
    game::{InputAction, Position, SeatId, StepEventKind, Tick},
};

use super::super::trace::{LayoutTrace, witness::PlanSnapshot};

pub(super) fn assert_reactive_escape_recovers_revoked_plant(trace: &LayoutTrace) -> &PlanSnapshot {
    for tick in [Tick(441), Tick(444)] {
        assert_eq!(
            trace
                .frames
                .iter()
                .find(|frame| frame.tick == tick)
                .unwrap()
                .actors[SeatId::Two.index()]
            .position,
            Position {
                x: 11_631,
                y: 1_536
            }
        );
    }
    let revoked = trace.route_traces[0]
        .iter()
        .find(|entry| entry.invalidator.is_some())
        .expect("the new bomb publication must revoke the original plant escape");
    assert_eq!(revoked.tick, Tick(441));
    assert_eq!(
        revoked.invalidator,
        Some(grenadiers_core::cpu::survival::RouteInvalidator::PlantScene)
    );
    let recovery = trace
        .plan_witnesses
        .iter()
        .find(|witness| {
            witness.plan.as_ref().is_some_and(|plan| {
                plan.execution.as_ref().is_some_and(|execution| {
                    execution.requires_reactive_support() && execution.plant().is_none()
                })
            })
        })
        .expect("Hard65 must replan a typed own-bomb escape after revocation");
    assert!(recovery.observation.parts().tick >= Tick(444));
    assert!(recovery.work_spent <= grenadiers_core::cpu::survival::WORK_ALLOWANCE);
    let actor = recovery
        .observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            grenadiers_core::cpu::ObservedActor::Visible(actor) if actor.seat == SeatId::Two => {
                Some(actor.id)
            }
            _ => None,
        })
        .unwrap();
    assert!(matches!(
        &recovery.observation.parts().bombs,
        BombKnowledge::Visible(bombs)
            if bombs.len() >= 2
                && bombs.iter().any(|bomb| bomb.owner == actor)
                && bombs.iter().any(|bomb| bomb.owner != actor)
    ));
    let plan = recovery.plan.as_ref().unwrap();
    assert_eq!(plan.control.intent.manual, None);
    assert!(plan.execution.as_ref().unwrap().route().survive_until() > Tick(502));
    for (tick, status) in [
        (Tick(445), grenadiers_core::cpu::survival::RouteStatus::Wait),
        (Tick(502), grenadiers_core::cpu::survival::RouteStatus::Wait),
        (
            Tick(503),
            grenadiers_core::cpu::survival::RouteStatus::Completed,
        ),
    ] {
        assert!(trace.route_traces[0].iter().any(|entry| {
            entry.tick == tick
                && entry.reactive_support
                && entry.plant.is_none()
                && entry.status == status
        }));
    }
    assert!(
        trace
            .frames
            .iter()
            .filter(|frame| (recovery.observation.parts().tick..=Tick(502)).contains(&frame.tick))
            .all(|frame| frame.actors[SeatId::Two.index()].alive)
    );
    recovery
}

pub(super) fn reactive_witness_survives(trace: &LayoutTrace, witness: &PlanSnapshot) -> bool {
    let Some(placement) = witness
        .plan
        .as_ref()
        .and_then(|plan| plan.execution.as_ref())
        .and_then(|execution| execution.plant())
        .map(|plant| *plant.placement())
    else {
        return false;
    };
    let owner = trace.initial_actors[SeatId::Two.index()].id();
    let Some(placed) = trace
        .frames
        .iter()
        .filter(|frame| frame.tick == placement.placed_at)
        .flat_map(|frame| &frame.report.events)
        .find_map(|event| match event.kind {
            StepEventKind::BombPlaced(bomb)
                if bomb.owner == owner && bomb.cell == placement.cell =>
            {
                Some(bomb)
            }
            _ => None,
        })
    else {
        return false;
    };
    let Some(detonation) = trace.frames.iter().find(|frame| {
        frame.report.events.iter().any(
            |event| matches!(event.kind, StepEventKind::BombDetonated(bomb) if bomb.id == placed.id),
        )
    }) else {
        return false;
    };
    let Some(clear_at) = detonation.flames.iter().map(|flame| flame.expires_at).max() else {
        return false;
    };
    trace
        .frames
        .iter()
        .filter(|frame| (placement.placed_at..=clear_at).contains(&frame.tick))
        .all(|frame| frame.actors[SeatId::Two.index()].alive)
        && trace.frames.iter().any(|frame| {
            frame.tick > clear_at
                && frame.accepted_inputs.iter().any(|(seat, action)| {
                    *seat == SeatId::Two
                        && matches!(
                            action,
                            InputAction::Press(_)
                                | InputAction::PlaceBomb
                                | InputAction::UseEquipment
                        )
                })
        })
}

pub(super) fn reactive_outcome_ledger(
    trace: &LayoutTrace,
) -> Vec<(
    grenadiers_core::game::Tick,
    Option<grenadiers_core::game::Tick>,
    bool,
    bool,
)> {
    let mut ledger: Vec<(
        grenadiers_core::game::Tick,
        Option<grenadiers_core::game::Tick>,
        bool,
        bool,
    )> = Vec::new();
    for witness in &trace.plan_witnesses {
        let Some(execution) = witness
            .plan
            .as_ref()
            .and_then(|plan| plan.execution.as_ref())
            .filter(|execution| {
                execution.requires_reactive_support() && execution.plant().is_some()
            })
        else {
            continue;
        };
        let placement = execution.plant().unwrap().placement();
        if ledger.iter().any(|entry| entry.0 == placement.placed_at) {
            continue;
        }
        let detonation = trace.frames.iter().find(|frame| {
            frame.tick == placement.detonates_at
                && frame.report.events.iter().any(
                    |event| matches!(event.kind, StepEventKind::BombDetonated(bomb) if bomb.cell == placement.cell),
                )
        });
        let clear_at =
            detonation.and_then(|frame| frame.flames.iter().map(|flame| flame.expires_at).max());
        let survived = clear_at.is_some_and(|clear_at| {
            trace
                .frames
                .iter()
                .filter(|frame| (placement.placed_at..=clear_at).contains(&frame.tick))
                .all(|frame| frame.actors[SeatId::Two.index()].alive)
        });
        let progressed = clear_at.is_some_and(|clear_at| {
            trace.frames.iter().any(|frame| {
                frame.tick > clear_at
                    && frame.accepted_inputs.iter().any(|(seat, action)| {
                        *seat == SeatId::Two
                            && matches!(
                                action,
                                InputAction::Press(_)
                                    | InputAction::PlaceBomb
                                    | InputAction::UseEquipment
                            )
                    })
            })
        });
        ledger.push((placement.placed_at, clear_at, survived, progressed));
    }
    ledger
}

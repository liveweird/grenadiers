use crate::cpu::policy::{ControlPlan, StrategicObjectiveKind, StrategicPhase, StrategicTarget};
use crate::cpu::{MovementIntent, Observation};
use crate::game::{Position, Tick};

pub(super) fn attained_firing_position(observation: &Observation, ordinary: &ControlPlan) -> bool {
    let Some(intent) = ordinary.next_memory.strategic_intent() else {
        return false;
    };
    let Some(position) = super::compatibility::actor(observation).map(|actor| actor.position)
    else {
        return false;
    };
    ordinary.intent.movement == MovementIntent::Stop
        && matches!(
            intent.objective,
            StrategicObjectiveKind::Farm | StrategicObjectiveKind::Attack
        )
        && matches!(
            intent.target,
            StrategicTarget::Block { .. } | StrategicTarget::Opponent { .. }
        )
        && matches!(
            intent.phase,
            StrategicPhase::Approach | StrategicPhase::Reengage
        )
        && intent.approach_cell == position.cell()
}

pub(super) fn pulse_progresses(plan: &super::SurvivalPlan, initial_position: Position) -> bool {
    plan.lease.as_ref().is_some_and(|lease| {
        if lease.movement() == MovementIntent::Stop {
            return false;
        }
        let end = lease.pose_after(lease.hold_ticks());
        bounds_guarantee_progress(initial_position, end.min(), end.max())
    })
}

const fn bounds_guarantee_progress(initial: Position, min: Position, max: Position) -> bool {
    initial.x < min.x || initial.x > max.x || initial.y < min.y || initial.y > max.y
}

pub(super) fn prefer_pulse(
    recovering: bool,
    candidate: &super::SurvivalPlan,
    candidate_score: u32,
    old: &super::SurvivalPlan,
    old_score: u32,
    position: Position,
) -> bool {
    let candidate_progresses = pulse_progresses(candidate, position);
    let old_progresses = pulse_progresses(old, position);
    if recovering && candidate_progresses != old_progresses {
        candidate_progresses
    } else {
        candidate_score < old_score
    }
}

pub(super) fn recover_progressing(
    recovering: bool,
    plan: Option<super::SurvivalPlan>,
    position: Position,
    tick: Tick,
) -> Option<super::SurvivalPlan> {
    let mut plan = recovering.then_some(plan)??;
    if !pulse_progresses(&plan, position) {
        return None;
    }
    plan.control.next_memory = plan
        .control
        .next_memory
        .defer_strategic_target(tick)
        .without_execution_authority();
    Some(plan)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uncertain_end_bounds_containing_the_start_do_not_guarantee_progress() {
        let initial = Position { x: 100, y: 100 };
        assert!(!bounds_guarantee_progress(
            initial,
            Position { x: 90, y: 100 },
            Position { x: 110, y: 100 }
        ));
        assert!(bounds_guarantee_progress(
            initial,
            Position { x: 101, y: 90 },
            Position { x: 110, y: 110 }
        ));
    }
}

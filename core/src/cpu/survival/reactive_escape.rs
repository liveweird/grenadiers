use super::reactive_assumptions::ReactivePlantAssumptions;
use super::{ExecutionCertificate, SurvivalPlan};
use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::policy::ControlPlan;
use crate::cpu::routing::{RouteLimits, RouteMode, RouteRequest, RouteStep, plan_respawn_route};
use crate::cpu::{
    BombKnowledge, Intent, MovementIntent, Observation, ObservedBombTrigger, WorkBudget,
};
use crate::game::Tick;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReactiveRouteCertificate {
    route: crate::cpu::routing::RouteCertificate,
    assumptions: ReactivePlantAssumptions,
}
impl ReactiveRouteCertificate {
    fn new(
        route: crate::cpu::routing::RouteCertificate,
        assumptions: ReactivePlantAssumptions,
    ) -> Self {
        Self { route, assumptions }
    }
    pub(crate) const fn route(&self) -> &crate::cpu::routing::RouteCertificate {
        &self.route
    }
    pub(super) fn into_parts(
        self,
    ) -> (
        crate::cpu::routing::RouteCertificate,
        ReactivePlantAssumptions,
    ) {
        (self.route, self.assumptions)
    }
}

pub(super) enum ReactiveEscapeAttempt {
    Ineligible,
    Failed,
    Certified(Box<SurvivalPlan>, crate::cpu::MobContactLease),
}

pub(super) fn certify(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    fallback: &SurvivalPlan,
    work: &mut WorkBudget,
) -> ReactiveEscapeAttempt {
    let Some(mut assumptions) =
        ReactivePlantAssumptions::certify_escape(observation, history, settled_stop_tick, work)
    else {
        return ReactiveEscapeAttempt::Ineligible;
    };
    let Some(result) = certify_inner(
        observation,
        history,
        settled_stop_tick,
        fallback,
        &mut assumptions,
        work,
    ) else {
        return ReactiveEscapeAttempt::Failed;
    };
    ReactiveEscapeAttempt::Certified(Box::new(result.0), result.1)
}

fn certify_inner(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    fallback: &SurvivalPlan,
    assumptions: &mut ReactivePlantAssumptions,
    work: &mut WorkBudget,
) -> Option<(SurvivalPlan, crate::cpu::MobContactLease)> {
    let actor = super::compatibility::actor(observation)?;
    let BombKnowledge::Visible(bombs) = &observation.parts().bombs else {
        return None;
    };
    let mut last_own = None;
    for bomb in bombs.iter().filter(|bomb| bomb.owner == actor.id) {
        let ObservedBombTrigger::Timed {
            remaining_ticks,
            paused: false,
        } = bomb.trigger
        else {
            return None;
        };
        let at = Tick(
            observation
                .parts()
                .tick
                .0
                .checked_add(u64::from(remaining_ticks))?,
        );
        last_own = Some(last_own.map_or(at, |old: Tick| old.max(at)));
    }
    let flame_ticks = crate::cpu::rules_config(observation)?.flame_ticks();
    let survive_until = Tick(last_own?.0.checked_add(flame_ticks)?.checked_add(1)?);
    let forecast = forecast(observation, history, None, ForecastLimits::default(), work).ok()?;
    if survive_until > forecast.horizon_end() {
        return None;
    }
    let route = plan_respawn_route(
        observation,
        &forecast,
        RouteRequest {
            start_window: TickWindow {
                start: Tick(observation.parts().tick.0.checked_add(1)?),
                end: Tick(observation.parts().tick.0.checked_add(8)?),
            },
            settled_stop_tick,
            survive_until,
            mode: RouteMode::ReactiveStationaryAssumptions,
            limits: RouteLimits::default(),
        },
        work,
    )
    .ok()?;
    let contact = crate::cpu::MobContactLease::certify_route(observation, &route, work).ok()?;
    *assumptions = assumptions.clone().with_contact(contact.clone());
    let (movement, hold_ticks) = match route.steps().first()? {
        RouteStep::Move { direction, updates } => (MovementIntent::Direction(*direction), *updates),
        RouteStep::Wait { .. } => (MovementIntent::Stop, 1),
    };
    Some((
        SurvivalPlan {
            control: ControlPlan {
                intent: Intent {
                    movement,
                    manual: None,
                },
                hold_ticks,
                next_memory: fallback.control.next_memory,
            },
            lease: None,
            execution: Some(ExecutionCertificate::ReactiveRoute(Box::new(
                ReactiveRouteCertificate::new(route, assumptions.clone()),
            ))),
        },
        contact,
    ))
}

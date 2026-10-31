use crate::cpu::routing::{RouteCertificate, RouteStep};
use crate::cpu::{BombKnowledge, MovementIntent, Observation};
use crate::game::{OilBarrelState, Tick};

pub(super) struct Input<'a> {
    pub(super) basis: &'a Observation,
    pub(super) current: &'a Observation,
    pub(super) certificate: &'a RouteCertificate,
    pub(super) nonplant: bool,
    pub(super) movement: MovementIntent,
    pub(super) no_pending: bool,
    pub(super) first_update: Tick,
    pub(super) settled_stop_tick: Option<Tick>,
}

pub(super) fn can_complete(input: Input<'_>) -> bool {
    let Some(settled) = input.settled_stop_tick else {
        return false;
    };
    let BombKnowledge::Visible(basis_bombs) = &input.basis.parts().bombs else {
        return false;
    };
    let BombKnowledge::Visible(current_bombs) = &input.current.parts().bombs else {
        return false;
    };
    input.nonplant
        && input.current.spike_pits().is_none()
        && input
            .certificate
            .steps()
            .iter()
            .all(|step| matches!(step, RouteStep::Wait { .. }))
        && input.movement == MovementIntent::Stop
        && input.no_pending
        && !basis_bombs.is_empty()
        && current_bombs.is_empty()
        && input.current.parts().flames.is_empty()
        && !input
            .current
            .parts()
            .oil
            .iter()
            .any(|oil| oil.state == OilBarrelState::Burning)
        && input.first_update <= settled
        && settled <= input.current.parts().tick
}

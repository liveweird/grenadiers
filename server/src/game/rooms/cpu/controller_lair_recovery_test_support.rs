use grenadiers_core::cpu::{Observation, ObservationParts, ObservedActor, VisibleActor};
use grenadiers_core::game::SeatId;

pub(super) fn visible_cpu(observation: &Observation) -> &VisibleActor {
    let ObservedActor::Visible(actor) = &observation.parts().actors[SeatId::Two.index()] else {
        unreachable!()
    };
    actor
}

pub(super) fn visible_cpu_mut(parts: &mut ObservationParts) -> &mut VisibleActor {
    let ObservedActor::Visible(actor) = &mut parts.actors[SeatId::Two.index()] else {
        unreachable!()
    };
    actor
}

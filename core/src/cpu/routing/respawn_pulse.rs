use crate::cpu::forecast::HazardForecast;
use crate::cpu::{Observation, WorkBudget};

use super::{PulseRequest, RouteFailure};

pub(crate) fn certify_respawn_pulse(
    observation: &Observation,
    forecast: &HazardForecast,
    request: PulseRequest,
    work: &mut WorkBudget,
) -> Result<super::PulseLease, RouteFailure> {
    if observation.parts().rules.version != 9 {
        return Err(RouteFailure::InvalidRequest);
    }
    super::pulse::certify_pulse_inner(
        observation,
        forecast,
        request,
        None,
        super::pickup_pulse::PickupSafety::Reject,
        None,
        work,
    )
}

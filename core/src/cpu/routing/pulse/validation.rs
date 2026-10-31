use super::{Context, HazardForecast, MovementIntent, PulseRequest, RouteFailure};

const MAX_PULSE_TICKS: u8 = 12;
pub(super) const MAX_CONDITIONAL_TICKS: u8 = 3;
const MAX_ACCEPTED_AGE: u64 = 6;
const STOPPED_PUBLICATION_TICKS: u64 = 3;

pub(super) fn validate_request(
    context: &Context<'_>,
    forecast: &HazardForecast,
    request: PulseRequest,
) -> Result<(), RouteFailure> {
    let basis = forecast.basis_tick().0;
    let latest_start_end = basis
        .checked_add(MAX_ACCEPTED_AGE + 2)
        .ok_or(RouteFailure::InvalidRequest)?;
    let latest_stop = request
        .start_window
        .end
        .0
        .checked_sub(1)
        .and_then(|tick| tick.checked_add(u64::from(request.hold_ticks)))
        .ok_or(RouteFailure::InvalidRequest)?;
    let required_lease = latest_stop
        .checked_add(STOPPED_PUBLICATION_TICKS + MAX_ACCEPTED_AGE + 1)
        .ok_or(RouteFailure::InvalidRequest)?;
    if request.hold_ticks == 0
        || request.hold_ticks > MAX_PULSE_TICKS
        || request.start_window.start.0 <= basis
        || request.start_window.end.0 > latest_start_end
        || request.settled_stop_tick > forecast.basis_tick()
        || request.stopped_safe_until.0 < required_lease
        || request.stopped_safe_until > forecast.horizon_end()
        || (request.movement == MovementIntent::Stop && request.hold_ticks != 1)
    {
        return Err(RouteFailure::InvalidRequest);
    }
    if request.settled_stop_tick.0 > context.observation.parts().tick.0 {
        return Err(RouteFailure::InvalidRequest);
    }
    Ok(())
}

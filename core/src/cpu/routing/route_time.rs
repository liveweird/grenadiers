use crate::game::Tick;

use super::RouteFailure;

pub(super) fn add_tick(tick: Tick, updates: u64) -> Result<Tick, RouteFailure> {
    tick.0
        .checked_add(updates)
        .map(Tick)
        .ok_or(RouteFailure::InvalidRequest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_clock_addition_rejects_overflow() {
        assert_eq!(
            add_tick(Tick(u64::MAX - 9), 10),
            Err(RouteFailure::InvalidRequest)
        );
        assert_eq!(add_tick(Tick(u64::MAX - 10), 10), Ok(Tick(u64::MAX)));
    }
}

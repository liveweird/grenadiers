use grenadiers_core::{
    game::{SeatId, Tick},
    room::CpuProfile,
};

use super::{LayoutCase, LayoutTrace, run_layout_trace_inner};

pub(crate) fn run_zero_bomber_trace(case: LayoutCase, ticks: usize, seat: SeatId) -> LayoutTrace {
    run_zero_bomber_trace_with_profile(case, ticks, seat, CpuProfile::HARD_V63)
}

pub(crate) fn run_zero_bomber_trace_with_profile(
    case: LayoutCase,
    ticks: usize,
    seat: SeatId,
    profile: CpuProfile,
) -> LayoutTrace {
    run_layout_trace_inner(case, ticks, profile, Some((seat, Tick(0), Tick(840))), true)
}

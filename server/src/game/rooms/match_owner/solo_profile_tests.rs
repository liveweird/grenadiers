use std::sync::Arc;

use grenadiers_core::room::{CpuProfile, RoomMode};
use tokio::{sync::watch, time::Instant};

use super::{PreparedMatchOwner, test_support::identity};
use crate::{game::rooms::profile::RoomProfile, round_seed::RoundSeedSource};

#[test]
fn archived_cpu_profile_cannot_label_a_current_live_worker() {
    for profile in [
        CpuProfile::INITIAL,
        CpuProfile::PULSE_V2,
        CpuProfile::ROUTE_V3,
        CpuProfile::PLANT_V4,
        CpuProfile::GOALS_V5,
        CpuProfile::WEAPONS_V6,
        CpuProfile::WALL_PASS_V7,
        CpuProfile::RESPONSIVE_V8,
        CpuProfile::ORIGIN_V9,
        CpuProfile::GUIDED_V10,
        CpuProfile::CLEAR_GAP_V11,
        CpuProfile::STEERING_V12,
        CpuProfile::BRACELET_V13,
        CpuProfile::PUNCH_V14,
        CpuProfile::EQUIPMENT_GOALS_V15,
    ] {
        assert!(matches!(
            prepared(profile),
            Err(crate::game::rooms::match_runtime::MatchRuntimeError::Publication)
        ));
    }
}

#[test]
fn both_exact_current_profiles_can_label_a_live_worker() {
    for profile in [CpuProfile::CURRENT, CpuProfile::CURRENT_EASY] {
        assert!(
            prepared(profile).is_ok(),
            "current profile {profile:?} must be admitted"
        );
    }
}

fn prepared(
    profile: CpuProfile,
) -> Result<
    (super::MatchOwnerHandle, PreparedMatchOwner),
    crate::game::rooms::match_runtime::MatchRuntimeError,
> {
    let mut room = identity(RoomProfile::ExpandedV5);
    room.mode = RoomMode::Solo(profile);
    room.private_cpu_seed = Some([17; 32]);
    let (_, shutdown) = watch::channel(false);
    PreparedMatchOwner::new_with_planner(
        room,
        Instant::now(),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(23);
            Ok(())
        }),
        Arc::new(|bytes: &mut [u8]| {
            bytes.fill(31);
            Ok(())
        }),
        crate::game::rooms::cpu::CpuPlanner::without_workers(),
        shutdown,
    )
}

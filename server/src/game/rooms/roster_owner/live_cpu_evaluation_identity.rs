use grenadiers_core::{
    game::{
        ArenaConstructionProfile, MobNavigationPolicy, PunchActivationPolicy, SpikePitLayoutPolicy,
    },
    room::BombCapacityDropMultiplier,
};

use super::super::live_cpu_evaluation_types::EvaluationCase;
use crate::game::rooms::roster_runtime::RosterRuntimeInspection;

pub(super) fn assert_identity(
    inspection: &RosterRuntimeInspection,
    case: EvaluationCase,
    navigation: MobNavigationPolicy,
) -> Result<(), String> {
    let simulation = inspection.simulation.as_ref().unwrap();
    let valid = inspection.construction_profile
        == ArenaConstructionProfile::WeightedLairClearanceV7
        && simulation.generation.version() == 7
        && simulation.state.rules().spike_pit_layout_policy()
            == SpikePitLayoutPolicy::LairClearanceV2
        && simulation.state.rules().punch_activation_policy() == PunchActivationPolicy::ContactV2
        && inspection.mob_navigation == navigation
        && inspection
            .lifecycle
            .settings()
            .bomb_capacity_drop_multiplier()
            == BombCapacityDropMultiplier::DEFAULT
        && case.profile().difficulty == case.difficulty;
    valid
        .then_some(())
        .ok_or_else(|| "live evaluation identity mismatch".to_owned())
}

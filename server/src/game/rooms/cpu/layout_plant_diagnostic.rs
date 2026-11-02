use grenadiers_core::cpu::{
    policy::PolicyError,
    survival::{HardPlantDiagnostic, SurvivalPlan, diagnose_hard_v61_plant},
};

use crate::game::rooms::cpu::PlannedResultWitness;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::game::rooms::roster_owner) struct FullProof {
    pub spent: u32,
    pub plan: Option<SurvivalPlan>,
    pub error: Option<PolicyError>,
}

pub(super) fn capture(
    witness: &PlannedResultWitness<'_>,
    direct_spent: u32,
    demolition_enabled: bool,
) -> (u32, Option<HardPlantDiagnostic>) {
    if witness.basis.escape_origin.is_some() || witness.plan.is_none() {
        return (witness.work_spent.saturating_sub(direct_spent), None);
    }
    let mut prefix = witness.work_spent.saturating_sub(direct_spent);
    let mut diagnostic = None;
    for _ in 0..8 {
        diagnostic = {
            let mut rng = witness.basis.rng.clone();
            diagnose_hard_v61_plant(
                witness.basis.observation,
                witness.basis.memory,
                witness.basis.hazard_memory,
                witness.basis.settled_stop_tick,
                &mut rng,
                65_536_u32.saturating_sub(prefix),
                false,
            )
            .ok()
        };
        let Some(probe) = diagnostic.as_ref() else {
            break;
        };
        let next = witness.work_spent.saturating_sub(probe.total_spent);
        if next == prefix {
            break;
        }
        prefix = next;
    }
    if demolition_enabled {
        let mut rng = witness.basis.rng.clone();
        diagnostic = diagnose_hard_v61_plant(
            witness.basis.observation,
            witness.basis.memory,
            witness.basis.hazard_memory,
            witness.basis.settled_stop_tick,
            &mut rng,
            65_536_u32.saturating_sub(prefix),
            true,
        )
        .ok();
    }
    (prefix, diagnostic)
}

pub(in crate::game::rooms::roster_owner) fn demolition_enabled(
    snapshots: &[super::PlanSnapshot],
) -> bool {
    snapshots.iter().all(|snapshot| {
        snapshot
            .plant_diagnostic
            .as_ref()
            .is_none_or(|diagnostic| diagnostic.demolition.is_none())
    })
}

pub(in crate::game::rooms::roster_owner) fn digest(value: &impl std::fmt::Debug) -> u64 {
    format!("{value:?}")
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
        })
}

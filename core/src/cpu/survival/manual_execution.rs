use crate::cpu::policy::PolicyError;
use crate::cpu::routing::{
    PlantCertificate, PlantPresence, PunchCertificate, PunchOutcome, RouteCertificate,
};
use crate::cpu::{ManualIntent, MovementIntent, Observation};
use crate::game::Tick;

use super::compatibility::{actor, actor_compatible, plant_actor_compatible, route_compatible};
use super::route_execution::CommittedPlant;
use super::{CommittedRoute, PlantContactCertificate};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionCertificate {
    Route(Box<RouteCertificate>),
    Plant(Box<PlantCertificate>),
    ContactPlant(Box<PlantContactCertificate>),
    Punch(Box<PunchCertificate>),
}

impl ExecutionCertificate {
    #[must_use]
    pub fn route(&self) -> &RouteCertificate {
        match self {
            Self::Route(route) => route,
            Self::Plant(plant) => plant.route(),
            Self::ContactPlant(plant) => plant.route(),
            Self::Punch(punch) => punch.route(),
        }
    }

    #[must_use]
    pub const fn plant(&self) -> Option<&PlantCertificate> {
        match self {
            Self::Plant(plant) => Some(plant),
            Self::ContactPlant(plant) => Some(plant.plant()),
            Self::Route(_) | Self::Punch(_) => None,
        }
    }

    #[must_use]
    pub const fn punch(&self) -> Option<&PunchCertificate> {
        match self {
            Self::Punch(punch) => Some(punch),
            Self::Route(_) | Self::Plant(_) | Self::ContactPlant(_) => None,
        }
    }

    #[must_use]
    pub const fn manual_intent(&self) -> Option<ManualIntent> {
        match self {
            Self::Route(_) => None,
            Self::Plant(_) | Self::ContactPlant(_) => Some(ManualIntent::Bomb),
            Self::Punch(_) => Some(ManualIntent::Equipment),
        }
    }

    #[must_use]
    pub fn manual_execution_tick(&self) -> Option<Tick> {
        match self {
            Self::Route(_) => None,
            Self::Plant(plant) => Some(plant.placement().placed_at),
            Self::ContactPlant(plant) => Some(plant.plant().placement().placed_at),
            Self::Punch(punch) => Some(punch.execution_tick()),
        }
    }

    /// Returns the exact opt-in capability required by a typed contact plant.
    #[must_use]
    pub const fn contact_plant_requires_blocked_birth_support(&self) -> Option<bool> {
        match self {
            Self::ContactPlant(plant) => Some(plant.requires_blocked_birth_support()),
            Self::Route(_) | Self::Plant(_) | Self::Punch(_) => None,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct CommittedPunch {
    certificate: PunchCertificate,
    outcome: PunchOutcome,
}

impl CommittedRoute {
    pub fn admit_execution(
        execution: ExecutionCertificate,
        basis: Observation,
        current: &Observation,
        admission_tick: Tick,
        accepted_movement: MovementIntent,
    ) -> Result<Self, PolicyError> {
        match execution {
            ExecutionCertificate::Route(certificate) => Self::admit(
                *certificate,
                basis,
                current,
                admission_tick,
                accepted_movement,
            ),
            ExecutionCertificate::Plant(certificate) => {
                let route = certificate.route().clone();
                Self::admit_inner(
                    route,
                    Some(CommittedPlant {
                        certificate: *certificate,
                        presence: PlantPresence::Awaiting,
                        contact: None,
                    }),
                    basis,
                    current,
                    admission_tick,
                    accepted_movement,
                )
            }
            ExecutionCertificate::ContactPlant(certificate) => {
                let (plant, proof) = certificate.into_parts();
                let route = plant.route().clone();
                Self::admit_inner(
                    route,
                    Some(CommittedPlant {
                        certificate: plant,
                        presence: PlantPresence::Awaiting,
                        contact: Some(proof),
                    }),
                    basis,
                    current,
                    admission_tick,
                    accepted_movement,
                )
            }
            ExecutionCertificate::Punch(certificate) => {
                if accepted_movement != MovementIntent::Stop
                    || !certificate.can_admit(&basis, current, admission_tick)
                {
                    return Err(PolicyError::InvalidObservation);
                }
                let mut route = Self::admit(
                    certificate.route().clone(),
                    basis,
                    current,
                    admission_tick,
                    accepted_movement,
                )?;
                route.punch = Some(CommittedPunch {
                    certificate: *certificate,
                    outcome: PunchOutcome::Awaiting,
                });
                Ok(route)
            }
        }
    }

    pub fn select_unchanged_punch(&mut self) -> bool {
        let Some(punch) = self.punch.as_mut() else {
            return false;
        };
        if punch.outcome == PunchOutcome::Relocated {
            return false;
        }
        punch.outcome = PunchOutcome::Unchanged;
        true
    }

    #[must_use]
    pub fn punch_outcome(&self) -> Option<PunchOutcome> {
        self.punch.as_ref().map(|punch| punch.outcome)
    }

    pub(super) fn scene_matches(&mut self, current: &Observation) -> bool {
        if let Some(punch) = self.punch.as_mut() {
            let candidate = if punch.outcome == PunchOutcome::Awaiting {
                let Ok(outcome) = punch.certificate.confirm(&self.basis, current) else {
                    return false;
                };
                outcome
            } else {
                punch.outcome
            };
            let Some(before) = actor(&self.basis) else {
                return false;
            };
            let expected =
                punch
                    .certificate
                    .expected_live_bombs(before.id, current.parts().tick, candidate);
            let compatible = punch
                .certificate
                .matches_scene(&self.basis, current, candidate)
                && actor_compatible(&self.basis, current, expected, before.weapon);
            if compatible {
                punch.outcome = candidate;
            }
            return compatible;
        }
        let Some(plant) = self.plant.as_mut() else {
            return route_compatible(&self.basis, current, self.certificate.evolution());
        };
        if let Some(contact) = &plant.contact {
            let Some(candidate) = super::contact_plant::contact_scene_candidate(
                &self.basis,
                current,
                &plant.certificate,
                contact,
                plant.presence,
            ) else {
                return false;
            };
            plant.presence = candidate;
            return true;
        }
        let candidate = if plant.presence == PlantPresence::Awaiting {
            let Ok(presence) = plant.certificate.confirm(&self.basis, current) else {
                return false;
            };
            presence
        } else {
            plant.presence
        };
        let compatible = plant
            .certificate
            .matches_scene(&self.basis, current, candidate)
            && plant_actor_compatible(&self.basis, current, &plant.certificate, candidate);
        if compatible {
            plant.presence = candidate;
        }
        compatible
    }
}

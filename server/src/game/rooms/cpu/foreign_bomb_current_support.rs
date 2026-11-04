use super::support;

use crate::game::rooms::cpu::PlanningBasis;
use grenadiers_core::{
    cpu::{
        BombKnowledge, CpuRng, Observation, ObservedBombTrigger,
        forecast::ObservedHazardMemory,
        policy::{StrategicObjectiveKind, StrategicPhase, UtilityMemory},
        survival::{
            HardPlantOutcome, PlantCandidateOutcome, PlantEscapeOrigin, WORK_ALLOWANCE,
            diagnose_hard_v61_plant,
        },
    },
    game::{EffectKind, EntityId, Equipment, OilBarrelState, PickupKind, Tick},
    room::{BoardSize, CpuDifficulty, CpuPolicy},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Guard {
    VisibleTeleporter,
    ActiveFlame,
    VisibleBombs,
    BurningOil,
    Other,
}

#[derive(Clone, Debug)]
pub(super) struct Capture {
    pub case: MatrixCase,
    pub policy: CpuPolicy,
    pub observation: Observation,
    pub memory: UtilityMemory,
    pub history: ObservedHazardMemory,
    pub settled_stop: Tick,
    pub rng: CpuRng,
    pub origin: Option<OriginCapture>,
    pub actor: EntityId,
    pub at_approach: bool,
    pub foreign_bombs: usize,
    pub foreign_deadlines: Vec<Tick>,
    pub guard: Guard,
    pub effects: [bool; 3],
    pub living_actors: usize,
    pub foreign_owner_dead: bool,
    pub equipment: [bool; 5],
    pub damage_grace: bool,
    pub pickups: usize,
    pub flames: usize,
    pub oil: usize,
}

#[derive(Clone)]
pub(super) struct OriginCapture(pub PlantEscapeOrigin);

impl std::fmt::Debug for OriginCapture {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("PlantEscapeOrigin(..)")
    }
}

impl OriginCapture {
    pub(super) fn retained(&self) -> bool {
        self.0.same_origin(&self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MatrixCase {
    pub difficulty: CpuDifficulty,
    pub board: BoardSize,
    pub lairs: bool,
    pub seed: u8,
}

#[derive(Debug)]
pub(super) struct MatrixOutcome {
    pub cases: usize,
    pub jobs: usize,
    pub captured_jobs: usize,
    pub at_approach_jobs: usize,
    pub guards: [usize; 5],
    pub effects: [usize; 3],
    pub living_actors: [usize; 5],
    pub foreign_owner_dead: usize,
    pub equipment: [usize; 5],
    pub damage_grace: usize,
    pub with_pickups: usize,
    pub with_flames: usize,
    pub with_oil: usize,
    pub examples: Vec<Capture>,
    pub placements: usize,
    pub delivered: usize,
    pub failed: usize,
}

pub(super) fn run_matrix() -> MatrixOutcome {
    let mut outcome = MatrixOutcome {
        cases: 0,
        jobs: 0,
        captured_jobs: 0,
        at_approach_jobs: 0,
        guards: [0; 5],
        effects: [0; 3],
        living_actors: [0; 5],
        foreign_owner_dead: 0,
        equipment: [0; 5],
        damage_grace: 0,
        with_pickups: 0,
        with_flames: 0,
        with_oil: 0,
        examples: Vec::new(),
        placements: 0,
        delivered: 0,
        failed: 0,
    };
    for (difficulty, seed) in [
        (CpuDifficulty::Easy, 71),
        (CpuDifficulty::Standard, 72),
        (CpuDifficulty::Hard, 73),
    ] {
        for board in [
            BoardSize::ElevenByNine,
            BoardSize::ThirteenByEleven,
            BoardSize::FifteenByThirteen,
        ] {
            for lairs in [false, true] {
                let matrix_case = MatrixCase {
                    difficulty,
                    board,
                    lairs,
                    seed,
                };
                let profile = match difficulty {
                    CpuDifficulty::Easy => grenadiers_core::room::CpuProfile::RECOVERY_V62_EASY,
                    CpuDifficulty::Standard => {
                        grenadiers_core::room::CpuProfile::RECOVERY_V62_STANDARD
                    }
                    CpuDifficulty::Hard => grenadiers_core::room::CpuProfile::HARD_V64,
                };
                let result = support::run_inner(
                    support::Case {
                        label: "foreign-bomb-current-matrix",
                        difficulty,
                        board,
                        lairs,
                        seed,
                        native: false,
                    },
                    840,
                    profile,
                    true,
                    |basis| {
                        outcome.jobs += 1;
                        if let Some(capture) = capture(matrix_case, basis) {
                            outcome.captured_jobs += 1;
                            outcome.at_approach_jobs += usize::from(capture.at_approach);
                            outcome.guards[capture.guard as usize] += 1;
                            for (index, active) in capture.effects.into_iter().enumerate() {
                                outcome.effects[index] += usize::from(active);
                            }
                            outcome.with_pickups += usize::from(capture.pickups > 0);
                            outcome.with_flames += usize::from(capture.flames > 0);
                            outcome.with_oil += usize::from(capture.oil > 0);
                            if capture.at_approach {
                                outcome.living_actors[capture.living_actors] += 1;
                                outcome.foreign_owner_dead +=
                                    usize::from(capture.foreign_owner_dead);
                                for (index, held) in capture.equipment.into_iter().enumerate() {
                                    outcome.equipment[index] += usize::from(held);
                                }
                                outcome.damage_grace += usize::from(capture.damage_grace);
                            }
                            if capture.at_approach
                                && !outcome.examples.iter().any(|example| {
                                    example.case.difficulty == capture.case.difficulty
                                        && example.case.lairs == capture.case.lairs
                                })
                            {
                                outcome.examples.push(capture);
                            }
                        }
                    },
                );
                outcome.cases += 1;
                outcome.placements += result.event_placements.iter().sum::<usize>();
                outcome.delivered += result.delivered;
                outcome.failed += result.failed;
            }
        }
    }
    outcome
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProbeSummary {
    pub case: MatrixCase,
    pub tick: Tick,
    pub outcome: HardPlantOutcome,
    pub total_spent: u32,
    pub placements: Vec<Option<grenadiers_core::cpu::routing::PlannedBomb>>,
    pub placement_spent: Vec<u32>,
    pub candidate_allowances: Vec<u32>,
    pub usefulness: Vec<Option<bool>>,
    pub usefulness_spent: Vec<u32>,
    pub candidate_outcomes: Vec<PlantCandidateOutcome>,
    pub rng_unchanged: bool,
    pub basis_unchanged: bool,
}

pub(super) fn probe(capture: &Capture) -> ProbeSummary {
    let before_rng = capture.rng.clone();
    let before_observation = capture.observation.clone();
    let before_memory = capture.memory;
    let before_history = capture.history.clone();
    let mut rng = capture.rng.clone();
    let diagnostic = diagnose_hard_v61_plant(
        &capture.observation,
        &capture.memory,
        &capture.history,
        capture.settled_stop,
        &mut rng,
        WORK_ALLOWANCE,
        false,
    )
    .unwrap();
    ProbeSummary {
        case: capture.case,
        tick: capture.observation.parts().tick,
        outcome: diagnostic.outcome,
        total_spent: diagnostic.total_spent,
        placements: diagnostic
            .candidates
            .iter()
            .map(|candidate| candidate.placement)
            .collect(),
        placement_spent: diagnostic
            .candidates
            .iter()
            .map(|candidate| candidate.placement_derivation_spent)
            .collect(),
        candidate_allowances: diagnostic
            .candidates
            .iter()
            .map(|candidate| candidate.allowance)
            .collect(),
        usefulness: diagnostic
            .candidates
            .iter()
            .map(|candidate| candidate.derived_exact_usefulness)
            .collect(),
        usefulness_spent: diagnostic
            .candidates
            .iter()
            .map(|candidate| candidate.usefulness_probe_spent)
            .collect(),
        candidate_outcomes: diagnostic
            .candidates
            .iter()
            .map(|candidate| candidate.outcome)
            .collect(),
        rng_unchanged: capture.rng.key() == before_rng.key()
            && capture.rng.draws() == before_rng.draws(),
        basis_unchanged: capture.observation == before_observation
            && capture.memory == before_memory
            && capture.history == before_history,
    }
}

fn capture(case: MatrixCase, basis: PlanningBasis<'_>) -> Option<Capture> {
    let intent = basis.memory.strategic_intent()?;
    if intent.objective != StrategicObjectiveKind::Farm || intent.phase != StrategicPhase::Approach
    {
        return None;
    }
    let actor = basis
        .observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            grenadiers_core::cpu::ObservedActor::Visible(actor)
                if actor.seat == basis.observation.parts().recipient =>
            {
                Some(actor)
            }
            _ => None,
        })?;
    if actor.live_bomb_count >= actor.bomb_capacity {
        return None;
    }
    let BombKnowledge::Visible(bombs) = &basis.observation.parts().bombs else {
        return None;
    };
    let foreign = bombs
        .iter()
        .filter(|bomb| bomb.owner != actor.id)
        .collect::<Vec<_>>();
    if foreign.is_empty() {
        return None;
    }
    let parts = basis.observation.parts();
    let living = parts
        .actors
        .iter()
        .filter_map(|actor| match actor {
            grenadiers_core::cpu::ObservedActor::Visible(actor) if actor.alive => Some(actor),
            _ => None,
        })
        .collect::<Vec<_>>();
    let foreign_deadlines = foreign
        .iter()
        .filter_map(|bomb| match bomb.trigger {
            ObservedBombTrigger::Timed {
                remaining_ticks,
                paused: false,
            } => parts
                .tick
                .0
                .checked_add(u64::from(remaining_ticks))
                .map(Tick),
            _ => None,
        })
        .collect();
    let guard = if parts
        .pickups
        .iter()
        .any(|pickup| pickup.kind == PickupKind::Teleporter)
    {
        Guard::VisibleTeleporter
    } else if !parts.flames.is_empty() {
        Guard::ActiveFlame
    } else if !bombs.is_empty() {
        Guard::VisibleBombs
    } else if parts
        .oil
        .iter()
        .any(|oil| oil.state == OilBarrelState::Burning)
    {
        Guard::BurningOil
    } else {
        Guard::Other
    };
    Some(Capture {
        case,
        policy: *basis.policy,
        observation: basis.observation.clone(),
        memory: *basis.memory,
        history: basis.hazard_memory.clone(),
        settled_stop: basis.settled_stop_tick,
        rng: basis.rng.clone(),
        origin: basis.escape_origin.cloned().map(OriginCapture),
        actor: actor.id,
        at_approach: intent.approach_cell == actor.position.cell(),
        foreign_bombs: foreign.len(),
        foreign_deadlines,
        guard,
        effects: [
            EffectKind::Shield,
            EffectKind::ShortFuse,
            EffectKind::LongFuse,
        ]
        .map(|kind| living.iter().any(|actor| actor.effects.iter().any(|effect| effect.kind == kind))),
        living_actors: living.len(),
        foreign_owner_dead: foreign.iter().any(|bomb| {
            parts.actors.iter().any(|actor| {
                matches!(actor, grenadiers_core::cpu::ObservedActor::Visible(owner) if owner.id == bomb.owner && !owner.alive)
            })
        }),
        equipment: [
            Equipment::Kick,
            Equipment::Punch,
            Equipment::Heart,
            Equipment::WallPass,
            Equipment::Bracelet,
        ]
        .map(|equipment| living.iter().any(|actor| actor.equipment == Some(equipment))),
        damage_grace: living
            .iter()
            .any(|actor| actor.damage_grace_remaining_ticks > 0),
        pickups: parts.pickups.len(),
        flames: parts.flames.len(),
        oil: parts.oil.len(),
    })
}

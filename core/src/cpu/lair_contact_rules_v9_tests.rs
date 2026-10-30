use crate::cpu::{Observation, ObservationError, ObservedRespawnLairs};
use crate::game::{CellId, LairStatusView, MobView, Position, Tick, lair_layout_v1};
use crate::room::{BoardSize, RespawnLairConfig, RespawnLairSettings};

use super::tests::{mob, observation};
use super::{MobContactEnvelope, SourceKind, WorkBudget};

fn respawn_observation(mobs: Vec<MobView>, deadlines: [Option<Tick>; 2]) -> Observation {
    let legacy = observation(Vec::new(), Tick(100));
    let environment = legacy.environment().unwrap().clone();
    let mut parts = legacy.into_parts();
    parts.rules.version = 9;
    let counts = [
        mobs.iter().filter(|mob| mob.lair == 0).count() as u8,
        mobs.iter().filter(|mob| mob.lair == 1).count() as u8,
    ];
    let lairs = lair_layout_v1(BoardSize::FifteenByThirteen)
        .into_iter()
        .enumerate()
        .map(|(index, cell)| LairStatusView {
            cell,
            next_spawn_at: deadlines[index],
            living_mobs: counts[index],
        })
        .collect();
    Observation::new_rules_v9(
        parts,
        environment,
        Some(ObservedRespawnLairs::new(
            RespawnLairConfig::Enabled(RespawnLairSettings::new(5).unwrap()),
            lairs,
            mobs,
        )),
    )
    .unwrap()
}

#[test]
fn rules_v9_overlay_enforces_nullable_deadline_occupancy_and_independent_timers() {
    let occupied = mob(10, Position::center(CellId::new(4, 4)));
    let valid = respawn_observation(vec![occupied], [None, Some(Tick(100))]);
    assert_eq!(
        valid.respawn_lairs().unwrap().lairs()[1].next_spawn_at,
        Some(Tick(100))
    );
    let environment = valid.environment().unwrap().clone();
    let overlay = valid.respawn_lairs().unwrap().clone();
    let mut bad_lairs = overlay.lairs().to_vec();
    bad_lairs[0].next_spawn_at = Some(Tick(200));
    assert_eq!(
        Observation::new_rules_v9(
            valid.clone().into_parts(),
            environment.clone(),
            Some(ObservedRespawnLairs::new(
                overlay.config(),
                bad_lairs,
                overlay.mobs().to_vec(),
            )),
        ),
        Err(ObservationError::InvalidActors)
    );
    let mut missing = overlay.lairs().to_vec();
    missing[1].next_spawn_at = None;
    assert!(
        Observation::new_rules_v9(
            valid.into_parts(),
            environment,
            Some(ObservedRespawnLairs::new(
                overlay.config(),
                missing,
                overlay.mobs().to_vec(),
            )),
        )
        .is_err()
    );
}

#[test]
fn rules_v9_mob_envelope_seeds_empty_and_death_plus_grace_independently() {
    let observed = respawn_observation(
        vec![mob(10, Position::center(CellId::new(4, 4)))],
        [None, Some(Tick(175))],
    );
    let envelope = MobContactEnvelope::new(&observed, &mut WorkBudget::new(100)).unwrap();
    let deadlines = |lair| {
        envelope
            .sources
            .iter()
            .filter(|source| matches!(source.kind, SourceKind::PossibleSpawn { lair: value } if value == lair))
            .map(|source| source.begins_at)
            .collect::<Vec<_>>()
    };
    assert_eq!(deadlines(0), vec![Tick(401); 4]);
    assert_eq!(deadlines(1), vec![Tick(175); 4]);
}

#[test]
fn rules_v9_disabled_lairs_use_absent_overlay_and_old_constructors_stay_closed() {
    let legacy = observation(Vec::new(), Tick(100));
    let environment = legacy.environment().unwrap().clone();
    let mut parts = legacy.into_parts();
    parts.rules.version = 9;
    let observed = Observation::new_rules_v9(parts.clone(), environment.clone(), None).unwrap();
    assert!(observed.respawn_lairs().is_none());
    assert!(Observation::new(parts.clone()).is_err());
    assert!(Observation::new_rules_v7(parts.clone(), environment.clone()).is_err());
    assert!(
        Observation::new_rules_v9(
            parts,
            environment,
            Some(ObservedRespawnLairs::new(
                RespawnLairConfig::Disabled,
                Vec::new(),
                Vec::new(),
            )),
        )
        .is_err()
    );
}

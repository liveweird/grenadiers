use crate::game::{
    Arena, CellId, EntityId, SpikePitPhase, SpikePitView, StormCloudConfig, Terrain, Tick,
    fixed_arena, spike_pit_layout_v1,
};
use crate::room::{SpikePitConfig, SpikePitSettings};

use super::tests::observation;
use super::*;

fn rules_v10_parts() -> ObservationParts {
    let mut parts = observation(BombKnowledge::Visible(Vec::new())).into_parts();
    parts.rules.version = 10;
    parts.arena = observed_arena(&fixed_arena());
    parts
}

fn observed_arena(arena: &Arena) -> ObservedArena {
    let terrain = (0..arena.height())
        .flat_map(|row| {
            (0..arena.width()).map(move |column| {
                arena
                    .terrain(CellId::new(column, row))
                    .expect("board iteration stays in bounds")
            })
        })
        .collect();
    ObservedArena {
        width: arena.width(),
        height: arena.height(),
        terrain,
        blocks: arena.blocks().iter().copied().collect(),
    }
}

fn environment(parts: &ObservationParts) -> ObservedEnvironment {
    ObservedEnvironment::new(
        StormCloudConfig::Disabled,
        None,
        parts
            .actors
            .iter()
            .map(|actor| matches!(actor, ObservedActor::Visible(_)).then_some(0))
            .collect(),
    )
}

fn enabled_pits() -> ObservedSpikePits {
    let arena = fixed_arena();
    enabled_pits_at(spike_pit_layout_v1(&arena, []))
}

fn enabled_pits_at(cells: impl IntoIterator<Item = CellId>) -> ObservedSpikePits {
    let pits = cells.into_iter().map(|cell| SpikePitView {
        cell,
        phase: SpikePitPhase::Idle,
        remaining_ticks: 0,
    });
    ObservedSpikePits::new(
        SpikePitConfig::Enabled(SpikePitSettings::DEFAULT),
        pits.collect(),
    )
}

#[test]
fn immutable_sites_remain_valid_after_preferred_cells_become_available() {
    let mut parts = rules_v10_parts();
    let original = enabled_pits();
    parts.arena.blocks.clear();

    let observed = construct(parts, Some(original.clone())).unwrap();
    assert_eq!(observed.spike_pits(), Some(&original));
}

#[test]
fn immutable_sites_do_not_require_a_concealed_initial_oil_exclusion() {
    let mut arena = fixed_arena();
    let removed = arena.blocks().clone();
    arena.remove_blocks(&removed);
    let current_preferred = spike_pit_layout_v1(&arena, []);
    let hidden_oil_site = current_preferred[0];
    let historical = spike_pit_layout_v1(&arena, [hidden_oil_site]);
    assert!(!historical.is_empty());
    assert_ne!(historical, current_preferred);

    let mut parts = rules_v10_parts();
    parts.arena = observed_arena(&arena);
    assert!(parts.oil.is_empty());
    let overlay = enabled_pits_at(historical);
    let observed = construct(parts, Some(overlay.clone())).unwrap();
    assert_eq!(observed.spike_pits(), Some(&overlay));
}

fn construct(
    parts: ObservationParts,
    pits: Option<ObservedSpikePits>,
) -> Result<Observation, ObservationError> {
    let environment = environment(&parts);
    Observation::new_rules_v10(parts, environment, None, pits)
}

#[test]
fn rules_v10_distinguishes_disabled_from_enabled_without_sites() {
    let mut parts = observation(BombKnowledge::Visible(Vec::new())).into_parts();
    parts.rules.version = 10;
    let disabled = construct(parts.clone(), None).unwrap();
    assert_eq!(disabled.spike_pits(), None);
    assert_eq!(
        rules_config(&disabled).unwrap().spike_pit_config(),
        SpikePitConfig::Disabled
    );

    let empty = ObservedSpikePits::new(
        SpikePitConfig::Enabled(SpikePitSettings::DEFAULT),
        Vec::new(),
    );
    let enabled = construct(parts, Some(empty)).unwrap();
    assert_eq!(enabled.spike_pits().unwrap().pits(), []);
    assert_eq!(
        rules_config(&enabled).unwrap().spike_pit_config(),
        SpikePitConfig::Enabled(SpikePitSettings::DEFAULT)
    );
}

#[test]
fn present_overlay_rejects_disabled_configuration_and_old_constructor_boundaries_hold() {
    let parts = rules_v10_parts();
    let environment = environment(&parts);
    assert_eq!(
        construct(
            parts.clone(),
            Some(ObservedSpikePits::new(SpikePitConfig::Disabled, Vec::new())),
        ),
        Err(ObservationError::InvalidActors)
    );
    assert_eq!(
        Observation::new(parts.clone()),
        Err(ObservationError::InvalidActors)
    );
    assert_eq!(
        Observation::new_rules_v9(parts.clone(), environment.clone(), None),
        Err(ObservationError::InvalidActors)
    );
    let mut rules_v9 = parts;
    rules_v9.rules.version = 9;
    assert_eq!(
        Observation::new_rules_v10(rules_v9, environment, None, None),
        Err(ObservationError::InvalidActors)
    );
}

#[test]
fn canonical_sites_and_phase_specific_clocks_are_accepted_and_preserved_by_mapping() {
    let parts = rules_v10_parts();
    let mut pits = enabled_pits().pits().to_vec();
    pits[0].phase = SpikePitPhase::Warning;
    pits[0].remaining_ticks = SpikePitSettings::DEFAULT.warning_ticks();
    pits[1].phase = SpikePitPhase::Raised;
    pits[1].remaining_ticks = SpikePitSettings::DEFAULT.raised_ticks();
    let overlay = ObservedSpikePits::new(SpikePitConfig::Enabled(SpikePitSettings::DEFAULT), pits);
    let observed = construct(parts, Some(overlay.clone())).unwrap();
    assert_eq!(observed.spike_pits(), Some(&overlay));

    let mapped = observed
        .map_parts(|parts| parts.rules.remaining_ticks -= 1)
        .unwrap();
    assert_eq!(mapped.spike_pits(), Some(&overlay));
}

#[test]
fn sites_must_be_unique_canonical_unblocked_floor_cells() {
    let parts = rules_v10_parts();
    let valid = enabled_pits();

    let mut out_of_order = valid.pits().to_vec();
    out_of_order.reverse();
    assert_eq!(
        construct(
            parts.clone(),
            Some(ObservedSpikePits::new(valid.config(), out_of_order))
        ),
        Err(ObservationError::InvalidActors)
    );

    assert_eq!(
        construct(parts.clone(), Some(enabled_pits_at([valid.pits()[0].cell]))),
        Err(ObservationError::InvalidActors)
    );

    let mut oversized = valid.pits().to_vec();
    oversized.push(SpikePitView {
        cell: CellId::new(7, 7),
        phase: SpikePitPhase::Idle,
        remaining_ticks: 0,
    });
    oversized.sort_by_key(|pit| pit.cell);
    assert_eq!(
        construct(
            parts.clone(),
            Some(ObservedSpikePits::new(valid.config(), oversized))
        ),
        Err(ObservationError::InvalidActors)
    );

    let mut duplicate = valid.pits().to_vec();
    duplicate[1] = duplicate[0];
    assert_eq!(
        construct(
            parts.clone(),
            Some(ObservedSpikePits::new(valid.config(), duplicate))
        ),
        Err(ObservationError::InvalidActors)
    );

    let mut noncanonical = valid.pits().to_vec();
    noncanonical[0].cell = CellId::new(1, 1);
    noncanonical.sort_by_key(|pit| pit.cell);
    assert_eq!(
        construct(
            parts.clone(),
            Some(ObservedSpikePits::new(valid.config(), noncanonical)),
        ),
        Err(ObservationError::InvalidActors)
    );

    let excluded_cell = valid.pits()[0].cell;
    let mut visible_oil = parts.clone();
    visible_oil.oil.push(ObservedOil {
        cell: excluded_cell,
        state: crate::game::OilBarrelState::Unlit,
    });
    assert_eq!(
        construct(visible_oil, Some(valid.clone())),
        Err(ObservationError::InvalidActors)
    );

    let mut blocked = parts;
    blocked.arena.blocks.push(excluded_cell);
    assert_eq!(
        construct(blocked, Some(valid)),
        Err(ObservationError::InvalidActors)
    );
}

#[test]
fn clock_bounds_and_future_deadline_overflow_are_rejected() {
    let parts = rules_v10_parts();
    let valid = enabled_pits();
    let mut invalid_remaining = valid.pits().to_vec();
    invalid_remaining[0].phase = SpikePitPhase::Idle;
    invalid_remaining[0].remaining_ticks = 1;
    assert_eq!(
        construct(
            parts.clone(),
            Some(ObservedSpikePits::new(valid.config(), invalid_remaining)),
        ),
        Err(ObservationError::InvalidActors)
    );

    let mut overflow_parts = parts;
    overflow_parts.tick = Tick(u64::MAX);
    let mut overflow = valid.pits().to_vec();
    overflow[0].phase = SpikePitPhase::Warning;
    overflow[0].remaining_ticks = 1;
    assert_eq!(
        construct(
            overflow_parts,
            Some(ObservedSpikePits::new(valid.config(), overflow)),
        ),
        Err(ObservationError::InvalidActors)
    );
}

#[test]
fn rules_v10_bombs_keep_rules_v9_distinct_cell_mechanics() {
    let mut parts = rules_v10_parts();
    let bomb = ObservedBomb {
        id: EntityId(20),
        placed_at: Tick(0),
        cell: CellId::new(1, 1),
        owner: EntityId(2),
        range: 2,
        trigger: ObservedBombTrigger::Remote,
        blast: crate::game::BlastKind::Standard,
        unknown: UnknownBombFacts::ALL,
    };
    parts.bombs = BombKnowledge::Visible(vec![
        bomb,
        ObservedBomb {
            id: EntityId(21),
            ..bomb
        },
    ]);
    assert_eq!(
        construct(
            parts,
            Some(ObservedSpikePits::new(
                SpikePitConfig::Enabled(SpikePitSettings::DEFAULT),
                Vec::new(),
            ))
        ),
        Err(ObservationError::InvalidBombs)
    );
}

#[test]
fn non_floor_site_is_rejected_even_when_it_is_on_the_board() {
    let parts = rules_v10_parts();
    let overlay = ObservedSpikePits::new(
        SpikePitConfig::Enabled(SpikePitSettings::DEFAULT),
        vec![SpikePitView {
            cell: CellId::new(0, 0),
            phase: SpikePitPhase::Idle,
            remaining_ticks: 0,
        }],
    );
    assert_eq!(
        construct(parts, Some(overlay)),
        Err(ObservationError::InvalidActors)
    );
    assert_eq!(
        Terrain::Wall,
        fixed_arena().terrain(CellId::new(0, 0)).unwrap()
    );
}

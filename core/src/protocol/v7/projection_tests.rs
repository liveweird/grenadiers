use crate::game::EventAudience;
use crate::protocol::{Cell, EntityId, RulesVersion, Seat, Tick};

use super::projection_facts::{
    BoundPublication, ClassicCapture, EventFacts, ProjectionError, ReportFacts,
};
use super::projection_test_support::{base_world, classic_state, room};
use super::projection_validation::{validate_reports, validate_world};
use super::*;

#[test]
fn actual_classic_capture_binds_and_prepares_equal_item_empty_views() {
    let state = classic_state();
    let capture = capture_classic(&state, &[]).unwrap();
    let room = room(
        0,
        GameSettings::for_rules(RulesVersion::ClassicV1),
        ItemSet::empty(),
    );
    let bound = bind_classic(&room, room.round_id, &capture).unwrap();
    let [one, two] = prepare_snapshot_pair(&bound).unwrap();
    assert_eq!(one, two);
    let json: serde_json::Value = serde_json::from_str(&one).unwrap();
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../../../../api/gameplay-v7.schema.json")).unwrap();
    assert!(jsonschema::validator_for(&schema).unwrap().is_valid(&json));
    assert_eq!(json["room"]["availableItems"], serde_json::json!([]));
    assert_eq!(
        json["room"]["settings"]["enabledItems"],
        serde_json::json!([])
    );
    assert_eq!(
        json["state"]["actors"][0]["inventory"],
        serde_json::json!({"equipment": null, "weapon": null})
    );
}

#[test]
fn future_items_and_triggers_validate_unbound_but_cannot_certify_classic() {
    let items = ItemSet::new([
        ItemId::RemoteBomb,
        ItemId::Landmine,
        ItemId::InvisibilityCloak,
        ItemId::InvisibleBombs,
    ])
    .unwrap();
    let mut future = base_world(10);
    future.available_items = items;
    future.enabled_items = items;
    future.actors[0].effects = EffectViews::new(vec![
        EffectView::new(EffectKind::InvisibleBombs, 9).unwrap(),
    ])
    .unwrap();
    future.actors[1].effects = EffectViews::new(vec![
        EffectView::new(EffectKind::InvisibilityCloak, 9).unwrap(),
    ])
    .unwrap();
    future.actors[0].live_bomb_count = 1;
    future.actors[1].live_bomb_count = 1;
    future.bombs = vec![
        BombView {
            id: EntityId::new(3),
            cell: Cell::new(2, 2).unwrap(),
            owner: EntityId::new(1),
            range: 2,
            placed_at: Tick::new(9).unwrap(),
            trigger: BombTrigger::remote(),
            blast: BlastKind::Standard,
        },
        BombView {
            id: EntityId::new(4),
            cell: Cell::new(3, 2).unwrap(),
            owner: EntityId::new(2),
            range: 2,
            placed_at: Tick::new(9).unwrap(),
            trigger: BombTrigger::contact(),
            blast: BlastKind::Standard,
        },
    ];
    let validated = validate_world(future).unwrap();
    let original = capture_classic(&classic_state(), &[]).unwrap();
    let forged = ClassicCapture {
        world: validated,
        ..original
    };
    let matching = room(0, GameSettings::new(0, 128, 3, 180, items).unwrap(), items);
    assert_eq!(
        bind_classic(&matching, matching.round_id, &forged),
        Err(ProjectionError::InvalidClassicBinding)
    );
}

#[test]
fn complete_validation_rejects_hidden_invalid_bombs_and_effect_semantics() {
    let items = ItemSet::new([ItemId::InvisibleBombs, ItemId::RemoteBomb]).unwrap();
    let mut hidden_invalid = base_world(10);
    hidden_invalid.available_items = items;
    hidden_invalid.enabled_items = items;
    hidden_invalid.actors[0].effects = EffectViews::new(vec![
        EffectView::new(EffectKind::InvisibleBombs, 10).unwrap(),
    ])
    .unwrap();
    hidden_invalid.actors[0].live_bomb_count = 1;
    hidden_invalid.bombs.push(BombView {
        id: EntityId::new(3),
        cell: Cell::new(2, 2).unwrap(),
        owner: EntityId::new(99),
        range: 2,
        placed_at: Tick::new(9).unwrap(),
        trigger: BombTrigger::remote(),
        blast: BlastKind::Standard,
    });
    assert_eq!(
        validate_world(hidden_invalid),
        Err(ProjectionError::InvalidBombs)
    );

    let mut effects = base_world(10);
    effects.available_items = ItemSet::new([ItemId::SlowFoot, ItemId::Shield]).unwrap();
    effects.enabled_items = effects.available_items;
    effects.actors[0].effects = EffectViews::new(
        (0..6)
            .map(|_| EffectView::new(EffectKind::SlowFoot, 1).unwrap())
            .collect(),
    )
    .unwrap();
    assert!(validate_world(effects.clone()).is_ok());
    effects.actors[0].effects = EffectViews::new(
        (0..7)
            .map(|_| EffectView::new(EffectKind::SlowFoot, 1).unwrap())
            .collect(),
    )
    .unwrap();
    assert_eq!(
        validate_world(effects.clone()),
        Err(ProjectionError::InvalidActors)
    );
    effects.actors[0].effects =
        EffectViews::new(vec![EffectView::new(EffectKind::Shield, 1).unwrap(); 2]).unwrap();
    assert_eq!(validate_world(effects), Err(ProjectionError::InvalidActors));
}

#[test]
fn report_validation_is_unfiltered_and_recipient_indices_restart_per_tick() {
    let world = validate_world(base_world(3)).unwrap();
    let kind = EventKind::PickupCollected {
        id: EntityId::new(1),
        seat: Seat::One,
        cell: Cell::new(1, 1).unwrap(),
        kind: PickupKind::BombCapacity,
    };
    let reports = vec![
        ReportFacts {
            tick: Tick::new(2).unwrap(),
            events: [
                EventAudience::Neither,
                EventAudience::One,
                EventAudience::Both,
            ]
            .map(|audience| EventFacts {
                tick: Tick::new(2).unwrap(),
                audience,
                kind,
            })
            .into(),
        },
        ReportFacts {
            tick: Tick::new(3).unwrap(),
            events: [EventAudience::Two, EventAudience::Both]
                .map(|audience| EventFacts {
                    tick: Tick::new(3).unwrap(),
                    audience,
                    kind,
                })
                .into(),
        },
    ];
    let reports = validate_reports(&world, reports).unwrap();
    let bound = BoundPublication {
        room: room(
            3,
            GameSettings::for_rules(RulesVersion::ClassicV1),
            ItemSet::empty(),
        ),
        world,
        reports,
    };
    let [one, two] = prepare_snapshot_pair(&bound).unwrap();
    let one: serde_json::Value = serde_json::from_str(&one).unwrap();
    let two: serde_json::Value = serde_json::from_str(&two).unwrap();
    assert_eq!(
        one["events"]
            .as_array()
            .unwrap()
            .iter()
            .map(|event| (
                event["tick"].as_u64().unwrap(),
                event["index"].as_u64().unwrap()
            ))
            .collect::<Vec<_>>(),
        vec![(2, 0), (2, 1), (3, 0)]
    );
    assert_eq!(
        two["events"]
            .as_array()
            .unwrap()
            .iter()
            .map(|event| (
                event["tick"].as_u64().unwrap(),
                event["index"].as_u64().unwrap()
            ))
            .collect::<Vec<_>>(),
        vec![(2, 0), (3, 0), (3, 1)]
    );
}

#[test]
fn cloak_is_opponent_only_and_bomb_blindness_hides_every_bomb() {
    let items = ItemSet::new([ItemId::InvisibilityCloak, ItemId::InvisibleBombs]).unwrap();
    let mut facts = base_world(10);
    facts.available_items = items;
    facts.enabled_items = items;
    facts.actors[0].effects = EffectViews::new(vec![
        EffectView::new(EffectKind::InvisibleBombs, 1).unwrap(),
    ])
    .unwrap();
    facts.actors[1].effects = EffectViews::new(vec![
        EffectView::new(EffectKind::InvisibilityCloak, 1).unwrap(),
    ])
    .unwrap();
    facts.actors[1].live_bomb_count = 1;
    facts.bombs.push(BombView {
        id: EntityId::new(3),
        cell: Cell::new(2, 2).unwrap(),
        owner: EntityId::new(2),
        range: 2,
        placed_at: Tick::new(9).unwrap(),
        trigger: BombTrigger::timed(10, false).unwrap(),
        blast: BlastKind::Standard,
    });
    let bound = BoundPublication {
        room: room(10, GameSettings::new(0, 128, 3, 180, items).unwrap(), items),
        world: validate_world(facts).unwrap(),
        reports: validate_reports(&validate_world(base_world(10)).unwrap(), vec![]).unwrap(),
    };
    let [one, two] = prepare_snapshot_pair(&bound).unwrap();
    let one: serde_json::Value = serde_json::from_str(&one).unwrap();
    let two: serde_json::Value = serde_json::from_str(&two).unwrap();
    assert_eq!(one["state"]["actors"][0]["type"], "visible");
    assert_eq!(one["state"]["actors"][1]["type"], "hidden");
    assert_eq!(one["state"]["bombs"], serde_json::json!([]));
    assert_eq!(two["state"]["actors"][1]["type"], "visible");
    assert_eq!(two["state"]["bombs"].as_array().unwrap().len(), 1);
}

#[test]
fn report_window_rejects_four_nonconsecutive_and_ninety_seven_without_truncation() {
    let world = validate_world(base_world(100)).unwrap();
    let empty = |tick| ReportFacts {
        tick: Tick::new(tick).unwrap(),
        events: vec![],
    };
    assert_eq!(
        validate_reports(&world, vec![empty(1), empty(2), empty(3), empty(4)]),
        Err(ProjectionError::InvalidReports)
    );
    assert_eq!(
        validate_reports(&world, vec![empty(1), empty(3)]),
        Err(ProjectionError::InvalidReports)
    );
    let kind = EventKind::PickupCollected {
        id: EntityId::new(1),
        seat: Seat::One,
        cell: Cell::new(1, 1).unwrap(),
        kind: PickupKind::BombCapacity,
    };
    let events = (0..97)
        .map(|_| EventFacts {
            tick: Tick::new(100).unwrap(),
            audience: EventAudience::Neither,
            kind,
        })
        .collect();
    assert_eq!(
        validate_reports(
            &world,
            vec![ReportFacts {
                tick: Tick::new(100).unwrap(),
                events
            }]
        ),
        Err(ProjectionError::InvalidReports)
    );
}

#[test]
fn frozen_clock_grace_maximum_and_positive_fuse_chain_are_valid_facts() {
    let mut facts = base_world(10);
    facts.actors[0].damage_grace_remaining_ticks = DamageGraceTicks::new(60).unwrap();
    facts.available_items = ItemSet::new([ItemId::Heart]).unwrap();
    facts.enabled_items = facts.available_items;
    let world = validate_world(facts).unwrap();
    let bomb = BombView {
        id: EntityId::new(3),
        cell: Cell::new(2, 2).unwrap(),
        owner: EntityId::new(1),
        range: 2,
        placed_at: Tick::new(5).unwrap(),
        trigger: BombTrigger::timed(5, false).unwrap(),
        blast: BlastKind::Standard,
    };
    let report = ReportFacts {
        tick: Tick::new(10).unwrap(),
        events: vec![EventFacts {
            tick: Tick::new(10).unwrap(),
            audience: EventAudience::Both,
            kind: EventKind::BombDetonated { bomb },
        }],
    };
    assert!(validate_reports(&world, vec![report]).is_ok());
}

#[test]
fn second_recipient_encoding_failure_returns_no_partial_pair() {
    let mut facts = base_world(10);
    facts.actors[0].effects = EffectViews::new(vec![
        EffectView::new(EffectKind::InvisibleBombs, 1).unwrap(),
    ])
    .unwrap();
    let bomb = BombView {
        id: EntityId::new(3),
        cell: Cell::new(2, 2).unwrap(),
        owner: EntityId::new(2),
        range: 2,
        placed_at: Tick::new(9).unwrap(),
        trigger: BombTrigger::timed(10, false).unwrap(),
        blast: BlastKind::Standard,
    };
    facts.bombs = vec![bomb; 2_000];
    let items = ItemSet::new([ItemId::InvisibleBombs]).unwrap();
    let publication = BoundPublication {
        room: room(10, GameSettings::new(0, 128, 3, 180, items).unwrap(), items),
        world: ValidatedWorld(facts),
        reports: ValidatedReports(vec![]),
    };
    assert_eq!(
        prepare_snapshot_pair(&publication),
        Err(ProjectionError::EncodingFailed)
    );
}

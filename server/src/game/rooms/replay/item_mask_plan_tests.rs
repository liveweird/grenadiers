use std::ffi::OsString;

use super::{
    GateContext, ItemMaskPlan, SOURCE_ENV, cardinality, enabled_item_plan, optional_unicode_env,
    parse_optional_unicode, parse_usize, validate_profile, validate_source,
};

#[test]
fn rejects_invalid_catalogs_and_cardinality_overflow() {
    assert!(ItemMaskPlan::checked(&[1, 4], 5, 2).is_ok());
    for (catalog, union) in [(&[0, 1][..], 1), (&[1, 3][..], 3), (&[1, 1][..], 1)] {
        assert!(ItemMaskPlan::checked(catalog, union, 2).is_err());
    }
    assert!(ItemMaskPlan::checked(&[1, 4], 7, 2).is_err());
    assert!(ItemMaskPlan::checked(&[1, 4], 5, 0).is_err());
    assert!(cardinality(usize::BITS as usize).is_err());
}

#[test]
fn checked_enabled_items_catalog_is_authoritative() {
    let plan = enabled_item_plan();
    let union = plan.catalog.iter().copied().fold(0, |all, bit| all | bit);
    assert_eq!(union, grenadiers_core::game::EnabledItems::ALL.bits());
    assert_eq!(plan.total, 1_usize << plan.catalog.len());
    assert_eq!(plan.partition_count, plan.total.div_ceil(plan.capacity));
}

#[test]
fn sparse_catalog_ordinals_map_to_exact_bits() {
    let plan = ItemMaskPlan::checked(&[1, 8, 32], 41, 8).unwrap();
    assert_eq!(
        (0..8).map(|n| plan.enabled_bits(n)).collect::<Vec<_>>(),
        [0, 1, 8, 9, 32, 33, 40, 41]
    );
}

#[test]
fn selection_is_exhaustive_by_default_and_rejects_bad_input() {
    let plan = ItemMaskPlan::checked(&[1, 4, 16], 21, 3).unwrap();
    assert_eq!(plan.select(None, None).unwrap().range, 0..8);
    for pair in [
        (Some("0"), None),
        (None, Some("3")),
        (Some("0"), Some("2")),
        (Some("3"), Some("3")),
        (Some("x"), Some("3")),
        (Some("00"), Some("3")),
    ] {
        assert!(plan.select(pair.0, pair.1).is_err(), "{pair:?}");
    }
    for value in ["", "+1", "-1", "01", " 1", "1 ", "184467440737095516160"] {
        assert!(parse_usize("TEST", value).is_err(), "{value}");
    }
}

#[test]
fn ranges_are_disjoint_and_cover_every_ordinal_once() {
    let plan = ItemMaskPlan::checked(&[1, 4, 16], 21, 3).unwrap();
    let ranges = (0..3)
        .map(|index| {
            plan.select(Some(&index.to_string()), Some("3"))
                .unwrap()
                .range
        })
        .collect::<Vec<_>>();
    assert_eq!(ranges, [0..3, 3..6, 6..8]);
    assert_eq!(
        ranges.into_iter().flatten().collect::<Vec<_>>(),
        (0..8).collect::<Vec<_>>()
    );
}

#[test]
fn gate_context_accepts_only_explicit_canonical_values() {
    assert!(validate_source(None).is_ok());
    assert!(validate_source(Some("0123456789abcdef0123456789abcdef01234567")).is_ok());
    for value in [
        "",
        "ABCDEF0123456789abcdef0123456789abcdef01",
        "g123456789abcdef0123456789abcdef01234567",
    ] {
        assert!(validate_source(Some(value)).is_err(), "{value}");
    }
    assert!(validate_profile(None).is_ok());
    for value in ["debug", "release", "coverage"] {
        assert!(validate_profile(Some(value)).is_ok());
    }
    for value in ["test", "cov", ""] {
        assert!(validate_profile(Some(value)).is_err());
    }
}

#[cfg(unix)]
#[test]
fn non_utf8_environment_values_are_rejected() {
    use std::os::unix::ffi::OsStringExt;
    assert!(parse_optional_unicode("TEST", Some(OsString::from_vec(vec![0xff]))).is_err());
}

#[test]
fn export_checked_plan_for_ci() {
    let plan = enabled_item_plan();
    match optional_unicode_env("GRENADIERS_SERVER_REPLAY_ITEM_MASK_EXPORT_PLAN")
        .unwrap()
        .as_deref()
    {
        None => return,
        Some("1") => {}
        Some(_) => panic!("GRENADIERS_SERVER_REPLAY_ITEM_MASK_EXPORT_PLAN must equal 1"),
    }
    let source = optional_unicode_env(SOURCE_ENV)
        .unwrap()
        .expect("planner source is required");
    plan.emit_plan(&source);
}

#[test]
#[should_panic(expected = "assertion `left == right` failed")]
fn completion_rejects_observed_count_before_reporting() {
    let plan = ItemMaskPlan::checked(&[1, 4], 5, 4).unwrap();
    let selection = plan.select(None, None).unwrap();
    let context = GateContext {
        source: None,
        profile: None,
    };
    plan.emit_completion(&selection, &context, selection.range.len() - 1);
}

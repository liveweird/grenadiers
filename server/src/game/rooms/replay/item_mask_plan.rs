use std::{env, ffi::OsString, fmt, ops::Range};

use serde::Serialize;

pub(super) const PARTITION_CAPACITY: usize = 131_072;
const INDEX_ENV: &str = "GRENADIERS_SERVER_REPLAY_ITEM_MASK_PARTITION_INDEX";
const COUNT_ENV: &str = "GRENADIERS_SERVER_REPLAY_ITEM_MASK_PARTITION_COUNT";
const SOURCE_ENV: &str = "GRENADIERS_SERVER_REPLAY_ITEM_MASK_SOURCE";
const PROFILE_ENV: &str = "GRENADIERS_SERVER_REPLAY_ITEM_MASK_PROFILE";

#[derive(Debug, PartialEq, Eq)]
pub(super) struct PlanError(String);

impl fmt::Display for PlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug)]
pub(super) struct ItemMaskPlan {
    catalog: Vec<u32>,
    total: usize,
    partition_count: usize,
    capacity: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Selection {
    pub(super) partition_index: Option<usize>,
    pub(super) range: Range<usize>,
}

#[derive(Debug)]
pub(super) struct GateContext {
    source: Option<String>,
    profile: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CompletionSummary<'a> {
    schema_version: u8,
    source: Option<&'a str>,
    profile: Option<&'a str>,
    selection: &'static str,
    partition_index: Option<usize>,
    partition_count: usize,
    start_ordinal: usize,
    end_ordinal_exclusive: usize,
    observed_count: usize,
    total_cardinality: usize,
}

impl ItemMaskPlan {
    pub(super) fn checked(
        catalog: &[u32],
        expected_union: u32,
        capacity: usize,
    ) -> Result<Self, PlanError> {
        if catalog.is_empty() {
            return Err(PlanError("item mask catalog must not be empty".into()));
        }
        if capacity == 0 {
            return Err(PlanError(
                "item mask partition capacity must be positive".into(),
            ));
        }
        let mut union = 0_u32;
        for &bit in catalog {
            if bit == 0 || !bit.is_power_of_two() {
                return Err(PlanError(format!(
                    "item mask catalog entry {bit} is not a nonzero single bit"
                )));
            }
            if union & bit != 0 {
                return Err(PlanError(format!(
                    "item mask catalog entry {bit} is duplicated"
                )));
            }
            union |= bit;
        }
        if union != expected_union {
            return Err(PlanError(format!(
                "item mask catalog union {union} does not equal enabled-items ALL {expected_union}"
            )));
        }
        let total = cardinality(catalog.len())?;
        let partition_count = total.div_ceil(capacity);
        Ok(Self {
            catalog: catalog.to_vec(),
            total,
            partition_count,
            capacity,
        })
    }

    pub(super) fn selection_from_environment(&self) -> Result<Selection, PlanError> {
        self.select(
            optional_unicode_env(INDEX_ENV)?.as_deref(),
            optional_unicode_env(COUNT_ENV)?.as_deref(),
        )
    }

    pub(super) fn context_from_environment() -> Result<GateContext, PlanError> {
        let source = optional_unicode_env(SOURCE_ENV)?;
        let profile = optional_unicode_env(PROFILE_ENV)?;
        validate_source(source.as_deref())?;
        validate_profile(profile.as_deref())?;
        Ok(GateContext { source, profile })
    }

    fn select(&self, index: Option<&str>, count: Option<&str>) -> Result<Selection, PlanError> {
        match (index, count) {
            (None, None) => Ok(Selection {
                partition_index: None,
                range: 0..self.total,
            }),
            (Some(_), None) | (None, Some(_)) => Err(PlanError(format!(
                "{INDEX_ENV} and {COUNT_ENV} must be provided together"
            ))),
            (Some(index), Some(count)) => {
                let index = parse_usize(INDEX_ENV, index)?;
                let count = parse_usize(COUNT_ENV, count)?;
                if count != self.partition_count {
                    return Err(PlanError(format!(
                        "{COUNT_ENV}={count} does not equal checked plan count {}",
                        self.partition_count
                    )));
                }
                if index >= count {
                    return Err(PlanError(format!(
                        "{INDEX_ENV}={index} is outside partition count {count}"
                    )));
                }
                Ok(Selection {
                    partition_index: Some(index),
                    range: self.range(index),
                })
            }
        }
    }

    pub(super) fn enabled_bits(&self, ordinal: usize) -> u32 {
        assert!(ordinal < self.total);
        self.catalog
            .iter()
            .enumerate()
            .filter(|(index, _)| ordinal & (1 << index) != 0)
            .fold(0, |enabled, (_, bit)| enabled | bit)
    }

    pub(super) fn emit_completion(
        &self,
        selection: &Selection,
        context: &GateContext,
        observed: usize,
    ) {
        assert_eq!(observed, selection.range.len());
        let summary = CompletionSummary {
            schema_version: 1,
            source: context.source.as_deref(),
            profile: context.profile.as_deref(),
            selection: if selection.partition_index.is_some() {
                "partition"
            } else {
                "all"
            },
            partition_index: selection.partition_index,
            partition_count: self.partition_count,
            start_ordinal: selection.range.start,
            end_ordinal_exclusive: selection.range.end,
            observed_count: observed,
            total_cardinality: self.total,
        };
        println!(
            "GRENADIERS_SERVER_REPLAY_ITEM_MASK_SUMMARY={}",
            serde_json::to_string(&summary).expect("completion summary must serialize")
        );
    }

    fn range(&self, index: usize) -> Range<usize> {
        let start = index * self.capacity;
        start..(start + self.capacity).min(self.total)
    }
}

fn cardinality(item_count: usize) -> Result<usize, PlanError> {
    1_usize
        .checked_shl(item_count.try_into().unwrap_or(u32::MAX))
        .ok_or_else(|| {
            PlanError(format!(
                "item mask catalog length {item_count} overflows cardinality"
            ))
        })
}

fn parse_usize(name: &str, value: &str) -> Result<usize, PlanError> {
    if value.is_empty()
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return Err(PlanError(format!(
            "{name} must be canonical unsigned decimal"
        )));
    }
    value
        .parse()
        .map_err(|_| PlanError(format!("{name} is outside usize range")))
}

fn optional_unicode_env(name: &str) -> Result<Option<String>, PlanError> {
    parse_optional_unicode(name, env::var_os(name))
}

fn parse_optional_unicode(
    name: &str,
    value: Option<OsString>,
) -> Result<Option<String>, PlanError> {
    value
        .map(|value: OsString| {
            value
                .into_string()
                .map_err(|_| PlanError(format!("{name} must be valid UTF-8")))
        })
        .transpose()
}

fn validate_source(source: Option<&str>) -> Result<(), PlanError> {
    if source.is_none_or(|value| {
        value.len() == 40
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    }) {
        Ok(())
    } else {
        Err(PlanError(format!(
            "{SOURCE_ENV} must be a 40-character lowercase hexadecimal commit"
        )))
    }
}

fn validate_profile(profile: Option<&str>) -> Result<(), PlanError> {
    if profile.is_none_or(|value| matches!(value, "debug" | "release" | "coverage")) {
        Ok(())
    } else {
        Err(PlanError(format!(
            "{PROFILE_ENV} must be debug, release, or coverage"
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::{
        ItemMaskPlan, cardinality, parse_optional_unicode, parse_usize, validate_profile,
        validate_source,
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
    fn sparse_catalog_ordinals_map_to_exact_bits() {
        let plan = ItemMaskPlan::checked(&[1, 8, 32], 41, 8).unwrap();
        assert_eq!(
            (0..8)
                .map(|ordinal| plan.enabled_bits(ordinal))
                .collect::<Vec<_>>(),
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
    #[should_panic(expected = "assertion `left == right` failed")]
    fn completion_rejects_observed_count_before_reporting() {
        let plan = ItemMaskPlan::checked(&[1, 4], 5, 4).unwrap();
        let selection = plan.select(None, None).unwrap();
        let context = super::GateContext {
            source: None,
            profile: None,
        };
        plan.emit_completion(&selection, &context, selection.range.len() - 1);
    }
}

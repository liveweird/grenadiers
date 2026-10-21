use std::{env, ffi::OsString, fmt, ops::Range};

use grenadiers_core::game::EnabledItems;
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlanSummary<'a> {
    schema_version: u8,
    source: &'a str,
    catalog: &'a [u32],
    capacity: usize,
    total_cardinality: usize,
    partition_count: usize,
    ranges: Vec<PlanRange>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlanRange {
    index: usize,
    start_ordinal: usize,
    end_ordinal_exclusive: usize,
}

pub(super) fn enabled_item_plan() -> ItemMaskPlan {
    let all = EnabledItems::ALL.bits();
    let catalog = (0..u32::BITS)
        .map(|index| 1_u32 << index)
        .filter(|bit| all & bit != 0)
        .collect::<Vec<_>>();
    ItemMaskPlan::checked(&catalog, all, PARTITION_CAPACITY)
        .unwrap_or_else(|error| panic!("invalid enabled-items catalog: {error}"))
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

    fn emit_plan(&self, source: &str) {
        validate_source(Some(source)).unwrap_or_else(|error| panic!("{error}"));
        let summary = PlanSummary {
            schema_version: 1,
            source,
            catalog: &self.catalog,
            capacity: self.capacity,
            total_cardinality: self.total,
            partition_count: self.partition_count,
            ranges: (0..self.partition_count)
                .map(|index| {
                    let range = self.range(index);
                    PlanRange {
                        index,
                        start_ordinal: range.start,
                        end_ordinal_exclusive: range.end,
                    }
                })
                .collect(),
        };
        println!(
            "GRENADIERS_SERVER_REPLAY_ITEM_MASK_PLAN={}",
            serde_json::to_string(&summary).expect("item-mask plan must serialize")
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
#[path = "item_mask_plan_tests.rs"]
mod tests;

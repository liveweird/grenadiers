use std::time::Duration;

use serde_json::{Value, json};

mod construction;
mod entropy;
mod registry;
mod registry_support;

pub(super) const RUNTIME_P99_BUDGET: Duration = Duration::from_micros(8_330);
pub(super) const OWNER_LAG_P99_BUDGET: Duration = Duration::from_millis(50);

pub(super) fn require_release() {
    assert!(
        !std::hint::black_box(cfg!(debug_assertions)),
        "measurement tests require cargo test --release"
    );
}

pub(super) fn summary(samples: &[Duration]) -> Value {
    assert!(!samples.is_empty(), "measurement sample must not be empty");
    let mut micros = samples
        .iter()
        .map(|sample| u64::try_from(sample.as_micros()).unwrap_or(u64::MAX))
        .collect::<Vec<_>>();
    micros.sort_unstable();
    json!({
        "count": micros.len(),
        "p50Us": percentile(&micros, 50),
        "p95Us": percentile(&micros, 95),
        "p99Us": percentile(&micros, 99),
        "maxUs": micros.last().copied().unwrap(),
    })
}

pub(super) fn summary_ns(samples: &[Duration]) -> Value {
    assert!(!samples.is_empty(), "measurement sample must not be empty");
    let mut nanos = samples.iter().map(Duration::as_nanos).collect::<Vec<_>>();
    nanos.sort_unstable();
    json!({
        "count": nanos.len(),
        "p50Ns": percentile(&nanos, 50),
        "p95Ns": percentile(&nanos, 95),
        "p99Ns": percentile(&nanos, 99),
        "maxNs": nanos.last().copied().unwrap(),
    })
}

pub(super) fn metadata() -> Value {
    json!({
        "sourceIdentity": required_metadata("GRENADIERS_MEASUREMENT_SOURCE"),
        "toolchain": required_metadata("GRENADIERS_MEASUREMENT_TOOLCHAIN"),
        "invocationDeclaredCargoProfile": "release",
        "debugAssertions": cfg!(debug_assertions),
        "roomProfile": "ExpandedV5",
        "recordingPolicy": "production-disabled",
        "timingUnit": "nanoseconds; unit does not imply clock accuracy",
    })
}

fn required_metadata(name: &str) -> String {
    let value =
        std::env::var(name).unwrap_or_else(|_| panic!("{name} must be explicit for measurements"));
    assert!(
        !value.trim().is_empty(),
        "{name} must not be blank for measurements"
    );
    value
}

pub(super) fn percentile_duration(samples: &[Duration], percentile_value: usize) -> Duration {
    let mut nanos = samples.iter().map(Duration::as_nanos).collect::<Vec<_>>();
    nanos.sort_unstable();
    let value = percentile(&nanos, percentile_value);
    Duration::from_nanos(u64::try_from(value).unwrap_or(u64::MAX))
}

fn percentile<T: Copy>(sorted: &[T], percentile_value: usize) -> T {
    assert!(!sorted.is_empty());
    assert!((1..=100).contains(&percentile_value));
    let rank = (percentile_value * sorted.len()).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

pub(super) fn emit(kind: &str, payload: Value) {
    let record = json!({
        "schema": 1,
        "kind": kind,
        "optimized": !cfg!(debug_assertions),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "observation": payload,
    });
    println!("M7_MEASUREMENT_JSON={record}");
}

#[test]
fn nearest_rank_percentiles_keep_tail_samples_visible() {
    let samples = (1..=100).map(Duration::from_micros).collect::<Vec<_>>();
    let value = summary(&samples);

    assert_eq!(value["p50Us"], 50);
    assert_eq!(value["p95Us"], 95);
    assert_eq!(value["p99Us"], 99);
    assert_eq!(value["maxUs"], 100);
    assert_eq!(percentile_duration(&samples, 99), Duration::from_micros(99));
    assert_eq!(summary_ns(&samples)["p99Ns"], 99_000);
}

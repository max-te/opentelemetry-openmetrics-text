use std::time::SystemTime;

use insta::assert_snapshot;
use opentelemetry::time::now;
use opentelemetry_openmetrics::convert::WriteOpenMetrics;

use crate::testsupport::make_test_metrics;

#[test]
fn matches_snapshot() {
    let metrics = make_test_metrics();
    let time = now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();
    let formatted = metrics.to_openmetrics_string().unwrap();
    let redacted = formatted
        .lines()
        .map(|line| {
            line.split_whitespace()
                .map(|t| String::from(if t.starts_with(&time) { "NOW.sthg" } else { t }))
                .reduce(|acc, e| format!("{acc} {e}"))
                .unwrap_or_default()
        })
        .collect::<Vec<String>>()
        .join("\n");
    assert_snapshot!(redacted);
}

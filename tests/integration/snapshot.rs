use std::time::SystemTime;

use insta::assert_snapshot;
use opentelemetry_openmetrics::convert::WriteOpenMetrics;

use crate::testsupport::make_test_metrics;

#[test]
fn matches_snapshot() {
    let (metrics, erasable_timestamps) = make_test_metrics();
    let mut formatted = metrics.to_openmetrics_string().unwrap();
    for (i, ts) in erasable_timestamps.iter().enumerate() {
        let ts = ts.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64().to_string();
        formatted = formatted.replace(&ts, &format!("<TIMESTAMP_{}>", i));
    }
    assert_snapshot!(formatted);
}

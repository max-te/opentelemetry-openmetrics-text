use std::time::SystemTime;

use insta::assert_snapshot;
use opentelemetry_openmetrics::convert::WriteOpenMetrics;

use testsupport::resource_metrics::make_test_metrics;
use testsupport::timestamps::get_all_timestamps;

#[test]
fn matches_snapshot() {
    let metrics = make_test_metrics();
    let erasable_timestamps = get_all_timestamps(&metrics);
    let mut formatted = metrics.to_openmetrics_string().unwrap();
    for (i, ts) in erasable_timestamps.iter().enumerate() {
        let ts = ts
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
            .to_string();
        formatted = formatted.replace(&ts, &format!("<TIMESTAMP_{}>", i));
    }
    assert_snapshot!(formatted);
}

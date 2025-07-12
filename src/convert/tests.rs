use std::fmt::Write;

use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::data::ScopeMetrics;

use super::*;

#[path = "../../tests/integration/testsupport.rs"]
mod testsupport;
use testsupport::make_test_metrics;

#[test]
fn test_write_sanitized_name() {
    let mut output = String::new();

    // Test with valid name
    write_sanitized_name(&mut output, "valid_metric_name").unwrap();
    assert_eq!(output, "valid_metric_name");

    // Test with name containing invalid characters
    output.clear();
    write_sanitized_name(&mut output, "invalid._ä.metric-name").unwrap();
    assert_eq!(output, "invalid_metric_name");

    // Test with name starting with digit
    output.clear();
    write_sanitized_name(&mut output, "1.metric").unwrap();
    assert_eq!(output, "_1_metric");
}

#[test]
fn test_write_escaped() {
    let mut output = String::new();

    // Test with string containing characters that need escaping
    write_escaped(
        &mut output,
        "Line 1\nLine 2\tTabbed\r\nWindows \"quoted\" \\ BS ❤️‍🩹",
    )
    .unwrap();
    assert_eq!(
        output,
        "Line 1\\nLine 2\tTabbed\r\\nWindows \\\"quoted\\\" \\\\ BS ❤️‍🩹"
    );

    // Test with string not needing escaping
    output.clear();
    write_escaped(&mut output, "Simple string").unwrap();
    assert_eq!(output, "Simple string");
}

#[test]
fn test_hash_attrs() {
    let attrs = vec![
        KeyValue::new("key1", "value1"),
        KeyValue::new("key2", "value2"),
    ];

    let hash1 = hash_attrs(attrs.iter());

    // Same attributes should produce same hash, order does not matter
    let attrs2 = vec![
        KeyValue::new("key2", "value2"),
        KeyValue::new("key1", "value1"),
    ];
    let hash2 = hash_attrs(attrs2.iter());

    assert_eq!(hash1, hash2);

    // Different attributes should produce different hash
    let attrs3 = vec![
        KeyValue::new("key1", "value1"),
        KeyValue::new("key2", "different"),
    ];
    let hash3 = hash_attrs(attrs3.iter());

    assert_ne!(hash1, hash3);
}

#[test]
fn test_write_attrs() {
    let mut output = String::new();
    let attrs = vec![
        KeyValue::new("key1", "value1"),
        KeyValue::new("key2", "value2"),
    ];

    write_attrs(&mut output, attrs.iter()).unwrap();
    assert_eq!(output, "key1=\"value1\",key2=\"value2\"");

    // Test with attributes containing characters that need escaping
    output.clear();
    let attrs_with_escapes = vec![
        KeyValue::new("key1", "value\nwith\nnewlines"),
        KeyValue::new("key2", "value\"with\"quotes"),
    ];

    write_attrs(&mut output, attrs_with_escapes.iter()).unwrap();
    assert_eq!(
        output,
        "key1=\"value\\nwith\\nnewlines\",key2=\"value\\\"with\\\"quotes\""
    );
}

#[test]
fn test_make_scope_name_attrs() {
    let scope_name = "test_scope";
    let attr = make_scope_name_attrs(scope_name);

    if cfg!(feature = "otel_scope_info") {
        assert!(attr.is_some());
        if let Some(kv) = attr {
            assert_eq!(kv.key.as_str(), "otel_scope_name");
            assert_eq!(kv.value.as_str(), "test_scope");
        }
    } else {
        assert!(attr.is_none());
    }
}

#[test]
fn test_to_timestamp() {
    use std::time::{Duration, UNIX_EPOCH};

    // Test with a known timestamp
    let time = UNIX_EPOCH + Duration::from_secs(1625097600);
    let timestamp = to_timestamp(time);
    let mut output = String::new();
    write!(output, "{}", timestamp).unwrap();
    #[cfg(feature = "fast")]
    assert_eq!(output, "1625097600.0");
    #[cfg(not(feature = "fast"))]
    assert_eq!(output, "1625097600");
}

#[cfg(feature = "otel_scope_info")]
#[test]
fn test_write_otel_scope_info() {
    let resource_metrics = make_test_metrics();
    let scopes: Vec<&ScopeMetrics> = resource_metrics.scope_metrics().collect();

    let mut output = String::new();
    write_otel_scope_info(&mut output, &scopes).unwrap();

    assert!(output.contains("# TYPE otel_scope info"));
    assert!(output.contains("otel_scope_info{"));
    assert!(output.contains("otel_scope_name=\"meter.1\""));
}

#[test]
fn test_get_type() {
    let resource_metrics = make_test_metrics();
    let scopes: Vec<&ScopeMetrics> = resource_metrics.scope_metrics().collect();

    for scope in scopes {
        for metric in scope.metrics() {
            let result = get_type(metric.data());
            assert!(result.is_ok());

            // Check that the type is one of the expected values
            let type_str = result.unwrap();
            assert!(
                type_str == "gauge" || type_str == "counter" || type_str == "histogram",
                "Unexpected metric type: {}",
                type_str
            );
        }
    }
}

#[test]
fn test_write_values() {
    let resource_metrics = make_test_metrics();
    let scopes: Vec<&ScopeMetrics> = resource_metrics.scope_metrics().collect();

    for scope in scopes {
        let scope_name = scope.scope().name();

        for metric in scope.metrics() {
            let mut output = String::new();
            let mut temp_buffer = String::from("staledata");
            let result = write_values(
                &mut output,
                &mut temp_buffer,
                scope_name,
                metric.data(),
                metric.name(),
            );

            assert!(result.is_ok());
            assert!(!output.is_empty());
            assert!(output.contains(metric.name()));
        }
    }
}

#[test]
fn test_write_gauge() {
    let resource_metrics = make_test_metrics();
    let scopes: Vec<&ScopeMetrics> = resource_metrics.scope_metrics().collect();
    let mut test_did_something = false;

    for scope in scopes {
        let scope_name = scope.scope().name();
        for metric in scope.metrics() {
            if let Ok("gauge") = get_type(metric.data()) {
                if let opentelemetry_sdk::metrics::data::AggregatedMetrics::F64(data) =
                    metric.data()
                {
                    if let opentelemetry_sdk::metrics::data::MetricData::Gauge(gauge) = data {
                        let mut output = String::new();
                        let mut temp_buffer = String::from("staledata");

                        let result = write_gauge(
                            &mut output,
                            metric.name(),
                            scope_name,
                            &mut temp_buffer,
                            gauge,
                        );

                        assert!(result.is_ok());
                        assert!(output.contains(metric.name()));
                        assert!(output.contains("{"));
                        assert!(output.contains("}"));
                        assert!(output.contains(" "));
                        test_did_something = true;
                    }
                }
            }
        }
    }
    assert!(test_did_something);
}

#[test]
fn test_write_counter() {
    let resource_metrics = make_test_metrics();
    let scopes: Vec<&ScopeMetrics> = resource_metrics.scope_metrics().collect();
    let mut test_did_something = false;

    for scope in scopes {
        let scope_name = scope.scope().name();

        for metric in scope.metrics() {
            if let Ok("counter") = get_type(metric.data()) {
                if let opentelemetry_sdk::metrics::data::AggregatedMetrics::U64(data) =
                    metric.data()
                {
                    if let opentelemetry_sdk::metrics::data::MetricData::Sum(sum) = data {
                        let mut output = String::new();
                        let mut temp_buffer = String::from("staledata");

                        let result = write_counter(
                            &mut output,
                            metric.name(),
                            scope_name,
                            &mut temp_buffer,
                            sum,
                        );

                        assert!(result.is_ok());
                        assert!(output.contains(metric.name()));
                        assert!(output.contains(" 125"));
                        test_did_something = true;
                    }
                }
            }
        }
    }
    assert!(test_did_something);
}

#[test]
fn test_write_histogram() {
    let resource_metrics = make_test_metrics();
    let scopes: Vec<&ScopeMetrics> = resource_metrics.scope_metrics().collect();
    let mut test_did_something = false;

    for scope in scopes {
        let scope_name = scope.scope().name();

        for metric in scope.metrics() {
            if let Ok("histogram") = get_type(metric.data()) {
                if let opentelemetry_sdk::metrics::data::AggregatedMetrics::F64(data) =
                    metric.data()
                {
                    if let opentelemetry_sdk::metrics::data::MetricData::Histogram(histogram) = data
                    {
                        let mut output = String::new();
                        let mut temp_buffer = String::from("staledata");

                        let result = write_histogram(
                            &mut output,
                            metric.name(),
                            scope_name,
                            &mut temp_buffer,
                            histogram,
                        );

                        assert!(result.is_ok());

                        assert!(output.contains(metric.name()));
                        assert!(output.contains("_count"));
                        assert!(output.contains("_sum"));
                        assert!(output.contains("_bucket"));
                        assert!(output.contains("le="));
                        test_did_something = true;
                    }
                }
            }
        }
    }
    assert!(test_did_something);
}

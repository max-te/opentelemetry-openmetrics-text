# opentelemetry-openmetrics

A Rust library for exporting OpenTelemetry metrics in the OpenMetrics text format. This serves as a protobuf-free alternative to the discontinued `opentelemetry-prometheus` crate.

> ⚠️ **Warning:** This implementation is not fully spec-compliant for [OpenTelemetry-to-OpenMetrics conversion](https://github.com/open-telemetry/opentelemetry-specification/blob/v1.45.0/specification/compatibility/prometheus_and_openmetrics.md). Some edge cases and complex metrics setups may not be handled correctly. This library is still in an experimental state.

## Features

- **Conversion**: Convert `opentelemetry-sdk` metric data to OpenMetrics-compliant text.
- **Ready-to-use Exporter**: ready-to-use exporter to output metrics in the OpenMetrics text format.

## Axum Integration Example

Below is an example of integrating the exporter with axum:

- **Initialize the exporter before registering meters:**

```rust
use std::time::Duration;
use opentelemetry_openmetrics::exporter::OpenMetricsExporter;
use opentelemetry_sdk::metrics::PeriodicReader;
use opentelemetry_sdk::metrics::SdkMeterProvider;

pub fn init_openmetrics_exporter() -> OpenMetricsExporter {
    let exporter = OpenMetricsExporter::new();
    let reader = PeriodicReader::builder(exporter.clone())
        .with_interval(Duration::from_secs(1))
        .build();
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .build();
    opentelemetry::global::set_meter_provider(meter_provider);
    exporter
}
```

- **Retain the exporter in your axum state.**

- **Register a GET `/metrics` handler:**

```rust
use axum::{extract::State, http::Response};
use opentelemetry_openmetrics::convert::MIME_TYPE;
use opentelemetry_openmetrics::exporter::OpenMetricsExporter;

async fn get_metrics(State(metrics): State<OpenMetricsExporter>) -> Response<String> {
    let metrics = metrics.text().await;
    Response::builder()
        .header(
            axum::http::header::CONTENT_TYPE,
            MIME_TYPE,
        )
        .body(metrics)
        .unwrap()
}
```


use std::sync::Arc;
use std::time::SystemTime;

use opentelemetry::KeyValue;
use opentelemetry::metrics::MeterProvider;
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::metrics::reader::MetricReader;
use opentelemetry_sdk::metrics::{ManualReader, SdkMeterProvider, Temporality};

#[derive(Debug, Clone)]
pub struct TestMetricsReader {
    inner: Arc<ManualReader>,
}

impl Default for TestMetricsReader {
    fn default() -> Self {
        Self {
            inner: Arc::new(ManualReader::builder().build()),
        }
    }
}

impl MetricReader for TestMetricsReader {
    fn register_pipeline(&self, pipeline: std::sync::Weak<opentelemetry_sdk::metrics::Pipeline>) {
        self.inner.register_pipeline(pipeline);
    }

    fn collect(
        &self,
        rm: &mut opentelemetry_sdk::metrics::data::ResourceMetrics,
    ) -> opentelemetry_sdk::error::OTelSdkResult {
        self.inner.collect(rm)
    }

    fn force_flush(&self) -> opentelemetry_sdk::error::OTelSdkResult {
        self.inner.force_flush()
    }

    fn shutdown(&self) -> opentelemetry_sdk::error::OTelSdkResult {
        self.inner.shutdown()
    }

    fn shutdown_with_timeout(
        &self,
        timeout: std::time::Duration,
    ) -> opentelemetry_sdk::error::OTelSdkResult {
        self.inner.shutdown_with_timeout(timeout)
    }

    fn temporality(&self, _kind: opentelemetry_sdk::metrics::InstrumentKind) -> Temporality {
        Temporality::Cumulative
    }
}

#[allow(dead_code)]
pub fn make_test_metrics() -> (ResourceMetrics, Vec<SystemTime>) {
    let reader = TestMetricsReader::default();
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(reader.clone())
        .build();
    let meter = meter_provider.meter("meter.1");

    let gauge = meter
        .f64_gauge("f64.gauge")
        .with_description("A \"gauge\"\nFor testing")
        .build();
    gauge.record(4.2, &[KeyValue::new("kk", "v1")]);
    gauge.record(4.22, &[KeyValue::new("kk", "v1")]);
    gauge.record(4.23, &[KeyValue::new("kk", "v2")]);

    let counter = meter.u64_counter("u64.counter").with_unit("s").build();
    counter.add(125, &[]);

    let hist = meter.f64_histogram("histo").build();
    hist.record(0.0, &[]);
    hist.record(1.3, &[]);
    hist.record(1.4, &[]);
    hist.record(13.0, &[]);

    let mut metrics = ResourceMetrics::default();
    reader.collect(&mut metrics).unwrap();
    let erasable_timestamps = collect_timestamps(&metrics);

    (metrics, erasable_timestamps)
}

fn collect_timestamps(metrics: &ResourceMetrics) -> Vec<SystemTime> {
    fn collect_timestamps_inner<T>(
        timestamps: &mut Vec<SystemTime>,
        metric_data: &opentelemetry_sdk::metrics::data::MetricData<T>,
    ) {
        match metric_data {
            opentelemetry_sdk::metrics::data::MetricData::Gauge(gauge) => {
                timestamps.push(gauge.time());
            }
            opentelemetry_sdk::metrics::data::MetricData::Sum(sum) => {
                timestamps.push(sum.time());
            }
            opentelemetry_sdk::metrics::data::MetricData::Histogram(histogram) => {
                timestamps.push(histogram.time());
                timestamps.push(histogram.start_time());
            }
            opentelemetry_sdk::metrics::data::MetricData::ExponentialHistogram(
                exponential_histogram,
            ) => {
                timestamps.push(exponential_histogram.time());
                timestamps.push(exponential_histogram.start_time());
            }
        }
    }

    let mut timestamps = Vec::new();
    for scope in metrics.scope_metrics() {
        for metric in scope.metrics() {
            match metric.data() {
                opentelemetry_sdk::metrics::data::AggregatedMetrics::F64(metric_data) => {
                    collect_timestamps_inner(&mut timestamps, metric_data);
                }
                opentelemetry_sdk::metrics::data::AggregatedMetrics::U64(metric_data) => {
                    collect_timestamps_inner(&mut timestamps, metric_data);
                }
                opentelemetry_sdk::metrics::data::AggregatedMetrics::I64(metric_data) => {
                    collect_timestamps_inner(&mut timestamps, metric_data)
                }
            }
        }
    }
    timestamps.sort_unstable();
    timestamps.dedup();
    timestamps
}

#[allow(dead_code)]
pub fn make_large_test_metrics() -> (ResourceMetrics, Vec<SystemTime>) {
    let reader = TestMetricsReader::default();
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(reader.clone())
        .build();
    let meter = meter_provider.meter("meter.1");

    let gauge = meter
        .f64_gauge("f64.gauge")
        .with_description("A \"gauge\"\nFor testing")
        .build();
    for i in 0..100 {
        gauge.record(4.22, &[KeyValue::new("foo.bar", format!("a{i}"))]);
    }

    let counter = meter.u64_counter("u64.counter").with_unit("s").build();
    for i in 0..1000 {
        counter.add(422 * i, &[KeyValue::new("high-low", format!("v\n{i}"))]);
    }

    let hist = meter.f64_histogram("histo").build();
    for i in 0..1000 {
        hist.record(
            4.22 / i as f64,
            &[
                KeyValue::new("x.y.z", format!("v{i}")),
                KeyValue::new("z.z.z", "fixed"),
                KeyValue::new("z.y.z", format!("0{}0", i + 1)),
            ],
        );
    }

    let mut metrics = ResourceMetrics::default();
    reader.collect(&mut metrics).unwrap();
    let erasable_timestamps = collect_timestamps(&metrics);
    
    (metrics, erasable_timestamps)
}

#[allow(dead_code)]
pub fn make_f64_gauge_metric(
    values: Vec<(f64, Vec<KeyValue>)>,
) -> opentelemetry_sdk::metrics::data::Gauge<f64> {
    let reader = TestMetricsReader::default();
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(reader.clone())
        .build();
    let scope_name = "test_meter";
    let meter = meter_provider.meter(scope_name);

    const MYGAUGE: &str = "mygauge";
    let gauge_builder = meter.f64_gauge(MYGAUGE);
    let gauge = gauge_builder.build();

    // Record all values with their attributes
    for (value, attrs) in values {
        gauge.record(value, attrs.as_slice());
    }

    // Collect metrics
    let mut metrics = ResourceMetrics::default();
    reader.collect(&mut metrics).unwrap();

    // Extract the gauge data
    let scope_metrics = metrics.scope_metrics().collect::<Vec<_>>();
    for scope in &scope_metrics {
        if scope.scope().name() == scope_name {
            for metric in scope.metrics() {
                if metric.name() == MYGAUGE {
                    if let opentelemetry_sdk::metrics::data::AggregatedMetrics::F64(
                        opentelemetry_sdk::metrics::data::MetricData::Gauge(gauge),
                    ) = metric.data()
                    {
                        return gauge.clone();
                    }
                }
            }
        }
    }

    unreachable!("should have found gauge data")
}

#[allow(dead_code)]
pub fn make_u64_counter_metric(
    values: Vec<(u64, Vec<KeyValue>)>,
) -> opentelemetry_sdk::metrics::data::Sum<u64> {
    let reader = TestMetricsReader::default();
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(reader.clone())
        .build();
    let scope_name = "test_meter";
    let meter = meter_provider.meter(scope_name);

    const MYCOUNTER: &str = "mycounter";
    let counter_builder = meter.u64_counter(MYCOUNTER);
    let counter = counter_builder.build();

    // Record all values with their attributes
    for (value, attrs) in values {
        counter.add(value, attrs.as_slice());
    }

    // Collect metrics
    let mut metrics = ResourceMetrics::default();
    reader.collect(&mut metrics).unwrap();

    // Extract the sum data
    let scope_metrics = metrics.scope_metrics().collect::<Vec<_>>();
    for scope in &scope_metrics {
        if scope.scope().name() == scope_name {
            for metric in scope.metrics() {
                if metric.name() == MYCOUNTER {
                    if let opentelemetry_sdk::metrics::data::AggregatedMetrics::U64(
                        opentelemetry_sdk::metrics::data::MetricData::Sum(sum),
                    ) = metric.data()
                    {
                        return sum.clone();
                    }
                }
            }
        }
    }

    unreachable!("should have found sum data")
}

#[allow(dead_code)]
pub fn make_f64_histogram_metric(
    values: Vec<(f64, Vec<KeyValue>)>,
) -> opentelemetry_sdk::metrics::data::Histogram<f64> {
    let reader = TestMetricsReader::default();
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(reader.clone())
        .build();
    let scope_name = "test_meter";
    let meter = meter_provider.meter(scope_name);

    const MYHISTOGRAM: &str = "myhistogram";
    let histogram_builder = meter.f64_histogram(MYHISTOGRAM);
    let histogram = histogram_builder.build();

    // Record all values with their attributes
    for (value, attrs) in values {
        histogram.record(value, attrs.as_slice());
    }

    // Collect metrics
    let mut metrics = ResourceMetrics::default();
    reader.collect(&mut metrics).unwrap();

    // Extract the histogram data
    let scope_metrics = metrics.scope_metrics().collect::<Vec<_>>();
    for scope in &scope_metrics {
        if scope.scope().name() == scope_name {
            for metric in scope.metrics() {
                if metric.name() == MYHISTOGRAM {
                    if let opentelemetry_sdk::metrics::data::AggregatedMetrics::F64(
                        opentelemetry_sdk::metrics::data::MetricData::Histogram(histogram),
                    ) = metric.data()
                    {
                        return histogram.clone();
                    }
                }
            }
        }
    }

    unreachable!("should have found histogram data")
}

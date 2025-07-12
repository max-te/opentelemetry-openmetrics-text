use opentelemetry::KeyValue;
use opentelemetry::metrics::MeterProvider;
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::metrics::reader::MetricReader;
use opentelemetry_sdk::metrics::{SdkMeterProvider};

use crate::reader::TestMetricsReader;

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

pub fn make_u64_gauge_metric(
    values: Vec<(u64, Vec<KeyValue>)>,
) -> opentelemetry_sdk::metrics::data::Gauge<u64> {
    let reader = TestMetricsReader::default();
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(reader.clone())
        .build();
    let scope_name = "test_meter";
    let meter = meter_provider.meter(scope_name);

    const MYGAUGE: &str = "mygauge";
    let gauge_builder = meter.u64_gauge(MYGAUGE);
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
                    if let opentelemetry_sdk::metrics::data::AggregatedMetrics::U64(
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


pub fn make_i64_gauge_metric(
    values: Vec<(i64, Vec<KeyValue>)>,
) -> opentelemetry_sdk::metrics::data::Gauge<i64> {
    let reader = TestMetricsReader::default();
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(reader.clone())
        .build();
    let scope_name = "test_meter";
    let meter = meter_provider.meter(scope_name);

    const MYGAUGE: &str = "mygauge";
    let gauge_builder = meter.i64_gauge(MYGAUGE);
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
                    if let opentelemetry_sdk::metrics::data::AggregatedMetrics::I64(
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


pub fn make_f64_counter_metric(
    values: Vec<(f64, Vec<KeyValue>)>,
) -> opentelemetry_sdk::metrics::data::Sum<f64> {
    let reader = TestMetricsReader::default();
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(reader.clone())
        .build();
    let scope_name = "test_meter";
    let meter = meter_provider.meter(scope_name);

    const MYCOUNTER: &str = "mycounter";
    let counter_builder = meter.f64_counter(MYCOUNTER);
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
                    if let opentelemetry_sdk::metrics::data::AggregatedMetrics::F64(
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


pub fn make_i64_counter_metric(
    values: Vec<(i64, Vec<KeyValue>)>,
) -> opentelemetry_sdk::metrics::data::Sum<i64> {
    let reader = TestMetricsReader::default();
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(reader.clone())
        .build();
    let scope_name = "test_meter";
    let meter = meter_provider.meter(scope_name);

    const MYCOUNTER: &str = "mycounter";
    let counter_builder = meter.i64_up_down_counter(MYCOUNTER);
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
                    if let opentelemetry_sdk::metrics::data::AggregatedMetrics::I64(
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

pub fn make_u64_histogram_metric(
    values: Vec<(u64, Vec<KeyValue>)>,
) -> opentelemetry_sdk::metrics::data::Histogram<u64> {
    let reader = TestMetricsReader::default();
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(reader.clone())
        .build();
    let scope_name = "test_meter";
    let meter = meter_provider.meter(scope_name);

    const MYHISTOGRAM: &str = "myhistogram";
    let histogram_builder = meter.u64_histogram(MYHISTOGRAM);
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
                    if let opentelemetry_sdk::metrics::data::AggregatedMetrics::U64(
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

use std::time::SystemTime;

use opentelemetry_sdk::metrics::data::ResourceMetrics;

pub fn get_all_timestamps(metrics: &ResourceMetrics) -> Vec<SystemTime> {
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
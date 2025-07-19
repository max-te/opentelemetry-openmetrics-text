use std::time::SystemTime;

use opentelemetry_sdk::metrics::data::{AggregatedMetrics, MetricData, ResourceMetrics};

pub fn get_all_timestamps(metrics: &ResourceMetrics) -> Vec<SystemTime> {
    fn collect_timestamps_inner<T>(timestamps: &mut Vec<SystemTime>, metric_data: &MetricData<T>) {
        match metric_data {
            MetricData::Gauge(gauge) => {
                timestamps.push(gauge.time());
            }
            MetricData::Sum(sum) => {
                timestamps.push(sum.time());
            }
            MetricData::Histogram(histogram) => {
                timestamps.push(histogram.time());
                timestamps.push(histogram.start_time());
            }
            MetricData::ExponentialHistogram(exponential_histogram) => {
                timestamps.push(exponential_histogram.time());
                timestamps.push(exponential_histogram.start_time());
            }
        }
    }

    let mut timestamps = Vec::new();
    for scope in metrics.scope_metrics() {
        for metric in scope.metrics() {
            match metric.data() {
                AggregatedMetrics::F64(metric_data) => {
                    collect_timestamps_inner(&mut timestamps, metric_data);
                }
                AggregatedMetrics::U64(metric_data) => {
                    collect_timestamps_inner(&mut timestamps, metric_data);
                }
                AggregatedMetrics::I64(metric_data) => {
                    collect_timestamps_inner(&mut timestamps, metric_data)
                }
            }
        }
    }
    timestamps.sort_unstable();
    timestamps.dedup();
    timestamps
}

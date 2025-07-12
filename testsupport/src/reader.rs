use std::sync::Arc;

use opentelemetry_sdk::metrics::reader::MetricReader;
use opentelemetry_sdk::metrics::{ManualReader, Temporality};

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

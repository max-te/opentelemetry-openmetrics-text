use std::fmt::Write;
use std::sync::Arc;
use std::time::Duration;

use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use opentelemetry_sdk::metrics::Temporality;
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::metrics::exporter::PushMetricExporter;
use tokio::sync::RwLock;

use crate::convert::ToOpenMetrics;

#[derive(Debug, Clone)]
pub struct OpenMetricsExporter {
    buffer: Arc<RwLock<String>>,
}

impl OpenMetricsExporter {
    pub fn new() -> Self {
        OpenMetricsExporter {
            buffer: Arc::new(RwLock::new(String::new())),
        }
    }

    pub async fn text(&self) -> String {
        self.buffer.read().await.clone()
    }
}

impl PushMetricExporter for OpenMetricsExporter {
    async fn export(&self, metrics: &ResourceMetrics) -> OTelSdkResult {
        #[cfg(feature = "tracing")]
        tracing::debug!("Exporting metrics");
        let mut buffer = self.buffer.write().await;
        buffer.clear();
        write!(buffer, "{}", ToOpenMetrics(metrics)).map_err(|err| {
            OTelSdkError::InternalFailure(format!("Failed to write to buffer: {}", err))
        })
    }

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        Ok(())
    }

    fn temporality(&self) -> Temporality {
        Temporality::Cumulative
    }
}

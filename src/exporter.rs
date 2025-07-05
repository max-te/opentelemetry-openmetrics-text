use std::sync::Arc;
use std::time::Duration;

use opentelemetry_sdk::error::{OTelSdkError, OTelSdkResult};
use opentelemetry_sdk::metrics::Temporality;
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::metrics::exporter::PushMetricExporter;
use tokio::sync::{Mutex, RwLock};

use crate::convert::WriteOpenMetrics;

#[derive(Debug, Clone)]
pub struct OpenMetricsExporter {
    buffer: Arc<RwLock<Option<String>>>,
    backbuffer: Arc<Mutex<Option<String>>>,
    // TODO: replace this with something simpler like arc-swap
}

impl Default for OpenMetricsExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenMetricsExporter {
    pub fn new() -> Self {
        OpenMetricsExporter {
            buffer: Arc::new(RwLock::new(Some(String::new()))),
            backbuffer: Arc::new(Mutex::new(Some(String::new()))),
        }
    }

    pub async fn text(&self) -> String {
        self.buffer.read().await.as_ref().unwrap().clone()
    }
}

impl PushMetricExporter for OpenMetricsExporter {
    async fn export(&self, metrics: &ResourceMetrics) -> OTelSdkResult {
        #[cfg(feature = "tracing")]
        tracing::debug!("Exporting metrics");
        let mut backbuffer = self.backbuffer.lock().await;
        let mut nextbuffer = backbuffer.take().unwrap_or_default();
        nextbuffer.clear();
        metrics
            .write_as_openmetrics(&mut nextbuffer)
            .map_err(|err| {
                OTelSdkError::InternalFailure(format!("Failed to write to buffer: {err}"))
            })?;

        let mut buffer = self.buffer.write().await;
        let oldbuffer = buffer.replace(nextbuffer);
        *backbuffer = oldbuffer;

        Ok(())
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

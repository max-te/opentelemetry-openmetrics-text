#[cfg(feature = "exporter")]
mod exporter;
mod parsing;
#[cfg(feature = "otel_scope_info")]
// Changes attributes
mod snapshot;

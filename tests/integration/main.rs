mod parsing;
#[cfg(feature = "otel_scope_info")]
// Changes attributes
#[cfg(feature = "fast")]
// Difference: .0 is rendered for floats with ryu but not with std::fmt::Display
mod snapshot;

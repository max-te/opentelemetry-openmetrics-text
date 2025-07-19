use std::borrow::Cow;
use std::fmt::{Display, Write};
use std::hash::{DefaultHasher, Hasher};
use std::time::SystemTime;

use crate::format::{FastDisplay, conwrite};
use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::Temporality;
use opentelemetry_sdk::metrics::data::{
    AggregatedMetrics, Gauge, Histogram, MetricData, ResourceMetrics, Sum,
};
use opentelemetry_sdk::metrics::data::{Metric, ScopeMetrics};
use unit::get_unit_suffixes;

#[cfg(test)]
mod tests;
mod unit;

/// The mime type of the text produced by this metrics formatter.
pub const MIME_TYPE: &str = "application/openmetrics-text; version=1.0.0; charset=utf-8";

/// Trait to write the metrics data in OpenMetrics text format.
pub trait WriteOpenMetrics {
    /// Writes the metrics into `f` in OpenMetrics text format.
    fn write_as_openmetrics(&self, f: &mut impl Write) -> std::fmt::Result;
    /// Creates and returns a [String] of the metrics data in OpenMetrics text format.
    fn to_openmetrics_string(&self) -> Result<String, std::fmt::Error> {
        let mut out = String::new();
        self.write_as_openmetrics(&mut out)?;
        Ok(out)
    }
}

/// Serialization context for common variables needed during conversion.
struct Context<'f, W: Write> {
    /// the output [Write] reference
    f: &'f mut W,
    /// a temporary buffer to store the serialized metric attributes
    attr_buffer: String,
    /// the sanitized name of the current metric
    name: String,
    /// the converted unit string of the current metric
    unit: Option<Cow<'static, str>>,
    /// the OpenMetrics metric type of the current metric
    typ: &'static str,
    /// the name of the current scope
    scope_name: &'f str,
}

impl<'f, W: Write> Context<'f, W> {
    fn with_output(f: &'f mut W) -> Self {
        Context {
            f,
            attr_buffer: String::with_capacity(256),
            name: String::with_capacity(64),
            unit: None,
            typ: "",
            scope_name: "",
        }
    }
}

impl WriteOpenMetrics for ResourceMetrics {
    fn write_as_openmetrics(&self, f: &mut impl Write) -> std::fmt::Result {
        // TODO: let resource_attrs = self.0.resource().into_iter().collect::<Vec<_>>();
        // write these into a `target_info` metric
        // (https://github.com/open-telemetry/opentelemetry-specification/blob/v1.45.0/specification/compatibility/prometheus_and_openmetrics.md#resource-attributes-1)

        let mut ctx = Context::with_output(f);
        let mut scopes: Vec<&ScopeMetrics> = self.scope_metrics().collect();
        scopes.sort_unstable_by_key(|s| s.scope().name());

        #[cfg(feature = "otel_scope_info")]
        write_otel_scope_info(ctx.f, &scopes)?;

        for scope in scopes {
            if cfg!(feature = "otel_scope_info") {
                ctx.scope_name = scope.scope().name();
            }
            let mut metrics: Vec<_> = scope.metrics().collect();
            metrics.sort_unstable_by_key(|met| met.name());

            for metric in metrics {
                if extract_type_unit_and_name(&mut ctx, metric) {
                    write_header(&mut ctx, metric.description())?;
                    write_values(&mut ctx, metric.data())?;
                } else {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("Unsupported metric type {metric:?}");
                }
            }
        }
        f.write_str("# EOF\n")?;
        Ok(())
    }
}

fn extract_type_unit_and_name(ctx: &mut Context<'_, impl Write>, metric: &Metric) -> bool {
    let Ok(typ) = get_type(metric.data()) else {
        return false;
    };
    ctx.typ = typ;
    ctx.unit = get_unit_suffixes(metric.unit());

    ctx.name.clear();
    let Ok(_) = write_sanitized_name(&mut ctx.name, metric.name()) else {
        return false;
    };
    if let Some(ref unit) = ctx.unit {
        ctx.name.push('_');
        ctx.name.push_str(unit);
    }

    true
}

/// Gets the OpenMetrics metric type for this [AggregatedMetrics].
/// Returns `Err(())` for unsupported metric types.
fn get_type(metric: &AggregatedMetrics) -> Result<&'static str, ()> {
    fn get_metric_data_type<T>(metric_data: &MetricData<T>) -> Result<&'static str, ()> {
        match metric_data {
            MetricData::Gauge(_) => Ok("gauge"),
            MetricData::Sum(sum) => {
                if sum.is_monotonic() {
                    Ok("counter")
                } else {
                    Ok("gauge")
                }
            }
            MetricData::Histogram(hist) => {
                if hist.temporality() == Temporality::Cumulative {
                    Ok("histogram")
                } else {
                    Err(())
                }
            }
            _ => Err(()),
        }
    }
    match metric {
        AggregatedMetrics::F64(metric_data) => get_metric_data_type(metric_data),
        AggregatedMetrics::U64(metric_data) => get_metric_data_type(metric_data),
        AggregatedMetrics::I64(metric_data) => get_metric_data_type(metric_data),
    }
}

/// Write the current metric's metadata. Make sure to call [extract_type_unit_and_name] first.
#[inline]
fn write_header(ctx: &mut Context<'_, impl Write>, description: &str) -> std::fmt::Result {
    let Context {
        f, name, unit, typ, ..
    } = ctx;
    for x in &["# TYPE ", name, " ", typ, "\n"] {
        f.write_str(x)?;
    }

    if let Some(unit) = unit {
        for x in &["# UNIT ", name, " ", unit, "\n"] {
            f.write_str(x)?;
        }
    }
    if !description.is_empty() {
        f.write_str("# HELP ")?;
        f.write_str(name)?;
        f.write_str(" ")?;
        write_escaped(f, description)?;
        f.write_char('\n')?;
    }
    Ok(())
}

/// Write a otel_scope metric of type info for all scopes in `metrics`
/// according to the [spec](https://github.com/open-telemetry/opentelemetry-specification/blob/v1.45.0/specification/compatibility/prometheus_and_openmetrics.md#instrumentation-scope-1).
#[cfg(feature = "otel_scope_info")]
fn write_otel_scope_info(f: &mut impl Write, metrics: &'_ Vec<&ScopeMetrics>) -> std::fmt::Result {
    f.write_str("# TYPE otel_scope info\n")?;

    for scope in metrics {
        let otel_attrs = &[
            KeyValue::new("otel_scope_name", scope.scope().name().to_owned()),
            KeyValue::new(
                "otel_scope_version",
                scope.scope().version().unwrap_or_default().to_owned(),
            ),
        ];
        f.write_str("otel_scope_info{")?;
        write_attrs(f, otel_attrs.iter().chain(scope.scope().attributes()))?;
        f.write_str("} 1\n")?;
    }
    Ok(())
}

/// Write all data points for this metric
fn write_values(ctx: &mut Context<'_, impl Write>, metric: &AggregatedMetrics) -> std::fmt::Result {
    match metric {
        AggregatedMetrics::F64(metric_data) => {
            match metric_data {
                MetricData::Gauge(gauge) => write_gauge(ctx, gauge),
                MetricData::Sum(sum) => write_counter(ctx, sum),
                MetricData::Histogram(histogram) => write_histogram(ctx, histogram),
                _ => unimplemented!("only gauge/sum/histogram metrics should be constructible"),
                // See https://github.com/open-telemetry/opentelemetry-specification/blob/v1.45.0/specification/compatibility/prometheus_and_openmetrics.md#exponential-histograms
                // for exponential histograms
            }
        }
        AggregatedMetrics::U64(metric_data) => match metric_data {
            MetricData::Gauge(gauge) => write_gauge(ctx, gauge),
            MetricData::Sum(sum) => write_counter(ctx, sum),
            MetricData::Histogram(histogram) => write_histogram(ctx, histogram),
            _ => unimplemented!("only gauge/sum/histogram metrics should be constructible"),
        },
        AggregatedMetrics::I64(metric_data) => match metric_data {
            MetricData::Gauge(gauge) => write_gauge(ctx, gauge),
            MetricData::Sum(sum) => write_counter(ctx, sum),
            MetricData::Histogram(histogram) => write_histogram(ctx, histogram),
            _ => unimplemented!("only gauge/sum/histogram metrics should be constructible"),
        },
    }
}

fn write_histogram<T: FastDisplay + Copy>(
    ctx: &mut Context<'_, impl Write>,
    histogram: &Histogram<T>,
) -> std::fmt::Result {
    let scope_name_attrs = make_scope_name_attrs(ctx.scope_name);
    let ts = to_timestamp(histogram.time());
    let created = to_timestamp(histogram.start_time());
    ctx.attr_buffer.clear();
    let attrs = &mut ctx.attr_buffer;
    write_attrs(attrs, scope_name_attrs.iter())?;
    conwrite!(
        ctx.f,
        ctx.name,
        "_created{",
        attrs,
        "} ",
        created,
        ' ',
        ts,
        '\n'
    )?;
    assert_eq!(
        histogram.temporality(),
        Temporality::Cumulative,
        "Only cumulative Histograms are supported"
    );

    let mut points: Vec<_> = histogram.data_points().collect();
    points.sort_by_cached_key(|p| hash_attrs(p.attributes()));

    for point in points {
        attrs.clear();
        write_attrs(attrs, point.attributes().chain(scope_name_attrs.iter()))?;

        writeln!(
            ctx.f,
            "{name}_count{{{attrs}}} {value} {ts}",
            name = ctx.name,
            value = point.count().fast_display(),
        )?;
        writeln!(
            ctx.f,
            "{name}_sum{{{attrs}}} {value} {ts}",
            name = ctx.name,
            value = point.sum().fast_display(),
        )?;

        #[cfg(feature = "histogram-min-max")]
        {
            // Non-compliant but useful
            // TODO: Expose as a separate gauge?
            if let Some(min) = point.min() {
                writeln!(
                    f,
                    "{name}_min{{{attrs}}} {value} {ts}",
                    value = min.fast_display()
                )?;
            }
            if let Some(max) = point.max() {
                writeln!(
                    f,
                    "{name}_max{{{attrs}}} {value} {ts}",
                    value = max.fast_display()
                )?;
            }
        }

        if !attrs.is_empty() {
            attrs.push(',');
        }
        let mut cumulative_count = 0;
        for (bound, count) in std::iter::zip(point.bounds(), point.bucket_counts()) {
            cumulative_count += count;
            conwrite!(
                // Not using write! here is a ~19% speedup
                ctx.f,
                ctx.name,
                "_bucket{",
                attrs,
                "le=\"",
                bound.fast_display(),
                "\"} ",
                cumulative_count.fast_display(),
                ' ',
                ts,
                '\n'
            )?;
            // writeln!(
            //     f,
            //     "{name}_bucket{{{attrs}le=\"{bound}\"}} {count} {ts}",
            //     bound = bound.fast_display(),
            //     count = cumulative_count.fast_display(),
            // )?;
        }
        writeln!(
            ctx.f,
            "{name}_bucket{{{attrs}le=\"+Inf\"}} {value} {ts}",
            name = ctx.name,
            value = point.count().fast_display()
        )?;
    }
    Ok(())
}

fn write_counter<T: FastDisplay + Copy>(
    ctx: &mut Context<'_, impl Write>,
    sum: &Sum<T>,
) -> std::fmt::Result {
    let attrs = &mut ctx.attr_buffer;
    let scope_name_attrs = make_scope_name_attrs(ctx.scope_name);
    assert_eq!(
        sum.temporality(),
        opentelemetry_sdk::metrics::Temporality::Cumulative,
        "Only cumulative sums are supported"
    );

    let mut points: Vec<_> = sum.data_points().collect();
    points.sort_by_cached_key(|p| hash_attrs(p.attributes()));

    let ts = to_timestamp(sum.time());

    if sum.is_monotonic() {
        for point in points {
            attrs.clear();
            write_attrs(attrs, point.attributes().chain(scope_name_attrs.iter()))?;
            writeln!(
                ctx.f,
                "{name}_total{{{attrs}}} {value} {ts}",
                name = ctx.name,
                value = point.value().fast_display(),
            )?;
        }
        Ok(())
    } else {
        for point in points {
            attrs.clear();
            write_attrs(attrs, point.attributes().chain(scope_name_attrs.iter()))?;
            writeln!(
                ctx.f,
                "{name}{{{attrs}}} {value} {ts}",
                name = ctx.name,
                value = point.value().fast_display()
            )?;
        }
        Ok(())
    }
}

fn write_gauge<T: FastDisplay + Copy>(
    ctx: &mut Context<'_, impl Write>,
    gauge: &Gauge<T>,
) -> std::fmt::Result {
    let attrs = &mut ctx.attr_buffer;
    let scope_name_attrs = make_scope_name_attrs(ctx.scope_name);
    let ts = to_timestamp(gauge.time());
    let mut points: Vec<_> = gauge.data_points().collect();
    points.sort_by_cached_key(|p| hash_attrs(p.attributes()));
    for point in points {
        attrs.clear();
        write_attrs(attrs, point.attributes().chain(scope_name_attrs.iter()))?;
        writeln!(
            ctx.f,
            "{name}{{{attrs}}} {value} {ts}",
            name = ctx.name,
            value = point.value().fast_display()
        )?;
    }
    Ok(())
}

/// Makes an `otel_scope_name` attribute with the specified `scope_name` if the `otel_scope_info` feature is active.
#[inline(always)]
fn make_scope_name_attrs(scope_name: &str) -> Option<KeyValue> {
    if cfg!(feature = "otel_scope_info") {
        Some(KeyValue::new("otel_scope_name", scope_name.to_owned()))
    } else {
        None
    }
}

/// Write the attribute string for attrs. Does not write curly braces.
fn write_attrs<'a, I: Iterator<Item = &'a KeyValue>>(
    f: &mut impl std::fmt::Write,
    attrs: I,
) -> std::fmt::Result {
    let mut first = true;

    let mut attrs: Vec<_> = attrs.collect();
    attrs.sort_unstable_by_key(|attr| &attr.key);

    for attr in attrs {
        if !first {
            f.write_char(',')?;
        }
        write_sanitized_name(f, attr.key.as_str())?;
        f.write_str("=\"")?;
        write_escaped(f, &attr.value.as_str())?;
        f.write_char('"')?;
        first = false;
    }
    Ok(())
}

/// Calculates a hash of the [KeyValue] pairs which is invariant under reordering of the [KeyValue]s within the [Iterator].
fn hash_attrs<'a, I: Iterator<Item = &'a KeyValue>>(attrs: I) -> u64 {
    let mut hash = 0;
    for kv in attrs {
        let mut hasher = DefaultHasher::default();
        hasher.write(kv.key.as_str().as_bytes());
        hasher.write(kv.value.as_str().as_bytes());
        hash ^= hasher.finish(); // XOR to be order-invariant
    }
    hash
}

/// Writes to `f` the contents of `value` as an escaped string. Does not put quotes around the value.
/// The chars to escape are `\`, `"` and `\n`.
fn write_escaped(f: &mut impl Write, value: &str) -> std::fmt::Result {
    #[inline]
    fn next_escape_char(bytes: &[u8]) -> Option<usize> {
        #[cfg(feature = "fast")]
        return memchr::memchr3(b'\\', b'"', b'\n', bytes);
        #[cfg(not(feature = "fast"))]
        bytes
            .iter()
            .position(|&byte| byte == b'\\' || byte == b'"' || byte == b'\n')
    }

    let mut bytes = value.as_bytes();

    while let Some(next_escape) = next_escape_char(bytes) {
        let (head, tail) = bytes.split_at(next_escape);
        f.write_str(str::from_utf8(head).expect("escapable chars should be on a char boundary"))?;
        match tail[0] {
            b'\\' => f.write_str("\\\\"),
            b'"' => f.write_str("\\\""),
            b'\n' => f.write_str("\\n"),
            _ => unreachable!("next_escape_char should find one of the 3 escapable chars"),
        }?;
        bytes = &tail[1..];
    }
    f.write_str(str::from_utf8(bytes).expect("escaped string should be valid utf-8"))
}

/// Write `name` as an OpenMetrics metrics name, replacing any illegal characters with underscore according to the
/// [spec](https://github.com/open-telemetry/opentelemetry-specification/blob/v1.45.0/specification/compatibility/prometheus_and_openmetrics.md#metric-metadata-1).
fn write_sanitized_name(f: &mut impl Write, name: &str) -> std::fmt::Result {
    // Multiple consecutive `_` characters MUST be replaced with a single `_` character
    let mut last_is_underscore = false;
    // The name must not start with a digit
    if name.starts_with(|c: char| c.is_ascii_digit()) {
        f.write_char('_')?;
        last_is_underscore = true;
    }
    for c in name.chars() {
        // Allowed characters are `a-z A-Z 0-9 : _`
        // Invalid characters in the metric name MUST be replaced with the `_` character.
        if c.is_ascii_alphanumeric() || c == ':' {
            f.write_char(c)?;
            last_is_underscore = false;
        } else {
            if !last_is_underscore {
                f.write_char('_')?;
            }
            last_is_underscore = true;
        }
    }
    Ok(())
}

/// Get a [Display] implementation which shows [SystemTime] as a unix timestamp in float seconds.
fn to_timestamp(time: SystemTime) -> impl Display {
    let ts = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs_f64();
    ts.fast_display()
}

use std::fmt::{Display, Write};
use std::hash::{DefaultHasher, Hasher};

use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::metrics::data::ScopeMetrics;
use unit::get_unit_suffixes;

#[cfg(test)]
mod tests;

pub const MIME_TYPE: &str = "application/openmetrics-text; version=1.0.0; charset=utf-8";
pub trait WriteOpenMetrics {
    fn write_as_openmetrics(&self, f: &mut impl Write) -> std::fmt::Result;
    fn to_openmetrics_string(&self) -> Result<String, std::fmt::Error> {
        let mut out = String::new();
        self.write_as_openmetrics(&mut out)?;
        Ok(out)
    }
}

impl WriteOpenMetrics for ResourceMetrics {
    fn write_as_openmetrics(&self, f: &mut impl Write) -> std::fmt::Result {
        // TODO: let resource_attrs = self.0.resource().into_iter().collect::<Vec<_>>();

        let mut temp_buffer = String::with_capacity(256);

        let mut scopes: Vec<&ScopeMetrics> = self.scope_metrics().collect();
        scopes.sort_unstable_by_key(|s| s.scope().name());

        #[cfg(feature = "otel_scope_info")]
        write_otel_scope_info(f, &scopes)?;

        for scope in scopes {
            let scope_name = scope.scope().name();

            let mut metrics: Vec<_> = scope.metrics().collect();
            metrics.sort_unstable_by_key(|met| met.name());

            for metric in metrics {
                let Ok(typ) = get_type(metric.data()) else {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("Unsupported metric type {metric:?}");
                    continue;
                };
                let mut name = String::with_capacity(metric.name().len());
                write_sanitized_name(&mut name, metric.name())?;
                let unit = get_unit_suffixes(metric.unit());
                if let Some(ref unit) = unit {
                    name.push('_');
                    name.push_str(unit);
                }

                writeln!(f, "# TYPE {name} {typ}")?;

                if let Some(unit) = unit {
                    writeln!(f, "# UNIT {name} {unit}")?;
                }
                if !metric.description().is_empty() {
                    write!(f, "# HELP {name} ")?;
                    write_escaped(f, metric.description())?;
                    f.write_char('\n')?;
                }
                write_values(f, &mut temp_buffer, scope_name, metric.data(), &name)?;
            }
        }
        writeln!(f, "# EOF")?;
        Ok(())
    }
}

#[cfg(feature = "otel_scope_info")]
fn write_otel_scope_info(f: &mut impl Write, metrics: &'_ Vec<&ScopeMetrics>) -> std::fmt::Result {
    // Reference https://github.com/open-telemetry/opentelemetry-specification/blob/v1.45.0/specification/compatibility/prometheus_and_openmetrics.md#instrumentation-scope-1
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

fn get_type(
    metric: &opentelemetry_sdk::metrics::data::AggregatedMetrics,
) -> Result<&'static str, ()> {
    fn get_metric_data_type<T>(
        metric_data: &opentelemetry_sdk::metrics::data::MetricData<T>,
    ) -> Result<&'static str, ()> {
        match metric_data {
            opentelemetry_sdk::metrics::data::MetricData::Gauge(_) => Ok("gauge"),
            opentelemetry_sdk::metrics::data::MetricData::Sum(sum) => {
                if sum.is_monotonic() {
                    Ok("counter")
                } else {
                    Ok("gauge")
                }
            }
            opentelemetry_sdk::metrics::data::MetricData::Histogram(_) => Ok("histogram"),
            opentelemetry_sdk::metrics::data::MetricData::ExponentialHistogram(_) => Err(()),
        }
    }
    match metric {
        opentelemetry_sdk::metrics::data::AggregatedMetrics::F64(metric_data) => {
            get_metric_data_type(metric_data)
        }
        opentelemetry_sdk::metrics::data::AggregatedMetrics::U64(metric_data) => {
            get_metric_data_type(metric_data)
        }
        opentelemetry_sdk::metrics::data::AggregatedMetrics::I64(metric_data) => {
            get_metric_data_type(metric_data)
        }
    }
}

fn write_values(
    f: &mut impl Write,
    temp_buffer: &mut String,
    scope_name: &str,
    metric: &opentelemetry_sdk::metrics::data::AggregatedMetrics,
    name: &str,
) -> std::fmt::Result {
    match metric {
        opentelemetry_sdk::metrics::data::AggregatedMetrics::F64(metric_data) => {
            match metric_data {
                opentelemetry_sdk::metrics::data::MetricData::Gauge(gauge) => {
                    write_gauge(f, name, scope_name, temp_buffer, gauge)
                }
                opentelemetry_sdk::metrics::data::MetricData::Sum(sum) => {
                    write_counter(f, name, scope_name, temp_buffer, sum)
                }
                opentelemetry_sdk::metrics::data::MetricData::Histogram(histogram) => {
                    write_histogram(f, name, scope_name, temp_buffer, histogram)
                }
                // See https://github.com/open-telemetry/opentelemetry-specification/blob/v1.45.0/specification/compatibility/prometheus_and_openmetrics.md#exponential-histograms
                // for exponential histograms
                _ => unimplemented!(),
            }
        }
        opentelemetry_sdk::metrics::data::AggregatedMetrics::U64(metric_data) => {
            match metric_data {
                opentelemetry_sdk::metrics::data::MetricData::Gauge(gauge) => {
                    write_gauge(f, name, scope_name, temp_buffer, gauge)
                }
                opentelemetry_sdk::metrics::data::MetricData::Sum(sum) => {
                    write_counter(f, name, scope_name, temp_buffer, sum)
                }
                opentelemetry_sdk::metrics::data::MetricData::Histogram(histogram) => {
                    write_histogram(f, name, scope_name, temp_buffer, histogram)
                }
                _ => unimplemented!(),
            }
        }
        opentelemetry_sdk::metrics::data::AggregatedMetrics::I64(metric_data) => {
            match metric_data {
                opentelemetry_sdk::metrics::data::MetricData::Gauge(gauge) => {
                    write_gauge(f, name, scope_name, temp_buffer, gauge)
                }
                opentelemetry_sdk::metrics::data::MetricData::Sum(sum) => {
                    write_counter(f, name, scope_name, temp_buffer, sum)
                }
                opentelemetry_sdk::metrics::data::MetricData::Histogram(histogram) => {
                    write_histogram(f, name, scope_name, temp_buffer, histogram)
                }
                _ => unimplemented!(),
            }
        }
    }
}

#[inline(always)]
fn make_scope_name_attrs(scope_name: &str) -> Option<KeyValue> {
    if cfg!(feature = "otel_scope_info") {
        Some(KeyValue::new("otel_scope_name", scope_name.to_owned()))
    } else {
        None
    }
}

macro_rules! conwrite {
    ($dst:expr, $($arg:expr),*) => {
        (|| {
        $(
            $dst.write_fmt(format_args!("{}", $arg))?;
        )*
        let res: std::fmt::Result = Ok(());
        res
        })()
    };
}

fn write_histogram<T: FastDisplay + Copy>(
    f: &mut impl Write,
    name: &str,
    scope_name: &str,
    temp_buffer: &mut String,
    histogram: &opentelemetry_sdk::metrics::data::Histogram<T>,
) -> std::fmt::Result {
    let scope_name_attrs = make_scope_name_attrs(scope_name);
    let ts = to_timestamp(histogram.time());
    let created = to_timestamp(histogram.start_time());
    temp_buffer.clear();
    let attrs = temp_buffer;
    write_attrs(attrs, scope_name_attrs.iter())?;
    conwrite!(f, name, "_created{", attrs, "} ", created, ' ', ts, '\n')?;
    assert_eq!(
        histogram.temporality(),
        opentelemetry_sdk::metrics::Temporality::Cumulative,
        "Only cumulative Histograms are supported"
    );

    let mut points: Vec<_> = histogram.data_points().collect();
    points.sort_by_cached_key(|p| hash_attrs(p.attributes()));

    for point in points {
        attrs.clear();
        write_attrs(attrs, point.attributes().chain(scope_name_attrs.iter()))?;

        writeln!(
            f,
            "{name}_count{{{attrs}}} {value} {ts}",
            value = point.count().fast_display(),
        )?;
        writeln!(
            f,
            "{name}_sum{{{attrs}}} {value} {ts}",
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
                f,
                name,
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
            f,
            "{name}_bucket{{{attrs}le=\"+Inf\"}} {value} {ts}",
            value = point.count().fast_display()
        )?;
    }
    Ok(())
}

fn write_counter<T: FastDisplay + Copy>(
    f: &mut impl Write,
    name: &str,
    scope_name: &str,
    temp_buffer: &mut String,
    sum: &opentelemetry_sdk::metrics::data::Sum<T>,
) -> std::fmt::Result {
    let attrs = temp_buffer;
    let scope_name_attrs = make_scope_name_attrs(scope_name);
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
                f,
                "{name}_total{{{attrs}}} {value} {ts}",
                value = point.value().fast_display(),
            )?;
        }
        Ok(())
    } else {
        for point in points {
            attrs.clear();
            write_attrs(attrs, point.attributes().chain(scope_name_attrs.iter()))?;
            writeln!(
                f,
                "{name}{{{attrs}}} {value} {ts}",
                value = point.value().fast_display()
            )?;
        }
        Ok(())
    }
}

fn write_gauge<T: FastDisplay + Copy>(
    f: &mut impl Write,
    name: &str,
    scope_name: &str,
    temp_buffer: &mut String,
    gauge: &opentelemetry_sdk::metrics::data::Gauge<T>,
) -> std::fmt::Result {
    let attrs = temp_buffer;
    let scope_name_attrs = make_scope_name_attrs(scope_name);
    let ts = to_timestamp(gauge.time());
    let mut points: Vec<_> = gauge.data_points().collect();
    points.sort_by_cached_key(|p| hash_attrs(p.attributes()));
    for point in points {
        attrs.clear();
        write_attrs(attrs, point.attributes().chain(scope_name_attrs.iter()))?;
        writeln!(
            f,
            "{name}{{{attrs}}} {value} {ts}",
            value = point.value().fast_display()
        )?;
    }
    Ok(())
}

fn to_timestamp(time: std::time::SystemTime) -> impl Display {
    let ts = time
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs_f64();
    ts.fast_display()
}

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
        f.write_str(str::from_utf8(head).unwrap())?;
        match tail[0] {
            b'\\' => f.write_str("\\\\"),
            b'"' => f.write_str("\\\""),
            b'\n' => f.write_str("\\n"),
            _ => unreachable!(),
        }?;
        bytes = &tail[1..];
    }
    f.write_str(str::from_utf8(bytes).unwrap())
}

fn write_sanitized_name(f: &mut impl Write, name: &str) -> std::fmt::Result {
    // Reference https://github.com/open-telemetry/opentelemetry-specification/blob/v1.45.0/specification/compatibility/prometheus_and_openmetrics.md#metric-metadata-1
    let mut last_is_underscore = false;
    if name.starts_with(|c: char| c.is_ascii_digit()) {
        f.write_char('_')?;
        last_is_underscore = true;
    }
    for c in name.chars() {
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

mod unit {
    /*!
     * OTEL style short unit to Prometheus style long unit conversion
     *
     * Extracted from the opentelemetry-prometheus crate (https://github.com/open-telemetry/opentelemetry-rust/blob/eac368a7e4addbee3b68c27a0eafae59928ad4c7/opentelemetry-prometheus/src/utils.rs)
     * Licensed under the Apache-2.0 License, Copyright 2025 The opentelemetry-rust Authors
     */

    use std::borrow::Cow;

    const NON_APPLICABLE_ON_PER_UNIT: [&str; 8] = ["1", "d", "h", "min", "s", "ms", "us", "ns"];

    pub(crate) fn get_unit_suffixes(unit: &str) -> Option<Cow<'static, str>> {
        // no unit return early
        if unit.is_empty() {
            return None;
        }

        // direct match with known units
        if let Some(matched) = get_prom_units(unit) {
            return Some(Cow::Borrowed(matched));
        }

        // converting foo/bar to foo_per_bar
        // split the string by the first '/'
        // if the first part is empty, we just return the second part if it's a match with known per unit
        // e.g
        // "test/y" => "per_year"
        // "km/s" => "kilometers_per_second"
        if let Some((first, second)) = unit.split_once('/') {
            return match (
                NON_APPLICABLE_ON_PER_UNIT.contains(&first),
                get_prom_units(first),
                get_prom_per_unit(second),
            ) {
                (true, _, Some(second_part)) | (false, None, Some(second_part)) => {
                    Some(Cow::Owned(format!("per_{second_part}")))
                }
                (false, Some(first_part), Some(second_part)) => {
                    Some(Cow::Owned(format!("{first_part}_per_{second_part}")))
                }
                _ => None,
            };
        }

        // Unmatched units and annotations are ignored
        // e.g. "{request}"
        None
    }

    fn get_prom_units(unit: &str) -> Option<&'static str> {
        match unit {
            // Time
            "d" => Some("days"),
            "h" => Some("hours"),
            "min" => Some("minutes"),
            "s" => Some("seconds"),
            "ms" => Some("milliseconds"),
            "us" => Some("microseconds"),
            "ns" => Some("nanoseconds"),

            // Bytes
            "By" => Some("bytes"),
            "KiBy" => Some("kibibytes"),
            "MiBy" => Some("mebibytes"),
            "GiBy" => Some("gibibytes"),
            "TiBy" => Some("tibibytes"),
            "KBy" => Some("kilobytes"),
            "MBy" => Some("megabytes"),
            "GBy" => Some("gigabytes"),
            "TBy" => Some("terabytes"),
            "B" => Some("bytes"),
            "KB" => Some("kilobytes"),
            "MB" => Some("megabytes"),
            "GB" => Some("gigabytes"),
            "TB" => Some("terabytes"),

            // SI
            "m" => Some("meters"),
            "V" => Some("volts"),
            "A" => Some("amperes"),
            "J" => Some("joules"),
            "W" => Some("watts"),
            "g" => Some("grams"),

            // Misc
            "Cel" => Some("celsius"),
            "Hz" => Some("hertz"),
            "1" => Some("ratio"),
            "%" => Some("percent"),
            _ => None,
        }
    }

    fn get_prom_per_unit(unit: &str) -> Option<&'static str> {
        match unit {
            "s" => Some("second"),
            "m" => Some("minute"),
            "h" => Some("hour"),
            "d" => Some("day"),
            "w" => Some("week"),
            "mo" => Some("month"),
            "y" => Some("year"),
            _ => None,
        }
    }
}
trait FastDisplay {
    fn fast_display(&self) -> impl Display + Copy + use<Self>;
}

#[cfg(feature = "fast")]
mod fast_impl_with {
    use super::FastDisplay;
    use std::fmt::Display;

    #[derive(Copy, Clone)]
    struct RyuDisplay<N: ryu::Float>(N);

    impl<N: ryu::Float> Display for RyuDisplay<N> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut buffer = ryu::Buffer::new();
            let formatted = buffer.format(self.0);
            f.write_str(formatted)
        }
    }

    impl FastDisplay for f64 {
        #[inline]
        fn fast_display(&self) -> impl Display + Copy + use<> {
            RyuDisplay(*self)
        }
    }

    #[derive(Copy, Clone)]
    struct ItoaDisplay<N: itoa::Integer>(N);

    impl<N: itoa::Integer> Display for ItoaDisplay<N> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut buffer = itoa::Buffer::new();
            let formatted = buffer.format(self.0);
            f.write_str(formatted)
        }
    }

    impl FastDisplay for u64 {
        #[inline]
        fn fast_display(&self) -> impl Display + Copy + use<> {
            ItoaDisplay(*self)
        }
    }

    impl FastDisplay for i64 {
        #[inline]
        fn fast_display(&self) -> impl Display + Copy + use<> {
            ItoaDisplay(*self)
        }
    }
}
#[cfg(not(feature = "fast"))]
mod fast_impl_without {
    use super::FastDisplay;
    use std::fmt::Display;

    impl FastDisplay for f64 {
        #[inline]
        fn fast_display(&self) -> impl Display + Copy + use<> {
            *self
        }
    }
    impl FastDisplay for u64 {
        #[inline]
        fn fast_display(&self) -> impl Display + Copy + use<> {
            *self
        }
    }

    impl FastDisplay for i64 {
        #[inline]
        fn fast_display(&self) -> impl Display + Copy + use<> {
            *self
        }
    }
}

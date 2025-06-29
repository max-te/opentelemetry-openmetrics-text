use std::borrow::Cow;
use std::fmt::{Display, Write};

use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::data::ResourceMetrics;

pub struct ToOpenMetrics<'a>(pub &'a ResourceMetrics);

impl<'a> ToOpenMetrics<'a> {
    pub const MIME_TYPE: &'static str =
        "application/openmetrics-text; version=1.0.0; charset=utf-8";
}

macro_rules! fprint {
    ($dst:expr, $($arg:expr),*) => {
        $(
            $arg.fmt($dst)?;
        )*
    };
}

impl<'a> Display for ToOpenMetrics<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let resource_attrs = self.0.resource().into_iter().collect::<Vec<_>>();

        let mut temp_buffer = String::with_capacity(256);
        for scope in self.0.scope_metrics() {
            // write_scope_info(f, scope)?;
            let scope_name = scope.scope().name();

            for metric in scope.metrics() {
                let Ok(typ) = get_type(metric.data()) else {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("Unsupported metric type {metric:?}");
                    continue;
                };
                let mut name = sanitize_name(metric.name());
                let unit = convert_unit(metric.unit());
                if !unit.is_empty() {
                    name.push('_');
                    name.push_str(&unit);
                }

                writeln!(f, "# TYPE {name} {typ}")?;

                if !unit.is_empty() {
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

fn get_type(
    metric: &opentelemetry_sdk::metrics::data::AggregatedMetrics,
) -> Result<&'static str, ()> {
    match metric {
        opentelemetry_sdk::metrics::data::AggregatedMetrics::F64(metric_data) => {
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
        opentelemetry_sdk::metrics::data::AggregatedMetrics::U64(metric_data) => {
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
        opentelemetry_sdk::metrics::data::AggregatedMetrics::I64(metric_data) => {
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
    }
}

fn write_values(
    f: &mut std::fmt::Formatter<'_>,
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

// fn write_scope_info(
//     f: &mut std::fmt::Formatter<'_>,
//     scope: &opentelemetry_sdk::metrics::data::ScopeMetrics,
// ) -> Result<(), std::fmt::Error> {
//     // https://github.com/open-telemetry/opentelemetry-specification/blob/v1.45.0/specification/compatibility/prometheus_and_openmetrics.md#instrumentation-scope-1
//     let scope_name = sanitize_name(scope.scope().name());
//     let scope_attrs = print_attrs(scope.scope().attributes());
//     writeln!(f, "# TYPE {scope_name} info")?;
//     writeln!(f, "{scope_name}_info{{{}}} 1", scope_attrs)?;
//     Ok(())
// }

fn write_histogram<T: FastDisplay + Copy>(
    f: &mut std::fmt::Formatter<'_>,
    name: &str,
    scope_name: &str,
    temp_buffer: &mut String,
    histogram: &opentelemetry_sdk::metrics::data::Histogram<T>,
) -> std::fmt::Result {
    let scope_name_attrs = &[KeyValue::new("otel_scope_name", scope_name.to_owned())];
    let ts = to_timestamp(histogram.time());
    let created = to_timestamp(histogram.start_time());
    temp_buffer.clear();
    let attrs = temp_buffer;
    write_attrs(attrs, scope_name_attrs.iter())?;
    fprint!(f, name, "_created{", attrs, "} ", created, ' ', ts, '\n');
    assert_eq!(
        histogram.temporality(),
        opentelemetry_sdk::metrics::Temporality::Cumulative,
        "Only cumulative Histograms are supported"
    );
    for point in histogram.data_points() {
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

        // Non-compliant but useful:
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

        if !attrs.is_empty() {
            attrs.push(',');
        }
        let mut cumulative_count = 0;
        for (bound, count) in std::iter::zip(point.bounds(), point.bucket_counts()) {
            cumulative_count += count;
            fprint!(
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
            );
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
    f: &mut std::fmt::Formatter<'_>,
    name: &str,
    scope_name: &str,
    temp_buffer: &mut String,
    sum: &opentelemetry_sdk::metrics::data::Sum<T>,
) -> std::fmt::Result {
    let attrs = temp_buffer;
    let scope_name_attrs = &[KeyValue::new("otel_scope_name", scope_name.to_owned())];
    assert_eq!(
        sum.temporality(),
        opentelemetry_sdk::metrics::Temporality::Cumulative,
        "Only cumulative sums are supported"
    );
    if sum.is_monotonic() {
        let ts = to_timestamp(sum.time());
        for point in sum.data_points() {
            attrs.clear();
            write_attrs(attrs, point.attributes().chain(scope_name_attrs))?;
            writeln!(
                f,
                "{name}_total{{{attrs}}} {value} {ts}",
                value = point.value().fast_display(),
            )?;
        }
        Ok(())
    } else {
        let ts = to_timestamp(sum.time());
        for point in sum.data_points() {
            attrs.clear();
            write_attrs(attrs, point.attributes().chain(scope_name_attrs))?;
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
    f: &mut std::fmt::Formatter<'_>,
    name: &str,
    scope_name: &str,
    temp_buffer: &mut String,
    gauge: &opentelemetry_sdk::metrics::data::Gauge<T>,
) -> std::fmt::Result {
    let attrs = temp_buffer;
    let scope_name_attrs = &[KeyValue::new("otel_scope_name", scope_name.to_owned())];
    let ts = to_timestamp(gauge.time());
    for point in gauge.data_points() {
        attrs.clear();
        write_attrs(attrs, point.attributes().chain(scope_name_attrs))?;
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

fn write_attrs<'a, I: Iterator<Item = &'a KeyValue>>(
    f: &mut impl std::fmt::Write,
    attrs: I,
) -> std::fmt::Result {
    let mut first = true;
    for attr in attrs {
        if !first {
            f.write_char(',')?;
        }
        f.write_str(&sanitize_name(attr.key.as_str()))?;
        f.write_str("=\"")?;
        write_escaped(f, &attr.value.as_str())?;
        f.write_char('"')?;
        first = false;
    }
    Ok(())
}

fn write_escaped(f: &mut impl Write, value: &str) -> std::fmt::Result {
    let mut bytes = value.as_bytes();
    let first_escape = memchr::memchr3(b'\\', b'"', b'\n', bytes);
    let Some(first_escape) = first_escape else {
        return f.write_str(unsafe { str::from_utf8_unchecked(bytes) });
    };
    let (head, tail) = bytes.split_at(first_escape);
    f.write_str(unsafe { str::from_utf8_unchecked(head) })?;
    f.write_char('\\')?;
    bytes = tail;

    while let Some(next_escape) = memchr::memchr3(b'\\', b'"', b'\n', bytes) {
        let (head, tail) = bytes.split_at(next_escape);
        f.write_str(unsafe { str::from_utf8_unchecked(head) })?;
        match tail[0] {
            b'\\' => f.write_str("\\\\"),
            b'"' => f.write_str("\\\""),
            b'\n' => f.write_str("\\n"),
            _ => unreachable!(),
        }?;
        bytes = &tail[1..];
    }
    f.write_str(unsafe { str::from_utf8_unchecked(bytes) })
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == ':' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn convert_unit(short_unit: &str) -> Cow<'static, str> {
    match short_unit {
        "" => Cow::Borrowed(""),
        "s" => Cow::Borrowed("seconds"),
        "ms" => Cow::Borrowed("milliseconds"),
        "us" => Cow::Borrowed("microseconds"),
        "ns" => Cow::Borrowed("nanoseconds"),
        "1" => Cow::Borrowed("ratio"),
        "By" => Cow::Borrowed("bytes"),
        _ => Cow::Owned(sanitize_name(short_unit)),
    }
}

trait FastDisplay {
    fn fast_display(&self) -> impl Display + Copy + use<Self>;
}

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
